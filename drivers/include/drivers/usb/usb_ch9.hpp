#pragma once

#include <cstddef>
#include <cstdint>
#include <utility>
#include <algorithm>

/**
 * @file usb_ch9.hpp
 * @brief Modern C++23 USB 2.0 Chapter 9 definitions, structures, and compile-time generators.
 */

// Chapter 9 Descriptor Lengths
inline constexpr uint8_t USB_DEV_QUALIFIER_DESC_LEN = 0x0AU;
inline constexpr uint8_t USB_DEV_DESC_LEN           = 0x12U;
inline constexpr uint8_t USB_CFG_DESC_LEN           = 0x09U;
inline constexpr uint8_t USB_ITF_DESC_LEN           = 0x09U;
inline constexpr uint8_t USB_EP_DESC_LEN            = 0x07U;
inline constexpr uint8_t USB_IAD_DESC_LEN           = 0x08U;
inline constexpr uint8_t USB_OTG_DESC_LEN           = 0x03U;
inline constexpr uint8_t USB_SETUP_PACKET_LEN       = 0x08U;

// Transfer Direction
inline constexpr uint8_t USB_TRX_MASK               = 0x80U;
inline constexpr uint8_t USB_TRX_OUT                = 0x00U;
inline constexpr uint8_t USB_TRX_IN                 = 0x80U;

// Request Types
inline constexpr uint8_t USB_REQTYPE_STRD           = 0x00U;
inline constexpr uint8_t USB_REQTYPE_CLASS          = 0x20U;
inline constexpr uint8_t USB_REQTYPE_VENDOR         = 0x40U;
inline constexpr uint8_t USB_REQTYPE_MASK           = 0x60U;

// Recipient Types
enum _usb_recp_type {
    USB_RECPTYPE_DEV  = 0x0U,
    USB_RECPTYPE_ITF  = 0x1U,
    USB_RECPTYPE_EP   = 0x2U,
    USB_RECPTYPE_MASK = 0x3U
};

// Power Status
inline constexpr uint8_t USBD_BUS_POWERED           = 0x00U;
inline constexpr uint8_t USBD_SELF_POWERED          = 0x01U;
inline constexpr uint8_t USB_STATUS_REMOTE_WAKEUP   = 2U;
inline constexpr uint8_t USB_STATUS_SELF_POWERED    = 1U;

// Standard Requests
enum _usb_request {
    USB_GET_STATUS        = 0x0U,
    USB_CLEAR_FEATURE     = 0x1U,
    USB_RESERVED2         = 0x2U,
    USB_SET_FEATURE       = 0x3U,
    USB_RESERVED4         = 0x4U,
    USB_SET_ADDRESS       = 0x5U,
    USB_GET_DESCRIPTOR    = 0x6U,
    USB_SET_DESCRIPTOR    = 0x7U,
    USB_GET_CONFIGURATION = 0x8U,
    USB_SET_CONFIGURATION = 0x9U,
    USB_GET_INTERFACE     = 0xAU,
    USB_SET_INTERFACE     = 0xBU,
    USB_SYNCH_FRAME       = 0xCU
};

// Standard Descriptor Types
enum _usb_desctype {
    USB_DESCTYPE_DEV              = 0x1U,
    USB_DESCTYPE_CONFIG           = 0x2U,
    USB_DESCTYPE_STR              = 0x3U,
    USB_DESCTYPE_ITF              = 0x4U,
    USB_DESCTYPE_EP               = 0x5U,
    USB_DESCTYPE_DEV_QUALIFIER    = 0x6U,
    USB_DESCTYPE_OTHER_SPD_CONFIG = 0x7U,
    USB_DESCTYPE_ITF_POWER        = 0x8U,
    USB_DESCTYPE_IAD              = 0xBU,
    USB_DESCTYPE_BOS              = 0xFU,
    USB_DESCTYPE_HID              = 0x21U,
    USB_DESCTYPE_REPORT           = 0x22U
};

// Endpoint Attributes
enum _usbx_type {
    USB_EP_ATTR_CTL  = 0x0U,
    USB_EP_ATTR_ISO  = 0x1U,
    USB_EP_ATTR_BULK = 0x2U,
    USB_EP_ATTR_INT  = 0x3U
};

inline constexpr uint8_t FEATURE_SELECTOR_EP        = 0x00U;
inline constexpr uint8_t FEATURE_SELECTOR_DEV       = 0x01U;
inline constexpr uint8_t FEATURE_SELECTOR_REMOTEWAKEUP = 0x01U;

#define BYTE_LOW(x)          (static_cast<uint8_t>((x) & 0x00FFU))
#define BYTE_HIGH(x)         (static_cast<uint8_t>(((x) & 0xFF00U) >> 8))
#define USB_MIN(a, b)        (((a) < (b)) ? (a) : (b))

inline constexpr uint8_t  USB_DEFAULT_CONFIG        = 0U;
inline constexpr uint8_t  USB_CLASS_HID             = 0x03U;
inline constexpr uint8_t  USB_CLASS_MSC             = 0x08U;

inline constexpr uint32_t DATA_STAGE_TIMEOUT        = 5000U;
inline constexpr uint32_t NODATA_STAGE_TIMEOUT      = 50U;

#pragma pack(push, 1)

struct usb_req {
    uint8_t  bmRequestType;
    uint8_t  bRequest;
    uint16_t wValue;
    uint16_t wIndex;
    uint16_t wLength;
};

union usb_setup {
    uint8_t data[8];
    usb_req req;
};

struct usb_desc_header {
    uint8_t bLength;
    uint8_t bDescriptorType;
};

struct usb_desc_dev {
    usb_desc_header header;
    uint16_t bcdUSB;
    uint8_t  bDeviceClass;
    uint8_t  bDeviceSubClass;
    uint8_t  bDeviceProtocol;
    uint8_t  bMaxPacketSize0;
    uint16_t idVendor;
    uint16_t idProduct;
    uint16_t bcdDevice;
    uint8_t  iManufacturer;
    uint8_t  iProduct;
    uint8_t  iSerialNumber;
    uint8_t  bNumberConfigurations;
};

struct usb_desc_config {
    usb_desc_header header;
    uint16_t wTotalLength;
    uint8_t  bNumInterfaces;
    uint8_t  bConfigurationValue;
    uint8_t  iConfiguration;
    uint8_t  bmAttributes;
    uint8_t  bMaxPower;
};

struct usb_desc_itf {
    usb_desc_header header;
    uint8_t bInterfaceNumber;
    uint8_t bAlternateSetting;
    uint8_t bNumEndpoints;
    uint8_t bInterfaceClass;
    uint8_t bInterfaceSubClass;
    uint8_t bInterfaceProtocol;
    uint8_t iInterface;
};

struct usb_desc_ep {
    usb_desc_header header;
    uint8_t  bEndpointAddress;
    uint8_t  bmAttributes;
    uint16_t wMaxPacketSize;
    uint8_t  bInterval;
};

struct usb_desc_LANGID {
    usb_desc_header header;
    uint16_t wLANGID;
};

struct usb_desc_str {
    usb_desc_header header;
    uint16_t unicode_string[128];
};

#pragma pack(pop)

#define USB_STRING_LEN(unicode_chars) (sizeof(usb_desc_header) + ((unicode_chars) << 1))

/**
 * @brief Modern C++23 compile-time USB UTF-16LE String Descriptor.
 * Enforces 4-byte alignment to prevent RV32IMAC bus traps and optimize DWC2 FIFO transfers.
 */
template <size_t N>
struct alignas(4) StringDescriptor {
    usb_desc_header header;
    char16_t        unicode_string[N];

    [[nodiscard]] constexpr const uint8_t* data() const noexcept {
        return reinterpret_cast<const uint8_t*>(this);
    }
    [[nodiscard]] constexpr size_t size() const noexcept {
        return header.bLength;
    }
};

/**
 * @brief Consteval helper to generate UTF-16LE string descriptors directly from ASCII string literals.
 * Eliminates manual character counting and comma-separated unicode character arrays.
 */
template <size_t N>
consteval auto make_string_descriptor(const char (&str)[N]) noexcept {
    StringDescriptor<N - 1> desc{};
    desc.header.bLength = static_cast<uint8_t>(sizeof(usb_desc_header) + (N - 1) * sizeof(char16_t));
    desc.header.bDescriptorType = USB_DESCTYPE_STR;
    for (size_t i = 0; i < N - 1; ++i) {
        desc.unicode_string[i] = static_cast<char16_t>(str[i]);
    }
    return desc;
}

/**
 * @brief Constexpr helper to generate standard Language ID string descriptors.
 */
constexpr usb_desc_LANGID make_language_id_descriptor(uint16_t lang_id = 0x0409U) noexcept {
    return usb_desc_LANGID{
        .header = {
            .bLength = sizeof(usb_desc_LANGID),
            .bDescriptorType = USB_DESCTYPE_STR
        },
        .wLANGID = lang_id
    };
}
