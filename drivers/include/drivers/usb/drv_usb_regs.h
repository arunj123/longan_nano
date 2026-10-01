#pragma once

#include "dwc2_regs.hpp"

#if __has_include("usb_conf.h")
#include "usb_conf.h"
#endif

/**
 * @file drv_usb_regs.h
 * @brief Modern C++23 zero-overhead register constants and bit manipulation.
 */

namespace drivers::usb {

inline constexpr uintptr_t USBFS_REG_BASE          = drivers::usb::dwc2::USBFS_BASE;
inline constexpr uint32_t  USBFS_MAX_TX_FIFOS      = 4U;
inline constexpr uint32_t  USBFS_MAX_PACKET_SIZE   = 64U;
inline constexpr uint32_t  USBFS_MAX_CHANNEL_COUNT = 8U;
inline constexpr uint32_t  USBFS_MAX_EP_COUNT      = 4U;
inline constexpr uint32_t  USBFS_MAX_FIFO_WORDLEN  = 320U;

#ifndef RX_FIFO_FS_SIZE
inline constexpr uint16_t RX_FIFO_FS_SIZE          = 128U;
#endif
#ifndef TX0_FIFO_FS_SIZE
inline constexpr uint16_t TX0_FIFO_FS_SIZE         = 64U;
#endif
#ifndef TX1_FIFO_FS_SIZE
inline constexpr uint16_t TX1_FIFO_FS_SIZE         = 128U;
#endif
#ifndef TX2_FIFO_FS_SIZE
inline constexpr uint16_t TX2_FIFO_FS_SIZE         = 32U;
#endif
#ifndef TX3_FIFO_FS_SIZE
inline constexpr uint16_t TX3_FIFO_FS_SIZE         = 32U;
#endif

inline constexpr uint32_t USB_DATA_FIFO_OFFSET     = 0x1000U;
inline constexpr uint32_t USB_DATA_FIFO_SIZE       = 0x1000U;

inline constexpr uint32_t USB_EMBEDDED_PHY         = 1U;
inline constexpr uint32_t USB_ULPI_PHY             = 2U;

inline constexpr uint32_t USB_SPEED_UNKNOWN        = 0U;
inline constexpr uint32_t USB_SPEED_LOW            = 1U;
inline constexpr uint32_t USB_SPEED_FULL           = 2U;
inline constexpr uint32_t USB_SPEED_HIGH           = 3U;

inline constexpr uint32_t USB_SPEED_INP_FULL       = 1U;

inline constexpr uint32_t EP0MPL_64                = (0U << 0);
inline constexpr uint32_t EP0MPL_32                = (1U << 0);
inline constexpr uint32_t EP0MPL_16                = (2U << 0);
inline constexpr uint32_t EP0MPL_8                 = (3U << 0);

inline constexpr uint32_t FRAME_INTERVAL_80        = (0U << 11);

inline constexpr uint32_t DPID_DATA0               = (0U << 15);
inline constexpr uint32_t DPID_DATA1               = (2U << 15);

inline constexpr uint32_t DOEP0_TLEN               = (0x7FU << 0);
inline constexpr uint32_t DOEP0_PCNT               = (1U << 19);
inline constexpr uint32_t DOEP0_STPCNT             = (3U << 29);

inline constexpr uint32_t GUSBCS_SRPCEN            = (1U << 8);
inline constexpr uint32_t GUSBCS_HNPCEN            = (1U << 9);
inline constexpr uint32_t GUSBCS_ULPIEOI           = (1U << 20);

inline constexpr uint32_t GINTF_OTGIF              = (1U << 2);
inline constexpr uint32_t GINTF_SESIF              = (1U << 30);
inline constexpr uint32_t GOTGINTF_SESEND          = (1U << 2);

inline constexpr uint32_t GINTEN_OTGIE             = (1U << 2);
inline constexpr uint32_t GINTEN_SESIE             = (1U << 30);

template <typename T = uint32_t>
constexpr T bit(size_t n) noexcept {
    return static_cast<T>(1ULL << n);
}

constexpr uint32_t bits(size_t start, size_t end) noexcept {
    return (0xFFFFFFFFUL << start) & (0xFFFFFFFFUL >> (31U - static_cast<uint32_t>(end)));
}

} // namespace drivers::usb

using namespace drivers::usb;
