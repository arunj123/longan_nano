#include "usbd_descriptors.hpp"
#include "drivers/usb/usb_types.hpp"

/* USB standard device descriptor */
alignas(4) usb_desc_dev msc_dev_desc = {
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
    .iManufacturer         = STR_IDX_MFC,
    .iProduct              = STR_IDX_PRODUCT,
    .iSerialNumber         = STR_IDX_SERIAL,
    .bNumberConfigurations = USBD_CFG_MAX_NUM,
};

/* USB MSC configuration descriptor */
alignas(4) UsbMscConfigDescSet msc_config_desc = {
    .config = {
        .header = {
            .bLength         = sizeof(usb_desc_config), 
            .bDescriptorType = USB_DESCTYPE_CONFIG
        },
        .wTotalLength         = MSC_CONFIG_DESC_SIZE,
        .bNumInterfaces       = 1U, // 1 MSC interface
        .bConfigurationValue  = 1U,
        .iConfiguration       = 0U,
        .bmAttributes         = 0x80, // Bus-powered
        .bMaxPower            = 0x32  // 100mA
    },

    .msc_itf = {
        .header = { .bLength = sizeof(usb_desc_itf), .bDescriptorType = USB_DESCTYPE_ITF },
        .bInterfaceNumber    = MSC_INTERFACE,
        .bAlternateSetting   = 0x00,
        .bNumEndpoints       = 2U,
        .bInterfaceClass     = 0x08U, // USB Mass Storage Class
        .bInterfaceSubClass  = 0x06U, // SCSI Transparent Command Set
        .bInterfaceProtocol  = 0x50U, // Bulk-Only Transport (BBB)
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
alignas(4) static const usb_desc_LANGID usbd_language_id_desc = {
    .header = { .bLength = sizeof(usb_desc_LANGID), .bDescriptorType = USB_DESCTYPE_STR },
    .wLANGID = 0x0409U
};

/* USB manufacture string */
alignas(4) static const usb_desc_str manufacturer_string = {
    .header = { .bLength = USB_STRING_LEN(6), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {'S', 'i', 'p', 'e', 'e', 'd'}
};

/* USB product string */
alignas(4) static const usb_desc_str product_string = {
    .header = { .bLength = USB_STRING_LEN(21), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {'L', 'o', 'n', 'g', 'a', 'n', ' ', 'N', 'a', 'n', 'o', ' ', 'S', 'D', ' ', 'R', 'e', 'a', 'd', 'e', 'r'}
};

/* USBD serial string */
alignas(4) static usb_desc_str serial_string = {
    .header = { .bLength = USB_STRING_LEN(12), .bDescriptorType = USB_DESCTYPE_STR },
    .unicode_string = {0}
};

/* USB string descriptor set */
void *const usbd_msc_strings[] = {
    (uint8_t *)&usbd_language_id_desc,
    (uint8_t *)&manufacturer_string,
    (uint8_t *)&product_string,
    (uint8_t *)&serial_string
};

void set_custom_serial_string(const char *ascii_str) {
    uint8_t len = 0;
    while (ascii_str[len] && len < 63) {
        serial_string.unicode_string[len] = static_cast<uint16_t>(ascii_str[len]);
        len++;
    }
    serial_string.header.bLength = static_cast<uint8_t>(USB_STRING_LEN(len));
    serial_string.header.bDescriptorType = USB_DESCTYPE_STR;
}
