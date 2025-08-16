/*!
    \file    usbd_descriptors.cpp
    \brief   USB composite device descriptors

    \version 2025-02-10, firmware for GD32VF103
*/

#include "usbd_descriptors.h"
#include <cstring>

#define USBD_VID                     0x28E9
#define USBD_PID                     0xABCD // New PID for the composite device

/* USB standard device descriptor */
const usb_desc_dev composite_dev_desc = {
    .header = {
        .bLength          = USB_DEV_DESC_LEN, 
        .bDescriptorType  = USB_DESCTYPE_DEV
    },
    .bcdUSB                = 0x0200,
    .bDeviceClass          = 0x00, // Class defined in interface
    .bDeviceSubClass       = 0x00,
    .bDeviceProtocol       = 0x00,
    .bMaxPacketSize0       = USB_FS_EP0_MAX_LEN,
    .idVendor              = USBD_VID,
    .idProduct             = USBD_PID,
    .bcdDevice             = 0x0100,
    .iManufacturer         = usb::STR_IDX_MFC,
    .iProduct              = usb::STR_IDX_PRODUCT,
    .iSerialNumber         = usb::STR_IDX_SERIAL,
    .bNumberConfigurations = USBD_CFG_MAX_NUM,
};

/* USB composite configuration descriptor */
const usb_composite_desc_config_set composite_config_desc = {
    .config = {
        .header = {
            .bLength         = sizeof(usb_desc_config), 
            .bDescriptorType = USB_DESCTYPE_CONFIG
        },
        .wTotalLength         = COMPOSITE_CONFIG_DESC_SIZE,
        .bNumInterfaces       = 3U, // We have 3 interfaces
        .bConfigurationValue  = 1U,
        .iConfiguration       = 0U,
        .bmAttributes         = 0x80, // Bus-powered
        .bMaxPower            = 0xFA  // 500mA
    },

    /******************** Standard HID Interface (Interface 0) ********************/
    .std_hid_itf = {
        .header = { .bLength = sizeof(usb_desc_itf), .bDescriptorType = USB_DESCTYPE_ITF },
        .bInterfaceNumber     = STD_HID_INTERFACE,
        .bAlternateSetting    = 0x00,
        .bNumEndpoints        = 1U,
        .bInterfaceClass      = usb::hid::HID_CLASS,
        .bInterfaceSubClass   = 0x00, // No Subclass
        .bInterfaceProtocol   = 0x00, // No Protocol
        .iInterface           = 0x00
    },
    .std_hid_desc = {
        .header = { .bLength = sizeof(usb::hid::DescHid), .bDescriptorType = usb::hid::DESC_TYPE_HID },
        .bcdHID               = 0x0111,
        .bCountryCode         = 0x00,
        .bNumDescriptors      = 1U,
        .bDescriptorType      = usb::hid::DESC_TYPE_REPORT,
        .wDescriptorLength    = STD_HID_REPORT_DESC_LEN,
    },
    .std_hid_epin = {
        .header = { .bLength = sizeof(usb_desc_ep), .bDescriptorType = USB_DESCTYPE_EP },
        .bEndpointAddress     = STD_HID_IN_EP,
        .bmAttributes         = USB_EP_ATTR_INT,
        .wMaxPacketSize       = STD_HID_IN_PACKET,
        .bInterval            = 0x0A // 10ms
    },

    /******************** Custom HID Interface (Interface 1) ********************/
    .custom_hid_itf = {
        .header = { .bLength = sizeof(usb_desc_itf), .bDescriptorType = USB_DESCTYPE_ITF },
        .bInterfaceNumber     = CUSTOM_HID_INTERFACE,
        .bAlternateSetting    = 0x00,
        .bNumEndpoints        = 2U,
        .bInterfaceClass      = usb::hid::HID_CLASS,
        .bInterfaceSubClass   = 0x00,
        .bInterfaceProtocol   = 0x00,
        .iInterface           = 0x00
    },
    .custom_hid_desc = {
        .header = { .bLength = sizeof(usb::hid::DescHid), .bDescriptorType = usb::hid::DESC_TYPE_HID },
        .bcdHID               = 0x0111,
        .bCountryCode         = 0x00,
        .bNumDescriptors      = 1U,
        .bDescriptorType      = usb::hid::DESC_TYPE_REPORT,
        .wDescriptorLength    = CUSTOM_HID_REPORT_DESC_LEN,
    },
    .custom_hid_epin = {
        .header = { .bLength = sizeof(usb_desc_ep), .bDescriptorType = USB_DESCTYPE_EP },
        .bEndpointAddress     = CUSTOM_HID_IN_EP,
        .bmAttributes         = USB_EP_ATTR_INT,
        .wMaxPacketSize       = CUSTOM_HID_IN_PACKET,
        .bInterval            = 0x20
    },
    .custom_hid_epout = {
        .header = { .bLength = sizeof(usb_desc_ep), .bDescriptorType = USB_DESCTYPE_EP },
        .bEndpointAddress     = CUSTOM_HID_OUT_EP,
        .bmAttributes         = USB_EP_ATTR_INT,
        .wMaxPacketSize       = CUSTOM_HID_OUT_PACKET,
        .bInterval            = 0x20
    },

    /******************** MSC Interface (Interface 2) ********************/
    .msc_itf = {
        .header = { .bLength = sizeof(usb_desc_itf), .bDescriptorType = USB_DESCTYPE_ITF },
        .bInterfaceNumber    = MSC_INTERFACE,
        .bAlternateSetting   = 0x00,
        .bNumEndpoints       = 2U,
        .bInterfaceClass     = usb::msc::MSC_CLASS,
        .bInterfaceSubClass  = usb::msc::MSC_SUBCLASS_SCSI,
        .bInterfaceProtocol  = usb::msc::MSC_PROTOCOL_BBB,
        .iInterface          = 0x00
    },
    .msc_epout = {
        .header = { .bLength = sizeof(usb_desc_ep), .bDescriptorType = USB_DESCTYPE_EP },
        .bEndpointAddress    = MSC_OUT_EP,
        .bmAttributes        = USB_EP_ATTR_BULK,
        .wMaxPacketSize      = MSC_OUT_PACKET,
        .bInterval           = 0x00
    },
    .msc_epin = {
        .header = { .bLength = sizeof(usb_desc_ep), .bDescriptorType = USB_DESCTYPE_EP },
        .bEndpointAddress    = MSC_IN_EP,
        .bmAttributes        = USB_EP_ATTR_BULK,
        .wMaxPacketSize      = MSC_IN_PACKET,
        .bInterval           = 0x00
    }
};

/* USB language ID Descriptor */
static const usb_desc_LANGID usbd_language_id_desc = {
    .header = { .bLength = sizeof(usb_desc_LANGID), .bDescriptorType = USB_DESCTYPE_STR },
    .wLANGID = usb::ENG_LANGID
};

/* USB manufacture string */
static const usb_desc_str manufacturer_string = {
    .header = { .bLength = USB_STRING_LEN(10), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {'G', 'i', 'g', 'a', 'D', 'e', 'v', 'i', 'c', 'e'}
};

/* USB product string */
static const usb_desc_str product_string = {
    .header = { .bLength = USB_STRING_LEN(18), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {'G', 'D', '3', '2', ' ', 'C', 'o', 'm', 'p', 'o', 's', 'i', 't', 'e', ' ', 'D', 'e', 'v'}
};

/* USBD serial string */
static usb_desc_str serial_string = {
    .header = { .bLength = USB_STRING_LEN(12), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {0} // Fix for -Wmissing-field-initializers
};

/* USB string descriptor set (standard C++ initialization) */
void *const usbd_composite_strings[] = {
    (uint8_t *)&usbd_language_id_desc,
    (uint8_t *)&manufacturer_string,
    (uint8_t *)&product_string,
    (uint8_t *)&serial_string
};