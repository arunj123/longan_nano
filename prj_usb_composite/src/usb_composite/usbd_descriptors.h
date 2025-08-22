/*!
    \file    usbd_descriptors.h
    \brief   Header file for USB composite device descriptors

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef USBD_DESCRIPTORS_H
#define USBD_DESCRIPTORS_H

extern "C" {
    #include "usbd_core.h"
}

#include "usb_types.h"

/**
 * @brief Standard HID Report Descriptor for a Composite Device
 *
 * This descriptor defines three top-level Application Collections, each with a unique
 * Report ID, effectively creating three distinct HID devices over a single endpoint:
 * 1. Report ID 1: A standard 3-button mouse with a scroll wheel.
 * 2. Report ID 2: A standard 6-key rollover keyboard.
 * 3. Report ID 3: A consumer control device for media keys (e.g., volume, play/pause).
 */
const uint8_t std_hid_report_descriptor[] = {
    //
    // ------------- Part 1: Mouse Definition -------------
    //
    0x05, 0x01,       // USAGE_PAGE (Generic Desktop)
    0x09, 0x02,       // USAGE (Mouse)
    0xA1, 0x01,       // COLLECTION (Application)
    0x85, 0x01,       //   REPORT_ID (1) - This makes the mouse report unique
    0x09, 0x01,       //   USAGE (Pointer)
    0xA1, 0x00,       //   COLLECTION (Physical)
    // --- Mouse Buttons (3 bits) ---
    0x05, 0x09,       //     USAGE_PAGE (Button)
    0x19, 0x01,       //     USAGE_MINIMUM (Button 1)
    0x29, 0x03,       //     USAGE_MAXIMUM (Button 3)
    0x15, 0x00,       //     LOGICAL_MINIMUM (0)
    0x25, 0x01,       //     LOGICAL_MAXIMUM (1)
    0x95, 0x03,       //     REPORT_COUNT (3) - Three 1-bit fields for buttons 1, 2, 3
    0x75, 0x01,       //     REPORT_SIZE (1)
    0x81, 0x02,       //     INPUT (Data,Var,Abs) - The 3 button bits
    // --- Padding (5 bits) ---
    0x95, 0x01,       //     REPORT_COUNT (1) - One 5-bit field
    0x75, 0x05,       //     REPORT_SIZE (5)
    0x81, 0x01,       //     INPUT (Cnst,Ary,Abs) - 5 bits of constant padding to round out the byte
    // --- X and Y Axes (2 bytes) ---
    0x05, 0x01,       //     USAGE_PAGE (Generic Desktop)
    0x09, 0x30,       //     USAGE (X)
    0x09, 0x31,       //     USAGE (Y)
    0x15, 0x81,       //     LOGICAL_MINIMUM (-127)
    0x25, 0x7F,       //     LOGICAL_MAXIMUM (127)
    0x75, 0x08,       //     REPORT_SIZE (8) - 8 bits per axis
    0x95, 0x02,       //     REPORT_COUNT (2) - For X and Y
    0x81, 0x06,       //     INPUT (Data,Var,Rel) - Two 8-bit relative values for X and Y
    0xC0,             //   END_COLLECTION (Physical)
    0xC0,             // END_COLLECTION (Application)

    //
    // ------------ Part 2: Keyboard Definition ------------
    //
    0x05, 0x01,       // USAGE_PAGE (Generic Desktop)
    0x09, 0x06,       // USAGE (Keyboard)
    0xA1, 0x01,       // COLLECTION (Application)
    0x85, 0x02,       //   REPORT_ID (2) - This makes the keyboard report unique
    // --- Modifier Keys (1 byte) ---
    0x05, 0x07,       //   USAGE_PAGE (Keyboard/Keypad)
    0x19, 0xE0,       //   USAGE_MINIMUM (Keyboard LeftControl)
    0x29, 0xE7,       //   USAGE_MAXIMUM (Keyboard Right GUI)
    0x15, 0x00,       //   LOGICAL_MINIMUM (0)
    0x25, 0x01,       //   LOGICAL_MAXIMUM (1)
    0x75, 0x01,       //   REPORT_SIZE (1)
    0x95, 0x08,       //   REPORT_COUNT (8) - 8 bits for 8 modifiers (L/R Ctrl, Shift, Alt, GUI)
    0x81, 0x02,       //   INPUT (Data,Var,Abs) - The modifier byte
    // --- Reserved Byte ---
    0x95, 0x01,       //   REPORT_COUNT (1)
    0x75, 0x08,       //   REPORT_SIZE (8)
    0x81, 0x01,       //   INPUT (Cnst,Ary,Abs) - A constant reserved byte
    // --- Keypress Array (6 bytes) ---
    0x95, 0x06,       //   REPORT_COUNT (6) - Can report up to 6 simultaneous keypresses
    0x75, 0x08,       //   REPORT_SIZE (8) - Each keycode is 8 bits
    0x15, 0x00,       //   LOGICAL_MINIMUM (0)
    0x25, 0x65,       //   LOGICAL_MAXIMUM (101) - Defines the range of valid keycodes
    0x05, 0x07,       //   USAGE_PAGE (Keyboard/Keypad)
    0x19, 0x00,       //   USAGE_MINIMUM (Reserved (no event indicated))
    0x29, 0x65,       //   USAGE_MAXIMUM (Keyboard Application)
    0x81, 0x00,       //   INPUT (Data,Ary,Abs) - The 6-byte key array
    0xC0,             // END_COLLECTION

    //
    // -------- Part 3: Consumer Control Definition --------
    //
    0x05, 0x0C,       // USAGE_PAGE (Consumer)
    0x09, 0x01,       // USAGE (Consumer Control)
    0xA1, 0x01,       // COLLECTION (Application)
    0x85, 0x03,       //   REPORT_ID (3) - This makes the consumer report unique
    // --- Consumer Control Button (2 bytes) ---
    0x19, 0x00,       //   USAGE_MINIMUM (Unassigned)
    0x2A, 0x3C, 0x02, //   USAGE_MAXIMUM (AC Forward) - Defines a large range of possible media keys
    0x15, 0x00,       //   LOGICAL_MINIMUM (0)
    0x26, 0x3C, 0x02, //   LOGICAL_MAXIMUM (572)
    0x95, 0x01,       //   REPORT_COUNT (1)
    0x75, 0x10,       //   REPORT_SIZE (16) - One 16-bit field for the pressed key's usage code
    0x81, 0x00,       //   INPUT (Data,Ary,Abs)
    0xC0              // END_COLLECTION
};

/* Custom HID Report Descriptor */
const uint8_t custom_hid_report_descriptor[] = {
    0x06, 0x00, 0xFF,  // Usage Page (Vendor-Defined)
    0x09, 0x01,        // Usage (Vendor-Defined)
    0xA1, 0x01,        // Collection (Application)
    
    // OUTPUT Report (Host -> Device) for image data
    0x09, 0x02,        //   Usage (Vendor-Defined 2)
    0x15, 0x00,        //   Logical Minimum (0)
    0x26, 0xFF, 0x00,  //   Logical Maximum (255)
    0x75, 0x08,        //   Report Size (8 bits)
    0x95, 0x40,        //   Report Count (64 bytes)
    0x91, 0x02,        //   Output (Data, Var, Abs)
    
    // INPUT Report (Device -> Host), optional but good practice
    0x09, 0x03,        //   Usage (Vendor-Defined 3)
    0x95, 0x40,        //   Report Count (64 bytes)
    0x81, 0x02,        //   Input (Data, Var, Abs)
    
    0xC0               // End Collection
};

/* Report descriptor lengths */
#define STD_HID_REPORT_DESC_LEN       sizeof(std_hid_report_descriptor)
#define CUSTOM_HID_REPORT_DESC_LEN    sizeof(custom_hid_report_descriptor)

/* Define a size for the configuration when MSC is disabled */
#define HID_ONLY_CONFIG_DESC_SIZE     (sizeof(usb_desc_config) + \
                                       sizeof(usb_desc_itf) + sizeof(usb::hid::DescHid) + sizeof(usb_desc_ep) + \
                                       sizeof(usb_desc_itf) + sizeof(usb::hid::DescHid) + sizeof(usb_desc_ep) * 2)

/* Total size of the full composite descriptor (remains the same) */
#define COMPOSITE_CONFIG_DESC_SIZE    (sizeof(usb_desc_config) + \
                                       sizeof(usb_desc_itf) + sizeof(usb::hid::DescHid) + sizeof(usb_desc_ep) + \
                                       sizeof(usb_desc_itf) + sizeof(usb::hid::DescHid) + sizeof(usb_desc_ep) * 2 + \
                                       sizeof(usb_desc_itf) + sizeof(usb_desc_ep) * 2)

/* Report IDs */
#define REPORT_ID_MOUSE             1U
#define REPORT_ID_KEYBOARD          2U
#define REPORT_ID_CONSUMER          3U

/* Composite descriptor structure */
typedef struct
{
    usb_desc_config         config;

    /* Standard HID (Mouse, Keyboard, Consumer) Interface */
    usb_desc_itf            std_hid_itf;
    usb::hid::DescHid       std_hid_desc;
    usb_desc_ep             std_hid_epin;

    /* Custom HID Interface */
    usb_desc_itf            custom_hid_itf;
    usb::hid::DescHid       custom_hid_desc;
    usb_desc_ep             custom_hid_epin;
    usb_desc_ep             custom_hid_epout;

    /* MSC Interface */
    usb_desc_itf            msc_itf;
    usb_desc_ep             msc_epout;
    usb_desc_ep             msc_epin;
} usb_composite_desc_config_set;

extern usb_desc_dev composite_dev_desc;
extern usb_composite_desc_config_set composite_config_desc;
extern void *const usbd_composite_strings[];

#endif /* USBD_DESCRIPTORS_H */