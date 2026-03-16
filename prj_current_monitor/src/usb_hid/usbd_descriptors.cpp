#include "usbd_descriptors.h"
#include <cstring>

#define USBD_VID                     0x28E9
#define USBD_PID                     0x1234 // Specific PID for Current Monitor

/* USB standard device descriptor */
usb_desc_dev dev_desc = {
    .header = {
        .bLength          = USB_DEV_DESC_LEN, 
        .bDescriptorType  = USB_DESCTYPE_DEV
    },
    .bcdUSB                = 0x0200,
    .bDeviceClass          = 0x00, 
    .bDeviceSubClass       = 0x00,
    .bDeviceProtocol       = 0x00,
    .bMaxPacketSize0       = USB_FS_EP0_MAX_LEN,
    .idVendor              = USBD_VID,
    .idProduct             = USBD_PID,
    .bcdDevice             = 0x0100,
    .iManufacturer         = usb::STR_IDX_MFC,
    .iProduct              = usb::STR_IDX_PRODUCT,
    .iSerialNumber         = usb::STR_IDX_SERIAL,
    .bNumberConfigurations = 1U,
};

/* USB configuration descriptor */
usb_hid_desc_config_set config_desc = {
    .config = {
        .header = {
            .bLength         = sizeof(usb_desc_config), 
            .bDescriptorType = USB_DESCTYPE_CONFIG
        },
        .wTotalLength         = CONFIG_DESC_SIZE,
        .bNumInterfaces       = 1U,
        .bConfigurationValue  = 1U,
        .iConfiguration       = 0U,
        .bmAttributes         = 0x80, // Bus-powered
        .bMaxPower            = 0xFA  // 500mA
    },

    .custom_hid_itf = {
        .header = { .bLength = sizeof(usb_desc_itf), .bDescriptorType = USB_DESCTYPE_ITF },
        .bInterfaceNumber     = 0,
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
        .wMaxPacketSize       = 64, // Max packet size
        .bInterval            = 0x01 // 1ms for high throughput
    },
    .custom_hid_epout = {
        .header = { .bLength = sizeof(usb_desc_ep), .bDescriptorType = USB_DESCTYPE_EP },
        .bEndpointAddress     = CUSTOM_HID_OUT_EP,
        .bmAttributes         = USB_EP_ATTR_INT,
        .wMaxPacketSize       = 64,
        .bInterval            = 0x01
    }
};

/* USB language ID Descriptor */
static const usb_desc_LANGID language_id_desc = {
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
    .header = { .bLength = USB_STRING_LEN(15), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {'I', 'N', 'A', '2', '1', '9', ' ', 'M', 'o', 'n', 'i', 't', 'o', 'r'}
};

/* USBD serial string */
static usb_desc_str serial_string = {
    .header = { .bLength = USB_STRING_LEN(12), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {0}
};

void *const usbd_strings[] = {
    (uint8_t *)&language_id_desc,
    (uint8_t *)&manufacturer_string,
    (uint8_t *)&product_string,
    (uint8_t *)&serial_string
};