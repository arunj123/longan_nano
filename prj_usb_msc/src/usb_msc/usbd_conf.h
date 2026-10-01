#pragma once

#include "usb_conf.h"
#include "drivers/usb/drv_usb_core.h"

inline constexpr uint8_t  USBD_CFG_MAX_NUM      = 1U;
inline constexpr uint8_t  USBD_ITF_MAX_NUM      = 1U;

inline constexpr uint16_t USB_STR_DESC_MAX_SIZE = 64U;
inline constexpr uint32_t USB_STRING_COUNT      = 4U;

/* Mass Storage Class Endpoints & Settings */
inline constexpr uint8_t  MSC_INTERFACE         = 0x00U;

inline constexpr uint8_t  MSC_IN_EP             = ep_in(1U);
inline constexpr uint8_t  MSC_OUT_EP            = ep_out(1U);
inline constexpr uint16_t MSC_IN_PACKET         = 64U;
inline constexpr uint16_t MSC_OUT_PACKET        = 64U;

inline constexpr uint32_t MSC_MEDIA_PACKET_SIZE = 512U;  /* Media buffer: 512 bytes (1 sector, eliminates TT split-transaction NAK timeout) */
inline constexpr uint8_t  MEM_LUN_NUM           = 1U;    /* 1 LUN for onboard MicroSD slot */
