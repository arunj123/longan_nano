#ifndef USBD_DESCRIPTORS_H
#define USBD_DESCRIPTORS_H

#include "drivers/usb/usb_core.hpp"

#include "usb_types.h"

/* Custom HID Report Descriptor for Data Streaming */
const uint8_t custom_hid_report_descriptor[] = {
    0x06, 0x00, 0xFF,  // Usage Page (Vendor-Defined)
    0x09, 0x01,        // Usage (Vendor-Defined)
    0xA1, 0x01,        // Collection (Application)
    
    // INPUT Report (Device -> Host) for current data
    // Format: [Report ID 1, Voltage(2), Current(2), Power(2)]
    0x85, 0x01,        //   Report ID (1)
    0x09, 0x03,        //   Usage (Vendor-Defined 3)
    0x15, 0x00,        //   Logical Minimum (0)
    0x26, 0xFF, 0x00,  //   Logical Maximum (255)
    0x75, 0x08,        //   Report Size (8 bits)
    0x95, 0x08,        //   Report Count (8 bytes)
    0x81, 0x02,        //   Input (Data, Var, Abs)
    
    0xC0               // End Collection
};

#define CUSTOM_HID_REPORT_DESC_LEN    sizeof(custom_hid_report_descriptor)

/* Configuration Descriptor Size */
#define CONFIG_DESC_SIZE              (sizeof(usb_desc_config) + \
                                       sizeof(usb_desc_itf) + \
                                       sizeof(usb::hid::DescHid) + \
                                       sizeof(usb_desc_ep) + \
                                       sizeof(usb_desc_ep))

#ifndef CUSTOM_HID_IN_EP
#define CUSTOM_HID_IN_EP              0x81U
#endif
#ifndef CUSTOM_HID_OUT_EP
#define CUSTOM_HID_OUT_EP             0x01U
#endif

/* Descriptor structure */
typedef struct
{
    usb_desc_config         config;
    usb_desc_itf            custom_hid_itf;
    usb::hid::DescHid       custom_hid_desc;
    usb_desc_ep             custom_hid_epin;
    usb_desc_ep             custom_hid_epout;
} usb_hid_desc_config_set;

extern usb_desc_dev dev_desc;
extern usb_hid_desc_config_set config_desc;
extern void *const usbd_strings[];

#endif /* USBD_DESCRIPTORS_H */