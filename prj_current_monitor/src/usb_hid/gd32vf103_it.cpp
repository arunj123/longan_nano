/*!
    \file    gd32vf103_it.cpp
    \brief   Main interrupt service routines for the current monitor project
*/

#include "usb_device.h"
#include "board.h"
#include "hal/exti.hpp"

extern "C" {

void USBFS_IRQHandler(void) {
    UsbDevice::getInstance().isr();
}

void USBFS_WKUP_IRQHandler(void) {
    UsbDevice::getInstance().wakeup_isr();
}

// This ISR handles the user key on the Longan Nano (PA8)
void EXTI5_9_IRQHandler(void) {
    if (hal::exti::Exti::is_pending(8)) {
        board_key_isr(); // Call the debounced key handler
        hal::exti::Exti::clear_pending(8);
    }
}

} // extern "C"