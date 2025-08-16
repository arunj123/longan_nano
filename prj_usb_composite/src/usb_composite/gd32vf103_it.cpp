/*!
    \file    gd32vf103_it.cpp
    \brief   Main interrupt service routines for the composite USB device

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

#include "usb_device.h"
#include "board.h"

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
        if (usb::is_configured()) {
            // Example: Send 'a' key press and release
            usb::send_keyboard_report(0x00, 0x04); // Press 'a'
            usb::send_keyboard_report(0x00, 0x00); // Release all keys

            // Example: Move mouse cursor right and down
            usb::send_mouse_report(10, 10, 0, 0);

            // Example: Send custom HID report for button 1
            usb::send_custom_hid_report(0x15, 0x01);
        }

        exti_interrupt_flag_clear(USER_KEY_EXTI_LINE);
    }
}

} // extern "C"