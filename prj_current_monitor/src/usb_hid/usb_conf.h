/*!
    \file    usb_conf.h
    \brief   USBFS driver basic configuration for the composite device

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef USB_CONF_H
#define USB_CONF_H

#ifdef __cplusplus
#include <cstdlib>
extern "C" {
#else
#include <stdlib.h>
#endif

#include "gd32vf103.h"
#ifdef __cplusplus
}
#endif

/* USB Core and Driver Config */
#define USB_SOF_OUTPUT              1U
#define USB_LOW_POWER               0U

/* Select USB Core */
#define USE_USB_FS

/* Select USB Operating Mode */
#define USE_DEVICE_MODE

#ifndef USE_DEVICE_MODE
    #ifndef USE_HOST_MODE
        #error "USE_DEVICE_MODE or USE_HOST_MODE should be defined"
    #endif
#endif

/* USB FIFO Size Configuration */
/*
 * Total available FIFO size for the GD32VF103 is 1.25KB (1280 bytes or 320 words).
 * RX FIFO is shared for all OUT endpoints.
 * TX FIFOs are dedicated for each IN endpoint.
 * The sum of all FIFO sizes must not exceed the total available size.
 * Sizes are in 32-bit words (4 bytes).
*/

// RX FIFO for all OUT endpoints (Custom HID OUT, MSC OUT)
#define RX_FIFO_FS_SIZE             128U  // 512 bytes

// TX FIFO for Control Endpoint 0 IN
#define TX0_FIFO_FS_SIZE            64U   // 256 bytes

// TX FIFO for Standard HID IN Endpoint 1
#define TX1_FIFO_FS_SIZE            16U   // 64 bytes (Packet size is 8 bytes)

// TX FIFO for Custom HID IN Endpoint 2
#define TX2_FIFO_FS_SIZE            16U   // 64 bytes (Packet size is 2 bytes)

// TX FIFO for MSC IN Endpoint 3
#define TX3_FIFO_FS_SIZE            64U   // 256 bytes (Packet size is 64 bytes)

/* Total allocated: 128 + 64 + 16 + 16 + 64 = 288 words. This is less than the 320-word limit. */

#endif /* USB_CONF_H */