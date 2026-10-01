/*!
    \file    interrupts.cpp
    \brief   Main interrupt service routines for the composite USB device
*/

#include "usb_device.h"
#include "rotary_encoder.h"
#include "bsp/board.hpp"
#include "hal/exti.hpp"
#include "hal/eclic.hpp"
#include "hal/time.hpp"

volatile bool user_key_pressed = false;

void board_key_init(void) {
    bsp::board::KeyButton::init();
    hal::exti::Exti::map_pin(hal::gpio::Port::A, 8);
    hal::exti::Exti::enable_line(8, hal::exti::Trigger::Falling);
    hal::eclic::Eclic::enable(hal::eclic::Irq::Exti5_9, 1, 0);
}

extern "C" {

void USBFS_IRQHandler(void) {
    UsbDevice::getInstance().isr();
}

void USBFS_WKUP_IRQHandler(void) {
    UsbDevice::getInstance().wakeup_isr();
}

// Handles user key on PA8 (EXTI line 8)
void EXTI5_9_IRQHandler(void) {
    if (hal::exti::Exti::is_pending(8)) {
        static hal::time::Instant last_key_press_time{0};
        const auto debounce_duration = hal::time::Duration::from_ms(50);
        const auto now = hal::time::Instant::now();

        if ((now - last_key_press_time) > debounce_duration) {
            last_key_press_time = now;
            user_key_pressed = true;
        }

        hal::exti::Exti::clear_pending(8);
    }
}

void EXTI10_15_IRQHandler(void) {
    // Rotation pin PB10 (EXTI line 10)
    if (hal::exti::Exti::is_pending(10)) {
        encoder::rotation_isr();
    }
    
    // Key press pin PB12 (EXTI line 12)
    if (hal::exti::Exti::is_pending(12)) {
        encoder::key_isr();
    }
}

} // extern "C"