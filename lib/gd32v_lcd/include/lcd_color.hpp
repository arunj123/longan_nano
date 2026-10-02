#pragma once
#include <cstdint>

namespace lcd {
namespace color {
    inline constexpr uint16_t Black       = 0x0000;
    inline constexpr uint16_t White       = 0xFFFF;
    inline constexpr uint16_t Red         = 0xF800;
    inline constexpr uint16_t Green       = 0x07E0;
    inline constexpr uint16_t Blue        = 0x001F;
    inline constexpr uint16_t Yellow      = 0xFFE0;
    inline constexpr uint16_t Cyan        = 0x07FF;
    inline constexpr uint16_t Magenta     = 0xF81F;
    inline constexpr uint16_t Gray        = 0x4208;
    inline constexpr uint16_t DarkGray    = 0x2104;
    inline constexpr uint16_t LightGray   = 0x8410;
    inline constexpr uint16_t DarkNavy    = 0x000B;
    inline constexpr uint16_t Orange      = 0xFD20;
    inline constexpr uint16_t Transparent = 0x0001; // Sentinel value for transparent background

    constexpr uint16_t rgb(uint8_t r, uint8_t g, uint8_t b) noexcept {
        return static_cast<uint16_t>(((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3));
    }
} // namespace color
} // namespace lcd
