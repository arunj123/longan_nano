/*!
    \file    gd32vf103_it.cpp
    \brief   Main interrupt service routines for the composite USB device
*/

#include "usb_device.h"
#include "board.h"
#include "rotary_encoder.h"
#include "hal/exti.hpp"

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
        board_key_isr();
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