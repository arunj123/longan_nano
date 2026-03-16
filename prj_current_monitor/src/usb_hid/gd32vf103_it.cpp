/*!
    \file    gd32vf103_it.cpp
    \brief   Main interrupt service routines for the current monitor project
*/

#include "usb_device.h"
#include "board.h"
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

// This ISR handles the user key on the Longan Nano (PA8)
void EXTI5_9_IRQHandler(void) {
    if (RESET != exti_interrupt_flag_get(USER_KEY_EXTI_LINE)) {
        board_key_isr(); // Call the debounced key handler
        exti_interrupt_flag_clear(USER_KEY_EXTI_LINE);
    }
}

} // extern "C"