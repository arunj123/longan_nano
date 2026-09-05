#include "drivers/usb/cdc_acm.hpp"
#include <cstring>

#define USBD_VID                          0x28E9U
#define USBD_PID                          0x018AU

#ifndef CDC_DATA_IN_EP
#define CDC_DATA_IN_EP                    0x81U  /* EP1 IN */
#endif
#ifndef CDC_DATA_OUT_EP
#define CDC_DATA_OUT_EP                   0x03U  /* EP3 OUT */
#endif
#ifndef CDC_CMD_EP
#define CDC_CMD_EP                        0x82U  /* EP2 IN */
#endif

#define USB_CDC_ACM_CONFIG_DESC_SIZE      67U

/* USB standard device descriptor */
static const usb_desc_dev cdc_dev_desc = {
    .header = {
        .bLength          = USB_DEV_DESC_LEN,
        .bDescriptorType  = USB_DESCTYPE_DEV
    },
    .bcdUSB                = 0x0200U,
    .bDeviceClass          = USB_CLASS_CDC,
    .bDeviceSubClass       = 0x00U,
    .bDeviceProtocol       = 0x00U,
    .bMaxPacketSize0       = USB_FS_EP0_MAX_LEN,
    .idVendor              = USBD_VID,
    .idProduct             = USBD_PID,
    .bcdDevice             = 0x0100U,
    .iManufacturer         = STR_IDX_MFC,
    .iProduct              = STR_IDX_PRODUCT,
    .iSerialNumber         = STR_IDX_SERIAL,
    .bNumberConfigurations = 1U
};

/* USB device configuration descriptor */
static const usb_cdc_desc_config_set cdc_config_desc = {
    .config = {
        .header = {
            .bLength         = sizeof(usb_desc_config),
            .bDescriptorType = USB_DESCTYPE_CONFIG
        },
        .wTotalLength         = USB_CDC_ACM_CONFIG_DESC_SIZE,
        .bNumInterfaces       = 0x02U,
        .bConfigurationValue  = 0x01U,
        .iConfiguration       = 0x00U,
        .bmAttributes         = 0x80U,
        .bMaxPower            = 0x32U
    },
    .cmd_itf = {
        .header = {
            .bLength         = sizeof(usb_desc_itf),
            .bDescriptorType = USB_DESCTYPE_ITF
        },
        .bInterfaceNumber     = 0x00U,
        .bAlternateSetting    = 0x00U,
        .bNumEndpoints        = 0x01U,
        .bInterfaceClass      = USB_CLASS_CDC,
        .bInterfaceSubClass   = USB_CDC_SUBCLASS_ACM,
        .bInterfaceProtocol   = USB_CDC_PROTOCOL_AT,
        .iInterface           = 0x00U
    },
    .cdc_header = {
        .header = {
            .bLength         = sizeof(usb_desc_header_func),
            .bDescriptorType = USB_DESCTYPE_CS_INTERFACE
        },
        .bDescriptorSubtype   = 0x00U,
        .bcdCDC               = 0x0110U
    },
    .cdc_call_managment = {
        .header = {
            .bLength         = sizeof(usb_desc_call_managment_func),
            .bDescriptorType = USB_DESCTYPE_CS_INTERFACE
        },
        .bDescriptorSubtype   = 0x01U,
        .bmCapabilities       = 0x00U,
        .bDataInterface       = 0x01U
    },
    .cdc_acm = {
        .header = {
            .bLength         = sizeof(usb_desc_acm_func),
            .bDescriptorType = USB_DESCTYPE_CS_INTERFACE
        },
        .bDescriptorSubtype   = 0x02U,
        .bmCapabilities       = 0x02U
    },
    .cdc_union = {
        .header = {
            .bLength         = sizeof(usb_desc_union_func),
            .bDescriptorType = USB_DESCTYPE_CS_INTERFACE
        },
        .bDescriptorSubtype   = 0x06U,
        .bMasterInterface     = 0x00U,
        .bSlaveInterface0     = 0x01U
    },
    .cdc_cmd_endpoint = {
        .header = {
            .bLength         = sizeof(usb_desc_ep),
            .bDescriptorType = USB_DESCTYPE_EP
        },
        .bEndpointAddress     = CDC_CMD_EP,
        .bmAttributes         = USB_EP_ATTR_INT,
        .wMaxPacketSize       = USB_CDC_CMD_PACKET_SIZE,
        .bInterval            = 0x0AU
    },
    .cdc_data_interface = {
        .header = {
            .bLength         = sizeof(usb_desc_itf),
            .bDescriptorType = USB_DESCTYPE_ITF
        },
        .bInterfaceNumber     = 0x01U,
        .bAlternateSetting    = 0x00U,
        .bNumEndpoints        = 0x02U,
        .bInterfaceClass      = 0x0AU, // USB_CLASS_DATA
        .bInterfaceSubClass   = 0x00U,
        .bInterfaceProtocol   = 0x00U,
        .iInterface           = 0x00U
    },
    .cdc_out_endpoint = {
        .header = {
            .bLength         = sizeof(usb_desc_ep),
            .bDescriptorType = USB_DESCTYPE_EP
        },
        .bEndpointAddress     = CDC_DATA_OUT_EP,
        .bmAttributes         = USB_EP_ATTR_BULK,
        .wMaxPacketSize       = USB_CDC_DATA_PACKET_SIZE,
        .bInterval            = 0x00U
    },
    .cdc_in_endpoint = {
        .header = {
            .bLength         = sizeof(usb_desc_ep),
            .bDescriptorType = USB_DESCTYPE_EP
        },
        .bEndpointAddress     = CDC_DATA_IN_EP,
        .bmAttributes         = USB_EP_ATTR_BULK,
        .wMaxPacketSize       = USB_CDC_DATA_PACKET_SIZE,
        .bInterval            = 0x00U
    }
};

static const usb_desc_LANGID usbd_language_id_desc = {
    .header = {
        .bLength         = sizeof(usb_desc_LANGID),
        .bDescriptorType = USB_DESCTYPE_STR
    },
    .wLANGID             = 0x0409U // ENG_LANGID
};

static const usb_desc_str manufacturer_string = {
    .header = {
        .bLength         = USB_STRING_LEN(10),
        .bDescriptorType = USB_DESCTYPE_STR
    },
    .unicode_string = {'G', 'i', 'g', 'a', 'D', 'e', 'v', 'i', 'c', 'e'}
};

static const usb_desc_str product_string = {
    .header = {
        .bLength         = USB_STRING_LEN(12),
        .bDescriptorType = USB_DESCTYPE_STR
    },
    .unicode_string = {'G', 'D', '3', '2', '-', 'C', 'D', 'C', '_', 'A', 'C', 'M'}
};

static usb_desc_str serial_string = {
    .header = {
        .bLength         = USB_STRING_LEN(12),
        .bDescriptorType = USB_DESCTYPE_STR
    },
    .unicode_string = {0}
};

static void *const usbd_cdc_strings[] = {
    const_cast<void*>(static_cast<const void*>(&usbd_language_id_desc)), // STR_IDX_LANGID (0)
    const_cast<void*>(static_cast<const void*>(&manufacturer_string)),     // STR_IDX_MFC (1)
    const_cast<void*>(static_cast<const void*>(&product_string)),          // STR_IDX_PRODUCT (2)
    static_cast<void*>(&serial_string)                                     // STR_IDX_SERIAL (3)
};

usb_desc cdc_desc = {
    .dev_desc    = const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&cdc_dev_desc)),
    .config_desc = const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(&cdc_config_desc)),
    .bos_desc    = nullptr,
    .strings     = usbd_cdc_strings
};

static usb_cdc_handler g_cdc_handler;

static uint8_t cdc_acm_init(usb_core_driver *udev, uint8_t config_index) {
    (void)config_index;
    usbd_ep_setup(udev, &(cdc_config_desc.cdc_in_endpoint));
    usbd_ep_setup(udev, &(cdc_config_desc.cdc_out_endpoint));
    usbd_ep_setup(udev, &(cdc_config_desc.cdc_cmd_endpoint));

    g_cdc_handler.packet_receive = 1U;
    g_cdc_handler.packet_sent = 1U;
    g_cdc_handler.receive_length = 0U;
    g_cdc_handler.line_coding = acm_line {
        .dwDTERate   = 115200U,
        .bCharFormat = 0U,
        .bParityType = 0U,
        .bDataBits   = 0x08U
    };

    udev->dev.class_data[0] = &g_cdc_handler;
    return USBD_OK;
}

static uint8_t cdc_acm_deinit(usb_core_driver *udev, uint8_t config_index) {
    (void)config_index;
    usbd_ep_clear(udev, CDC_DATA_IN_EP);
    usbd_ep_clear(udev, CDC_DATA_OUT_EP);
    usbd_ep_clear(udev, CDC_CMD_EP);
    return USBD_OK;
}

static uint8_t cdc_acm_req(usb_core_driver *udev, usb_req *req) {
    usb_cdc_handler *cdc = static_cast<usb_cdc_handler*>(udev->dev.class_data[0]);
    if (!cdc) return USBD_FAIL;

    switch (req->bRequest) {
        case CDC_REQ_SET_LINE_CODING:
            // Receive line coding from host on EP0
            udev->dev.transc_out[0].remain_len = req->wLength;
            udev->dev.transc_out[0].xfer_buf = cdc->cmd;
            usbd_ctl_recev(udev);
            break;

        case CDC_REQ_GET_LINE_CODING:
            cdc->cmd[0] = static_cast<uint8_t>(cdc->line_coding.dwDTERate);
            cdc->cmd[1] = static_cast<uint8_t>(cdc->line_coding.dwDTERate >> 8);
            cdc->cmd[2] = static_cast<uint8_t>(cdc->line_coding.dwDTERate >> 16);
            cdc->cmd[3] = static_cast<uint8_t>(cdc->line_coding.dwDTERate >> 24);
            cdc->cmd[4] = cdc->line_coding.bCharFormat;
            cdc->cmd[5] = cdc->line_coding.bParityType;
            cdc->cmd[6] = cdc->line_coding.bDataBits;

            udev->dev.transc_in[0].xfer_buf = cdc->cmd;
            udev->dev.transc_in[0].remain_len = 7U;
            usbd_ctl_send(udev);
            break;

        case CDC_REQ_SET_CONTROL_LINE_STATE:
            usbd_ctl_status_send(udev);
            break;

        default:
            return USBD_FAIL;
    }
    return USBD_OK;
}

static uint8_t cdc_acm_in(usb_core_driver *udev, uint8_t ep_num) {
    (void)ep_num;
    usb_cdc_handler *cdc = static_cast<usb_cdc_handler*>(udev->dev.class_data[0]);
    if (cdc) {
        cdc->packet_sent = 1U;
    }
    return USBD_OK;
}

static uint8_t cdc_acm_out(usb_core_driver *udev, uint8_t ep_num) {
    usb_cdc_handler *cdc = static_cast<usb_cdc_handler*>(udev->dev.class_data[0]);
    if (cdc) {
        cdc->packet_receive = 1U;
        cdc->receive_length = udev->dev.transc_out[ep_num].xfer_count;
    }
    return USBD_OK;
}

usb_class_core cdc_class = {
    .command            = 0,
    .alter_set          = 0,
    .init               = cdc_acm_init,
    .deinit             = cdc_acm_deinit,
    .req_proc           = cdc_acm_req,
    .set_intf           = nullptr,
    .ctlx_in            = nullptr,
    .ctlx_out           = nullptr,
    .data_in            = cdc_acm_in,
    .data_out           = cdc_acm_out,
    .SOF                = nullptr,
    .incomplete_isoc_in = nullptr,
    .incomplete_isoc_out = nullptr
};

uint8_t cdc_acm_check_ready(usb_core_driver *udev) {
    usb_cdc_handler *cdc = static_cast<usb_cdc_handler*>(udev->dev.class_data[0]);
    if (cdc) {
        if (cdc->packet_receive == 1U && cdc->packet_sent == 1U) {
            return 0U;
        }
    }
    return 1U;
}

void cdc_acm_data_send(usb_core_driver *udev) {
    usb_cdc_handler *cdc = static_cast<usb_cdc_handler*>(udev->dev.class_data[0]);
    if (cdc && cdc->receive_length != 0U) {
        cdc->packet_sent = 0U;
        usbd_ep_send(udev, CDC_DATA_IN_EP, cdc->data, cdc->receive_length);
        cdc->receive_length = 0U;
    }
}

void cdc_acm_data_receive(usb_core_driver *udev) {
    usb_cdc_handler *cdc = static_cast<usb_cdc_handler*>(udev->dev.class_data[0]);
    if (cdc) {
        cdc->packet_receive = 0U;
        cdc->packet_sent = 0U;
        usbd_ep_recev(udev, CDC_DATA_OUT_EP, cdc->data, USB_CDC_DATA_PACKET_SIZE);
    }
}
