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
        board_key_isr(); // Call the debounced key handler
        exti_interrupt_flag_clear(USER_KEY_EXTI_LINE);
    }
}

void EXTI10_15_IRQHandler(void) {
    // Check if the rotation pin (PB10) triggered the interrupt
    if (RESET != exti_interrupt_flag_get(EXTI_10)) {
        encoder::rotation_isr();
    }
    
    // Check if the key press pin (PB12) triggered the interrupt
    if (RESET != exti_interrupt_flag_get(EXTI_12)) {
        encoder::key_isr();
    }
}

} // extern "C"