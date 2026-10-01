#pragma once

#include <cstdint>

/**
 * @file usb_conf.h
 * @brief Modern C++23 USB FIFO configuration for Longan Nano Composite HID (Keyboard/Mouse/Consumer + Custom HID).
 */

namespace drivers::usb {
inline constexpr uint16_t RX_FIFO_FS_SIZE  = 128U;
inline constexpr uint16_t TX0_FIFO_FS_SIZE = 64U;
inline constexpr uint16_t TX1_FIFO_FS_SIZE = 16U;
inline constexpr uint16_t TX2_FIFO_FS_SIZE = 32U;
inline constexpr uint16_t TX3_FIFO_FS_SIZE = 64U;
}