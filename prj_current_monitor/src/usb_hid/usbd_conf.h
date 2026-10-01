#pragma once

#include "usb_conf.h"
#include "drivers/usb/drv_usb_core.h"

inline constexpr uint8_t  USBD_CFG_MAX_NUM        = 1U;
inline constexpr uint8_t  USBD_ITF_MAX_NUM        = 3U;

inline constexpr uint16_t USB_STR_DESC_MAX_SIZE   = 64U;
inline constexpr uint32_t USB_STRING_COUNT        = 4U;

/* Composite interface numbers */
inline constexpr uint8_t  STD_HID_INTERFACE       = 0x00U;
inline constexpr uint8_t  CUSTOM_HID_INTERFACE    = 0x01U;
inline constexpr uint8_t  MSC_INTERFACE           = 0x02U;

inline constexpr uint8_t  USBD_HID_INTERFACE      = STD_HID_INTERFACE;
inline constexpr uint8_t  USBD_MSC_INTERFACE      = MSC_INTERFACE;

/* Custom HID interface endpoints */
inline constexpr uint8_t  CUSTOM_HID_IN_EP        = ep_in(1U);
inline constexpr uint8_t  CUSTOM_HID_OUT_EP       = ep_out(1U);
inline constexpr uint16_t CUSTOM_HID_IN_PACKET    = 64U;
inline constexpr uint16_t CUSTOM_HID_OUT_PACKET   = 64U;

/* Mass Storage Class Endpoints & Configuration */
inline constexpr uint8_t  MSC_IN_EP               = ep_in(3U);
inline constexpr uint8_t  MSC_OUT_EP              = ep_out(3U);
inline constexpr uint16_t MSC_IN_PACKET           = 64U;
inline constexpr uint16_t MSC_OUT_PACKET          = 64U;

/* Standalone compatibility */
inline constexpr uint32_t MSC_MEDIA_PACKET_SIZE   = 2048U;
inline constexpr uint8_t  MEM_LUN_NUM             = 1U;
inline constexpr uint16_t MSC_DATA_PACKET_SIZE    = MSC_IN_PACKET;