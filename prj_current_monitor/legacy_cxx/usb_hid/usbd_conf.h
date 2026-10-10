#pragma once

#include "usb_conf.h"
#include "drivers/usb/drv_usb_core.h"

inline constexpr uint8_t  USBD_CFG_MAX_NUM        = 1U;
inline constexpr uint8_t  USBD_ITF_MAX_NUM        = 1U;

inline constexpr uint16_t USB_STR_DESC_MAX_SIZE   = 64U;
inline constexpr uint32_t USB_STRING_COUNT        = 4U;

/* Custom HID interface and endpoints */
inline constexpr uint8_t  CUSTOM_HID_INTERFACE    = 0x00U;
inline constexpr uint8_t  CUSTOM_HID_IN_EP        = ep_in(1U);
inline constexpr uint8_t  CUSTOM_HID_OUT_EP       = ep_out(1U);
inline constexpr uint16_t CUSTOM_HID_IN_PACKET    = 64U;
inline constexpr uint16_t CUSTOM_HID_OUT_PACKET   = 64U;