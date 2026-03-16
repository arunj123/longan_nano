/*!
    \file    usbd_conf.h
    \brief   the header file of USBFS device-mode configuration for the composite device

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef USBD_CONF_H
#define USBD_CONF_H

#include "usb_conf.h"

#define USBD_CFG_MAX_NUM                   1
#define USBD_ITF_MAX_NUM                   3 // Total number of interfaces

#define USB_STR_DESC_MAX_SIZE              64
#define USB_STRING_COUNT                   4U

/*
 * ===================================================================
 * Interface and Endpoint Definitions for the Composite Device
 * ===================================================================
 * The original GigaDevice library files for each class (HID, MSC, etc.)
 * were designed to be used standalone. They often use their own macros
 * like 'USBD_HID_INTERFACE' or 'USBD_MSC_INTERFACE'.
 *
 * To make them work together in a composite device without modifying the
 * library files themselves, we define our composite interface numbers
 * and then create compatibility defines that map the original macros
 * to our new composite interface numbers.
*/

/* Step 1: Define our composite interface numbers */
#define STD_HID_INTERFACE                  0x00U
#define CUSTOM_HID_INTERFACE               0x01U
#define MSC_INTERFACE                      0x02U

/* Step 2: Create compatibility macros for the original library files */
#define USBD_HID_INTERFACE                 STD_HID_INTERFACE
#define USBD_MSC_INTERFACE                 MSC_INTERFACE
/* CUSTOM_HID_INTERFACE is already used by the custom hid core, so no new define is needed. */

/* Step 3: Define endpoints for each interface */

/* Standard HID (Keyboard/Mouse/Consumer) Class Endpoint */
#define STD_HID_IN_EP                      EP_IN(1U)
#define STD_HID_IN_PACKET                  8U

/* Custom HID Class Endpoints */
#define CUSTOM_HID_IN_EP                   EP_IN(2U)
#define CUSTOM_HID_OUT_EP                  EP_OUT(2U)
#define CUSTOM_HID_IN_PACKET               64U
#define CUSTOM_HID_OUT_PACKET              64U

/* Mass Storage Class Endpoints & Configuration */
#define MSC_IN_EP                          EP_IN(3U)
#define MSC_OUT_EP                         EP_OUT(3U)
#define MSC_IN_PACKET                      64U
#define MSC_OUT_PACKET                     64U

/*
 * ===================================================================
 * Compatibility definitions for standalone library files
 * ===================================================================
*/

/* These macros are required by the standalone MSC library files */
#define MSC_MEDIA_PACKET_SIZE              2048U /* Buffer size for media access, must be >= block size */
#define MEM_LUN_NUM                        1U    /* Number of Logical Units (drives), e.g., 1 for one SD card */
#define MSC_DATA_PACKET_SIZE               MSC_IN_PACKET /* Compatibility for usbd_msc_core.h */

#endif /* USBD_CONF_H */