#pragma once

#include "usb_conf.h"
#include "drivers/usb/drv_usb_core.h"

inline constexpr uint8_t  USBD_CFG_MAX_NUM        = 1U;
inline constexpr uint8_t  USBD_ITF_MAX_NUM        = 2U;

inline constexpr uint16_t USB_STR_DESC_MAX_SIZE   = 64U;
inline constexpr uint32_t USB_STRING_COUNT        = 4U;

/* Composite interface numbers */
inline constexpr uint8_t  STD_HID_INTERFACE       = 0x00U;
inline constexpr uint8_t  CUSTOM_HID_INTERFACE    = 0x01U;
inline constexpr uint8_t  USBD_HID_INTERFACE      = STD_HID_INTERFACE;

/* Standard HID (Keyboard/Mouse/Consumer) Class Endpoint */
inline constexpr uint8_t  STD_HID_IN_EP           = ep_in(1U);
inline constexpr uint16_t STD_HID_IN_PACKET       = 16U;

/* Custom HID Class Endpoints */
inline constexpr uint8_t  CUSTOM_HID_IN_EP        = ep_in(2U);
inline constexpr uint8_t  CUSTOM_HID_OUT_EP       = ep_out(2U);
inline constexpr uint16_t CUSTOM_HID_IN_PACKET    = 64U;
inline constexpr uint16_t CUSTOM_HID_OUT_PACKET   = 64U;