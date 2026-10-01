#pragma once

#include "dwc2_regs.hpp"

#if __has_include("usb_conf.h")
#include "usb_conf.h"
#endif

/**
 * @file drv_usb_regs.h
 * @brief Compatibility header mapping legacy register symbols to modern dwc2_regs.hpp
 */

#ifndef __IO
#define __IO volatile
#endif

#define USBFS_REG_BASE                drivers::usb::dwc2::USBFS_BASE
#define USBFS_MAX_TX_FIFOS            4U
#define USBFS_MAX_PACKET_SIZE         64U
#define USBFS_MAX_CHANNEL_COUNT       8U
#define USBFS_MAX_EP_COUNT            4U
#define USBFS_MAX_FIFO_WORDLEN        320U

#ifndef RX_FIFO_FS_SIZE
#define RX_FIFO_FS_SIZE               128U
#endif
#ifndef TX0_FIFO_FS_SIZE
#define TX0_FIFO_FS_SIZE              64U
#endif
#ifndef TX1_FIFO_FS_SIZE
#define TX1_FIFO_FS_SIZE              128U
#endif
#ifndef TX2_FIFO_FS_SIZE
#define TX2_FIFO_FS_SIZE              32U
#endif
#ifndef TX3_FIFO_FS_SIZE
#define TX3_FIFO_FS_SIZE              32U
#endif

#define USB_DATA_FIFO_OFFSET          0x1000U
#define USB_DATA_FIFO_SIZE            0x1000U

#define USB_EMBEDDED_PHY              1U
#define USB_ULPI_PHY                  2U

#define USB_SPEED_UNKNOWN             0U
#define USB_SPEED_LOW                 1U
#define USB_SPEED_FULL                2U
#define USB_SPEED_HIGH                3U

#define USB_SPEED_INP_FULL            1U

#define EP0MPL_64                     (0U << 0)
#define EP0MPL_32                     (1U << 0)
#define EP0MPL_16                     (2U << 0)
#define EP0MPL_8                      (3U << 0)

#define FRAME_INTERVAL_80             (0U << 11)

#define DPID_DATA0                    (0U << 15)
#define DPID_DATA1                    (2U << 15)

#define DOEP0_TLEN                    (0x7FU << 0)
#define DOEP0_PCNT                    (1U << 19)
#define DOEP0_STPCNT                  (3U << 29)

#define GUSBCS_SRPCEN                 (1U << 8)
#define GUSBCS_HNPCEN                 (1U << 9)
#define GUSBCS_ULPIEOI                (1U << 20)

#define GINTF_OTGIF                   (1U << 2)
#define GINTF_SESIF                   (1U << 30)
#define GOTGINTF_SESEND               (1U << 2)

#define GINTEN_OTGIE                  (1U << 2)
#define GINTEN_SESIE                  (1U << 30)

#define BIT(x)                        (1U << (x))
#define BITS(start, end)              ((0xFFFFFFFFUL << (start)) & (0xFFFFFFFFUL >> (31U - (uint32_t)(end))))
