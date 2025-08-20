/*!
    \file    gd32vf103_it.cpp
    \brief   Main interrupt service routines for the composite USB device

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

#include "usb_device.h"
#include "board.h"
#include "rotary_encoder.h"
#include <cstdio>
extern "C" {
#include "systick.h" // For delay_1ms
}

volatile bool user_key_pressed = false;

extern "C" {

void USBFS_IRQHandler(void) {
    UsbDevice::getInstance().isr();
}

void USBFS_WKUP_IRQHandler(void) {
    UsbDevice::getInstance().wakeup_isr();
}

void TIMER2_IRQHandler(void) {
    UsbDevice::getInstance().timer_isr();
}

// This ISR now handles the single user key on the Longan Nano (PA8)
void EXTI5_9_IRQHandler(void) {
    if (RESET != exti_interrupt_flag_get(USER_KEY_EXTI_LINE)) {
        
        user_key_pressed = true; // Set the flag to indicate the key was pressed

        exti_interrupt_flag_clear(USER_KEY_EXTI_LINE);
    }
}

// This handler for the button press on PB0 is the same as it was for PA0.
void EXTI0_IRQHandler(void) {
    if (RESET != exti_interrupt_flag_get(EXTI_0)) {
        encoder::key_isr();
    }
}

// ADD this new handler for pins 10 through 15.
// We check that it was specifically our pin (EXTI_10) that triggered it.
void EXTI10_15_IRQHandler(void) {
    if (RESET != exti_interrupt_flag_get(EXTI_10)) {
        encoder::rotation_isr();
    }
}

} // extern "C"