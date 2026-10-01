#pragma once

#include <cstdint>
#include "usbd_core.h"

/**
 * @file usbd_enum.h
 * @brief USB standard Chapter 9 enumeration engine.
 */

enum usb_reqsta : uint8_t {
    REQ_SUPP    = 0x0U,
    REQ_NOTSUPP = 0x1U
};

enum _str_index {
    STR_IDX_LANGID  = 0x00U,
    STR_IDX_MFC     = 0x01U,
    STR_IDX_PRODUCT = 0x02U,
    STR_IDX_SERIAL  = 0x03U,
    STR_IDX_CONFIG  = 0x04U,
    STR_IDX_ITF     = 0x05U,
    STR_IDX_MAX     = 0x0AU
};

enum _usb_pwrsta {
    USB_PWRSTA_SELF_POWERED  = 0x1U,
    USB_PWRSTA_REMOTE_WAKEUP = 0x2U
};

enum _usb_feature {
    USB_FEATURE_EP_HALT       = 0x0U,
    USB_FEATURE_REMOTE_WAKEUP = 0x1U,
    USB_FEATURE_TEST_MODE     = 0x2U
};

inline constexpr uint16_t USBD_ENG_LANGID = 0x0409U;

constexpr bool is_ctl_ep(uint8_t ep) noexcept {
    return (0x00U == ep) || (0x80U == ep);
}

// Zero-overhead constexpr compatibility wrapper
constexpr bool CTL_EP(uint8_t ep) noexcept {
    return is_ctl_ep(ep);
}

usb_reqsta usbd_standard_request(usb_core_driver* udev, usb_req* req);
usb_reqsta usbd_class_request(usb_core_driver* udev, usb_req* req);
usb_reqsta usbd_vendor_request(usb_core_driver* udev, usb_req* req);
void usbd_enum_error(usb_core_driver* udev, usb_req* req);
void int_to_unicode(uint32_t value, uint8_t* pbuf, uint8_t len);
void serial_string_get(uint16_t* unicode_str);
