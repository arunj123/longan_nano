#pragma once

#include "drivers/usb/usb_core.hpp"

#pragma pack(push, 1)

struct acm_line {
    uint32_t dwDTERate;
    uint8_t  bCharFormat;
    uint8_t  bParityType;
    uint8_t  bDataBits;
};

struct usb_desc_header_func {
    usb_desc_header header;
    uint8_t   bDescriptorSubtype;
    uint16_t  bcdCDC;
};

struct usb_desc_call_managment_func {
    usb_desc_header header;
    uint8_t  bDescriptorSubtype;
    uint8_t  bmCapabilities;
    uint8_t  bDataInterface;
};

struct usb_desc_acm_func {
    usb_desc_header header;
    uint8_t  bDescriptorSubtype;
    uint8_t  bmCapabilities;
};

struct usb_desc_union_func {
    usb_desc_header header;
    uint8_t  bDescriptorSubtype;
    uint8_t  bMasterInterface;
    uint8_t  bSlaveInterface0;
};

struct usb_cdc_desc_config_set {
    usb_desc_config               config;
    usb_desc_itf                  cmd_itf;
    usb_desc_header_func          cdc_header;
    usb_desc_call_managment_func  cdc_call_managment;
    usb_desc_acm_func             cdc_acm;
    usb_desc_union_func           cdc_union;
    usb_desc_ep                   cdc_cmd_endpoint;
    usb_desc_itf                  cdc_data_interface;
    usb_desc_ep                   cdc_out_endpoint;
    usb_desc_ep                   cdc_in_endpoint;
};

#pragma pack(pop)

#ifndef USB_CLASS_CDC
#define USB_CLASS_CDC 0x02U
#endif

#ifndef USB_CDC_SUBCLASS_ACM
#define USB_CDC_SUBCLASS_ACM 0x02U
#endif

#ifndef USB_CDC_PROTOCOL_AT
#define USB_CDC_PROTOCOL_AT 0x01U
#endif

#ifndef USB_DESCTYPE_CS_INTERFACE
#define USB_DESCTYPE_CS_INTERFACE 0x24U
#endif

#ifndef CDC_ACM_DATA_PACKET_SIZE
#define CDC_ACM_DATA_PACKET_SIZE 64U
#endif

#ifndef CDC_ACM_CMD_PACKET_SIZE
#define CDC_ACM_CMD_PACKET_SIZE  8U
#endif

#ifndef USB_CDC_DATA_PACKET_SIZE
#define USB_CDC_DATA_PACKET_SIZE 64U
#endif

#ifndef USB_CDC_CMD_PACKET_SIZE
#define USB_CDC_CMD_PACKET_SIZE  8U
#endif

#ifndef USB_CDC_RX_LEN
#define USB_CDC_RX_LEN           64U
#endif

inline constexpr uint8_t CDC_REQ_SET_LINE_CODING        = 0x20U;
inline constexpr uint8_t CDC_REQ_GET_LINE_CODING        = 0x21U;
inline constexpr uint8_t CDC_REQ_SET_CONTROL_LINE_STATE = 0x22U;

struct usb_cdc_handler {
    uint8_t  data[CDC_ACM_DATA_PACKET_SIZE];
    uint8_t  cmd[CDC_ACM_CMD_PACKET_SIZE];
    uint8_t  packet_sent;
    uint8_t  packet_receive;
    uint32_t receive_length;
    acm_line line_coding;
};

extern usb_desc cdc_desc;
extern usb_class_core cdc_class;

uint8_t cdc_acm_check_ready(usb_core_driver *udev);
void cdc_acm_data_send(usb_core_driver *udev);
void cdc_acm_data_receive(usb_core_driver *udev);
