#pragma once

#include <cstdint>

// ============================================================================
// USB Chapter 9 Standard Definitions
// ============================================================================

#pragma pack(push, 1)

/// Standard USB Descriptor Header
struct usb_desc_header {
    uint8_t bLength;
    uint8_t bDescriptorType;
};

/// Standard USB Device Descriptor
struct usb_desc_dev {
    usb_desc_header header;
    uint16_t        bcdUSB;
    uint8_t         bDeviceClass;
    uint8_t         bDeviceSubClass;
    uint8_t         bDeviceProtocol;
    uint8_t         bMaxPacketSize0;
    uint16_t        idVendor;
    uint16_t        idProduct;
    uint16_t        bcdDevice;
    uint8_t         iManufacturer;
    uint8_t         iProduct;
    uint8_t         iSerialNumber;
    uint8_t         bNumberConfigurations;
};

/// Standard USB Configuration Descriptor
struct usb_desc_config {
    usb_desc_header header;
    uint16_t        wTotalLength;
    uint8_t         bNumInterfaces;
    uint8_t         bConfigurationValue;
    uint8_t         iConfiguration;
    uint8_t         bmAttributes;
    uint8_t         bMaxPower;
};

/// Standard USB Interface Descriptor
struct usb_desc_itf {
    usb_desc_header header;
    uint8_t         bInterfaceNumber;
    uint8_t         bAlternateSetting;
    uint8_t         bNumEndpoints;
    uint8_t         bInterfaceClass;
    uint8_t         bInterfaceSubClass;
    uint8_t         bInterfaceProtocol;
    uint8_t         iInterface;
};

/// Standard USB Endpoint Descriptor
struct usb_desc_ep {
    usb_desc_header header;
    uint8_t         bEndpointAddress;
    uint8_t         bmAttributes;
    uint16_t        wMaxPacketSize;
    uint8_t         bInterval;
};

/// Standard USB Setup Request
struct usb_req {
    uint8_t  bmRequestType;
    uint8_t  bRequest;
    uint16_t wValue;
    uint16_t wIndex;
    uint16_t wLength;
};

/// CDC Functional Descriptors
struct usb_desc_header_func {
    usb_desc_header header;
    uint8_t         bDescriptorSubtype;
    uint16_t        bcdCDC;
};

struct usb_desc_call_managment_func {
    usb_desc_header header;
    uint8_t         bDescriptorSubtype;
    uint8_t         bmCapabilities;
    uint8_t         bDataInterface;
};

struct usb_desc_acm_func {
    usb_desc_header header;
    uint8_t         bDescriptorSubtype;
    uint8_t         bmCapabilities;
};

struct usb_desc_union_func {
    usb_desc_header header;
    uint8_t         bDescriptorSubtype;
    uint8_t         bMasterInterface;
    uint8_t         bSlaveInterface0;
};

struct usb_desc_LANGID {
    usb_desc_header header;
    uint16_t        wLANGID;
};

struct usb_desc_str {
    usb_desc_header header;
    uint16_t        unicode_string[64];
};

#pragma pack(pop)

#define USB_STRING_LEN(x) (static_cast<uint8_t>((x) * 2U + 2U))

// Descriptor Types
inline constexpr uint8_t USB_DESCTYPE_DEV          = 1U;
inline constexpr uint8_t USB_DESCTYPE_CONFIG       = 2U;
inline constexpr uint8_t USB_DESCTYPE_STR          = 3U;
inline constexpr uint8_t USB_DESCTYPE_ITF          = 4U;
inline constexpr uint8_t USB_DESCTYPE_EP           = 5U;
inline constexpr uint8_t USB_DESCTYPE_DEV_QUAL     = 6U;
inline constexpr uint8_t USB_DESCTYPE_OTHER_SPEED  = 7U;
inline constexpr uint8_t USB_DESCTYPE_ITF_PWR      = 8U;
inline constexpr uint8_t USB_DESCTYPE_CS_INTERFACE = 0x24U;
inline constexpr uint8_t USB_DESCTYPE_CS_ENDPOINT  = 0x25U;

// Descriptor Lengths
inline constexpr uint8_t USB_DEV_DESC_LEN          = 18U;
inline constexpr uint8_t USB_CONFIG_DESC_LEN       = 9U;
inline constexpr uint8_t USB_ITF_DESC_LEN          = 9U;
inline constexpr uint8_t USB_EP_DESC_LEN           = 7U;

// Standard Requests
inline constexpr uint8_t USB_REQ_GET_STATUS        = 0x00U;
inline constexpr uint8_t USB_REQ_CLEAR_FEATURE     = 0x01U;
inline constexpr uint8_t USB_REQ_SET_FEATURE       = 0x03U;
inline constexpr uint8_t USB_REQ_SET_ADDRESS       = 0x05U;
inline constexpr uint8_t USB_REQ_GET_DESCRIPTOR    = 0x06U;
inline constexpr uint8_t USB_REQ_SET_DESCRIPTOR    = 0x07U;
inline constexpr uint8_t USB_REQ_GET_CONFIGURATION = 0x08U;
inline constexpr uint8_t USB_REQ_SET_CONFIGURATION = 0x09U;
inline constexpr uint8_t USB_REQ_GET_INTERFACE     = 0x0AU;
inline constexpr uint8_t USB_REQ_SET_INTERFACE     = 0x0BU;
inline constexpr uint8_t USB_REQ_SYNCH_FRAME       = 0x0CU;

// Request Types & Recipients
inline constexpr uint8_t USB_REQTYPE_MASK          = 0x60U;
inline constexpr uint8_t USB_REQTYPE_STRD          = 0x00U;
inline constexpr uint8_t USB_REQTYPE_CLASS         = 0x20U;
inline constexpr uint8_t USB_REQTYPE_VND           = 0x40U;

inline constexpr uint8_t USB_RECPTYPE_MASK         = 0x1FU;
inline constexpr uint8_t USB_RECPTYPE_DEV          = 0x00U;
inline constexpr uint8_t USB_RECPTYPE_ITF          = 0x01U;
inline constexpr uint8_t USB_RECPTYPE_EP           = 0x02U;
inline constexpr uint8_t USB_RECPTYPE_OTHER        = 0x03U;

// Endpoint Types
inline constexpr uint8_t USB_EPTYPE_CTRL           = 0x00U;
inline constexpr uint8_t USB_EPTYPE_ISOC           = 0x01U;
inline constexpr uint8_t USB_EPTYPE_BULK           = 0x02U;
inline constexpr uint8_t USB_EPTYPE_INTR           = 0x03U;
inline constexpr uint8_t USB_EPTYPE_MASK           = 0x03U;

// Endpoint Attributes
inline constexpr uint8_t USB_EP_ATTR_CTL           = 0x00U;
inline constexpr uint8_t USB_EP_ATTR_ISO           = 0x01U;
inline constexpr uint8_t USB_EP_ATTR_BULK          = 0x02U;
inline constexpr uint8_t USB_EP_ATTR_INT           = 0x03U;

// Endpoint Direction & ID Helpers
#define EP_DIR(ep_addr)             (((ep_addr) >> 7) & 0x01U)
#define EP_ID(ep_addr)              ((ep_addr) & 0x7FU)
#define EP_IN(ep_addr)              ((ep_addr) | 0x80U)
#define EP_OUT(ep_addr)             ((ep_addr) & 0x7FU)

inline constexpr uint16_t EP_MAX_PACKET_SIZE_MASK  = 0x07FFU;
inline constexpr uint8_t  USB_FS_EP0_MAX_LEN       = 64U;

// Standard String Descriptor Indices
inline constexpr uint8_t STR_IDX_LANGID            = 0x00U;
inline constexpr uint8_t STR_IDX_MFC               = 0x01U;
inline constexpr uint8_t STR_IDX_PRODUCT           = 0x02U;
inline constexpr uint8_t STR_IDX_SERIAL            = 0x03U;
inline constexpr uint8_t STR_IDX_CONFIG            = 0x04U;
inline constexpr uint8_t STR_IDX_ITF               = 0x05U;

// Standard USB Classes
inline constexpr uint8_t USB_CLASS_CDC             = 0x02U;
inline constexpr uint8_t USB_CLASS_HID             = 0x03U;
inline constexpr uint8_t USB_CLASS_MSC             = 0x08U;
inline constexpr uint8_t USB_CDC_SUBCLASS_ACM      = 0x02U;
inline constexpr uint8_t USB_CDC_PROTOCOL_AT       = 0x01U;
