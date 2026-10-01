/*!
    \file    gd32vf103_it.cpp
    \brief   Main interrupt service routines for the current monitor project
*/

#include "usb_device.h"

extern "C" {

void USBFS_IRQHandler(void) {
    UsbDevice::getInstance().isr();
}

void USBFS_WKUP_IRQHandler(void) {
    UsbDevice::getInstance().wakeup_isr();
}

} // extern "C"