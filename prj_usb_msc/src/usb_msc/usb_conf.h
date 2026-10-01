#pragma once

#include <cstdint>

/**
 * @file usb_conf.h
 * @brief Modern C++23 USB FIFO configuration for Longan Nano Mass Storage Class (MSC).
 *
 * Total available FIFO size for GD32VF103 is 1.25 KB (1280 bytes = 320 words).
 * RX FIFO: shared for all OUT endpoints (EP0 OUT, EP1 MSC OUT).
 * TX FIFOs: dedicated per IN endpoint.
 *
 * RX FIFO: 96 words (384 bytes = 6x 64B packets, ample for OUT data packets + status)
 * TX0 FIFO (EP0 IN): 32 words (128 bytes = 2x 64B EP0 MPS, double-buffered)
 * TX1 FIFO (EP1 MSC Bulk IN): 192 words (768 bytes = 512B sector + 256B headroom, eliminates FIFO wrap/drop)
 * TX2 FIFO: 0 words (Unused)
 * TX3 FIFO: 0 words (Unused)
 * Total: 96 + 32 + 192 = 320 words (100% exact hardware fit)
 */

namespace drivers::usb {
inline constexpr uint16_t RX_FIFO_FS_SIZE  = 96U;
inline constexpr uint16_t TX0_FIFO_FS_SIZE = 32U;
inline constexpr uint16_t TX1_FIFO_FS_SIZE = 192U;
inline constexpr uint16_t TX2_FIFO_FS_SIZE = 0U;
inline constexpr uint16_t TX3_FIFO_FS_SIZE = 0U;
}
