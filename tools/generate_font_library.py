#!/usr/bin/env python3
"""
Generates modern C++23 font library headers for Longan Nano ST7735 LCD:
- lib/gd32v_lcd/include/lcd_color.hpp
- lib/gd32v_lcd/include/lcd_font.hpp
- lib/gd32v_lcd/include/lcd_draw.hpp
"""

import os
import sys
import re
from PIL import Image, ImageFont, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
LCD_INC = os.path.join(ROOT, "lib", "gd32v_lcd", "include")
os.makedirs(LCD_INC, exist_ok=True)

# 1. Generate lcd_color.hpp
color_hpp = """#pragma once
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

// Global namespace alias for backwards compatibility across existing applications
namespace color = lcd::color;
"""

with open(os.path.join(LCD_INC, "lcd_color.hpp"), "w", encoding="utf-8") as f:
    f.write(color_hpp)
print("Generated lcd_color.hpp")

# 2. Font5x7 (475 bytes)
vals_5x7 = [0, 0, 0, 0, 0, 0, 0, 95, 0, 0, 0, 7, 0, 7, 0, 20, 127, 20, 127, 20, 36, 42, 127, 42, 18, 35, 19, 8, 100, 98, 54, 73, 85, 34, 80, 0, 5, 3, 0, 0, 0, 28, 34, 65, 0, 0, 65, 34, 28, 0, 20, 8, 62, 8, 20, 8, 8, 62, 8, 8, 0, 80, 48, 0, 0, 8, 8, 8, 8, 8, 0, 96, 96, 0, 0, 32, 16, 8, 4, 2, 62, 81, 73, 69, 62, 0, 66, 127, 64, 0, 66, 97, 81, 73, 70, 33, 65, 69, 75, 49, 24, 20, 18, 127, 16, 39, 69, 69, 69, 57, 60, 74, 73, 73, 48, 1, 113, 9, 5, 3, 54, 73, 73, 73, 54, 6, 73, 73, 41, 30, 0, 54, 54, 0, 0, 0, 86, 54, 0, 0, 8, 20, 34, 65, 0, 20, 20, 20, 20, 20, 0, 65, 34, 20, 8, 2, 1, 81, 9, 6, 50, 73, 121, 65, 62, 126, 17, 17, 17, 126, 127, 73, 73, 73, 54, 62, 65, 65, 65, 34, 127, 65, 65, 34, 28, 127, 73, 73, 73, 65, 127, 9, 9, 9, 1, 62, 65, 73, 73, 122, 127, 8, 8, 8, 127, 0, 65, 127, 65, 0, 32, 64, 65, 63, 1, 127, 8, 20, 34, 65, 127, 64, 64, 64, 64, 127, 2, 12, 2, 127, 127, 4, 8, 16, 127, 62, 65, 65, 65, 62, 127, 9, 9, 9, 6, 62, 65, 81, 33, 94, 127, 9, 25, 41, 70, 70, 73, 73, 73, 49, 1, 1, 127, 1, 1, 63, 64, 64, 64, 63, 31, 32, 64, 32, 31, 63, 64, 56, 64, 63, 99, 20, 8, 20, 99, 7, 8, 112, 8, 7, 97, 81, 73, 69, 67, 0, 127, 65, 65, 0, 2, 4, 8, 16, 32, 0, 65, 65, 127, 0, 4, 2, 1, 2, 4, 64, 64, 64, 64, 64, 0, 1, 2, 4, 0, 32, 84, 84, 84, 120, 127, 72, 68, 68, 56, 56, 68, 68, 68, 32, 56, 68, 68, 72, 127, 56, 84, 84, 84, 24, 8, 126, 9, 1, 2, 12, 82, 82, 82, 62, 127, 8, 4, 4, 120, 0, 68, 125, 64, 0, 32, 64, 68, 61, 0, 127, 16, 40, 68, 0, 0, 65, 127, 64, 0, 124, 4, 24, 4, 120, 124, 8, 4, 4, 120, 56, 68, 68, 68, 56, 124, 20, 20, 20, 8, 8, 20, 20, 24, 124, 124, 8, 4, 4, 8, 72, 84, 84, 84, 32, 4, 63, 68, 64, 32, 60, 64, 64, 32, 124, 28, 32, 64, 32, 28, 60, 64, 48, 64, 60, 68, 40, 16, 40, 68, 12, 80, 80, 80, 60, 68, 100, 84, 76, 68, 0, 8, 54, 65, 0, 0, 0, 127, 0, 0, 0, 65, 54, 8, 0, 16, 8, 8, 16, 8]
assert len(vals_5x7) == 95 * 5

# 3. Font8x16 (1520 bytes)
vals_8x16 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 0, 0, 24, 24, 0, 0, 0, 72, 108, 36, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 36, 36, 127, 18, 18, 18, 127, 18, 18, 18, 0, 0, 0, 0, 8, 28, 42, 42, 10, 12, 24, 40, 40, 42, 42, 28, 8, 8, 0, 0, 0, 34, 37, 21, 21, 21, 42, 88, 84, 84, 84, 34, 0, 0, 0, 0, 0, 12, 18, 18, 18, 10, 118, 37, 41, 17, 145, 110, 0, 0, 0, 6, 6, 4, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 32, 16, 16, 8, 8, 8, 8, 8, 8, 16, 16, 32, 64, 0, 0, 2, 4, 8, 8, 16, 16, 16, 16, 16, 16, 8, 8, 4, 2, 0, 0, 0, 0, 0, 8, 8, 107, 28, 28, 107, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 127, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 6, 4, 3, 0, 0, 0, 0, 0, 0, 0, 0, 254, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 6, 0, 0, 0, 0, 128, 64, 64, 32, 32, 16, 16, 8, 8, 4, 4, 2, 2, 0, 0, 0, 0, 24, 36, 66, 66, 66, 66, 66, 66, 66, 36, 24, 0, 0, 0, 0, 0, 8, 14, 8, 8, 8, 8, 8, 8, 8, 8, 62, 0, 0, 0, 0, 0, 60, 66, 66, 66, 32, 32, 16, 8, 4, 66, 126, 0, 0, 0, 0, 0, 60, 66, 66, 32, 24, 32, 64, 64, 66, 34, 28, 0, 0, 0, 0, 0, 32, 48, 40, 36, 36, 34, 34, 126, 32, 32, 120, 0, 0, 0, 0, 0, 126, 2, 2, 2, 26, 38, 64, 64, 66, 34, 28, 0, 0, 0, 0, 0, 56, 36, 2, 2, 26, 38, 66, 66, 66, 36, 24, 0, 0, 0, 0, 0, 126, 34, 34, 16, 16, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 60, 66, 66, 66, 36, 24, 36, 66, 66, 66, 60, 0, 0, 0, 0, 0, 24, 36, 66, 66, 66, 100, 88, 64, 64, 36, 28, 0, 0, 0, 0, 0, 0, 0, 0, 24, 24, 0, 0, 0, 0, 24, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 8, 8, 4, 0, 0, 0, 64, 32, 16, 8, 4, 2, 4, 8, 16, 32, 64, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4, 8, 16, 32, 64, 32, 16, 8, 4, 2, 0, 0, 0, 0, 0, 60, 66, 66, 70, 64, 32, 16, 16, 0, 24, 24, 0, 0, 0, 0, 0, 28, 34, 90, 85, 85, 85, 85, 45, 66, 34, 28, 0, 0, 0, 0, 0, 8, 8, 24, 20, 20, 36, 60, 34, 66, 66, 231, 0, 0, 0, 0, 0, 31, 34, 34, 34, 30, 34, 66, 66, 66, 34, 31, 0, 0, 0, 0, 0, 124, 66, 66, 1, 1, 1, 1, 1, 66, 34, 28, 0, 0, 0, 0, 0, 31, 34, 66, 66, 66, 66, 66, 66, 66, 34, 31, 0, 0, 0, 0, 0, 63, 66, 18, 18, 30, 18, 18, 2, 66, 66, 63, 0, 0, 0, 0, 0, 63, 66, 18, 18, 30, 18, 18, 2, 2, 2, 7, 0, 0, 0, 0, 0, 60, 34, 34, 1, 1, 1, 113, 33, 34, 34, 28, 0, 0, 0, 0, 0, 231, 66, 66, 66, 66, 126, 66, 66, 66, 66, 231, 0, 0, 0, 0, 0, 62, 8, 8, 8, 8, 8, 8, 8, 8, 8, 62, 0, 0, 0, 0, 0, 124, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 17, 15, 0, 0, 0, 119, 34, 18, 10, 14, 10, 18, 18, 34, 34, 119, 0, 0, 0, 0, 0, 7, 2, 2, 2, 2, 2, 2, 2, 2, 66, 127, 0, 0, 0, 0, 0, 119, 54, 54, 54, 54, 42, 42, 42, 42, 42, 107, 0, 0, 0, 0, 0, 227, 70, 70, 74, 74, 82, 82, 82, 98, 98, 71, 0, 0, 0, 0, 0, 28, 34, 65, 65, 65, 65, 65, 65, 65, 34, 28, 0, 0, 0, 0, 0, 63, 66, 66, 66, 66, 62, 2, 2, 2, 2, 7, 0, 0, 0, 0, 0, 28, 34, 65, 65, 65, 65, 65, 77, 83, 50, 28, 96, 0, 0, 0, 0, 63, 66, 66, 66, 62, 18, 18, 34, 34, 66, 199, 0, 0, 0, 0, 0, 124, 66, 66, 2, 4, 24, 32, 64, 66, 66, 62, 0, 0, 0, 0, 0, 127, 73, 8, 8, 8, 8, 8, 8, 8, 8, 28, 0, 0, 0, 0, 0, 231, 66, 66, 66, 66, 66, 66, 66, 66, 66, 60, 0, 0, 0, 0, 0, 231, 66, 66, 34, 36, 36, 20, 20, 24, 8, 8, 0, 0, 0, 0, 0, 107, 73, 73, 73, 73, 85, 85, 54, 34, 34, 34, 0, 0, 0, 0, 0, 231, 66, 36, 36, 24, 24, 24, 36, 36, 66, 231, 0, 0, 0, 0, 0, 119, 34, 34, 20, 20, 8, 8, 8, 8, 8, 28, 0, 0, 0, 0, 0, 126, 33, 32, 16, 16, 8, 4, 4, 66, 66, 63, 0, 0, 0, 120, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 120, 0, 0, 0, 2, 2, 4, 4, 8, 8, 8, 16, 16, 32, 32, 32, 64, 64, 0, 30, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 30, 0, 0, 56, 68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 6, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 66, 120, 68, 66, 66, 252, 0, 0, 0, 0, 0, 3, 2, 2, 2, 26, 38, 66, 66, 66, 38, 26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 68, 2, 2, 2, 68, 56, 0, 0, 0, 0, 0, 96, 64, 64, 64, 120, 68, 66, 66, 66, 100, 216, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 66, 126, 2, 2, 66, 60, 0, 0, 0, 0, 0, 240, 136, 8, 8, 126, 8, 8, 8, 8, 8, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124, 34, 34, 28, 2, 60, 66, 66, 60, 0, 0, 0, 3, 2, 2, 2, 58, 70, 66, 66, 66, 66, 231, 0, 0, 0, 0, 0, 12, 12, 0, 0, 14, 8, 8, 8, 8, 8, 62, 0, 0, 0, 0, 0, 48, 48, 0, 0, 56, 32, 32, 32, 32, 32, 32, 34, 30, 0, 0, 0, 3, 2, 2, 2, 114, 18, 10, 22, 18, 34, 119, 0, 0, 0, 0, 0, 14, 8, 8, 8, 8, 8, 8, 8, 8, 8, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 146, 146, 146, 146, 146, 183, 0, 0, 0, 0, 0, 0, 0, 0, 0, 59, 70, 66, 66, 66, 66, 231, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 66, 66, 66, 66, 66, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 38, 66, 66, 66, 34, 30, 2, 7, 0, 0, 0, 0, 0, 0, 0, 120, 68, 66, 66, 66, 68, 120, 64, 224, 0, 0, 0, 0, 0, 0, 0, 119, 76, 4, 4, 4, 4, 31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 124, 66, 2, 60, 64, 66, 62, 0, 0, 0, 0, 0, 0, 0, 8, 8, 62, 8, 8, 8, 8, 8, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 99, 66, 66, 66, 66, 98, 220, 0, 0, 0, 0, 0, 0, 0, 0, 0, 231, 66, 36, 36, 20, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 235, 73, 73, 85, 85, 34, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 118, 36, 24, 24, 24, 36, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 231, 66, 36, 36, 20, 24, 8, 8, 7, 0, 0, 0, 0, 0, 0, 0, 126, 34, 16, 8, 8, 68, 126, 0, 0, 0, 192, 32, 32, 32, 32, 32, 16, 32, 32, 32, 32, 32, 32, 192, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 0, 6, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 6, 0, 12, 50, 194, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
assert len(vals_8x16) == 95 * 16

# 4. Generate Font16x24
font_tt_path = os.path.join(ROOT, "tools", "display_manager", "fonts", "FreeSansBold.ttf")
font_tt = ImageFont.truetype(font_tt_path, 20)
deg_font = ImageFont.truetype(font_tt_path, 12)
chars_16x24 = " %*+-./0123456789:ACFVWm" # 24 characters
font16x24_bytes = []

for ch in chars_16x24:
    im = Image.new("1", (16, 24), 0)
    draw = ImageDraw.Draw(im)
    if ch == '*': # Degree symbol
        bbox = draw.textbbox((0, 0), "o", font=deg_font)
        w = bbox[2] - bbox[0]
        x = (16 - w) // 2 - bbox[0]
        y = 2 - bbox[1]
        draw.text((x, y), "o", font=deg_font, fill=1)
    elif ch == ' ':
        pass
    else:
        bbox = draw.textbbox((0, 0), ch, font=font_tt)
        w = bbox[2] - bbox[0]
        h = bbox[3] - bbox[1]
        x = max(0, (16 - w) // 2 - bbox[0])
        y = max(0, (24 - h) // 2 - bbox[1])
        if ch == '.':
            y = 17
            x = 5
        elif ch == '-':
            y = 10
        draw.text((x, y), ch, font=font_tt, fill=1)
    
    for r in range(24):
        b0 = 0
        b1 = 0
        for col in range(8):
            if im.getpixel((col, r)):
                b0 |= (1 << (7 - col))
        for col in range(8, 16):
            if im.getpixel((col, r)):
                b1 |= (1 << (15 - col))
        font16x24_bytes.append(b0)
        font16x24_bytes.append(b1)

assert len(font16x24_bytes) == len(chars_16x24) * 48

def format_hex_lines(byte_list, per_line=16, indent="        "):
    lines = []
    for i in range(0, len(byte_list), per_line):
        chunk = byte_list[i:i+per_line]
        lines.append(indent + ", ".join(f"0x{b:02X}" for b in chunk) + ",")
    return "\n".join(lines)

font5x7_formatted = format_hex_lines(vals_5x7, 15)
font8x16_formatted = format_hex_lines(vals_8x16, 16)
font16x24_formatted = format_hex_lines(font16x24_bytes, 16)

# Construct lcd_font.hpp
font_hpp = f"""#pragma once
#include <cstdint>
#include <cstddef>
#include <array>
#include <string_view>

namespace lcd {{
namespace font {{

enum class GlyphLayout : uint8_t {{
    ColumnMajorLsb, // 1 byte per column (bit 0 = top row). Used by Font5x7.
    RowMajorLsb,    // 1 byte per row (bit 0 = left col). Used by Font8x16.
    RowMajorMsb,    // 1 byte per row (bit 7 = left col). Used by Font16x24.
}};

// Structural type for string literal NTTP (C++20/C++23)
template <size_t N>
struct FixedString {{
    char buf[N]{{}};
    constexpr FixedString(const char (&str)[N]) {{
        for (size_t i = 0; i < N; ++i) buf[i] = str[i];
    }}
    constexpr size_t size() const noexcept {{ return N > 0 ? N - 1 : 0; }}
    constexpr char operator[](size_t i) const noexcept {{ return buf[i]; }}
}};

// Compile-time helpers to collect unique sorted characters from a string
template <FixedString Str>
consteval size_t count_unique_chars() {{
    size_t count = 0;
    for (size_t i = 0; i < Str.size(); ++i) {{
        char c = Str[i];
        bool seen = false;
        for (size_t j = 0; j < i; ++j) {{
            if (Str[j] == c) {{ seen = true; break; }}
        }}
        if (!seen) ++count;
    }}
    return count;
}}

template <FixedString Str>
consteval auto get_unique_chars() {{
    constexpr size_t K = count_unique_chars<Str>();
    std::array<char, K> result{{}};
    size_t idx = 0;
    for (size_t i = 0; i < Str.size(); ++i) {{
        char c = Str[i];
        bool seen = false;
        for (size_t j = 0; j < i; ++j) {{
            if (Str[j] == c) {{ seen = true; break; }}
        }}
        if (!seen) {{
            result[idx++] = c;
        }}
    }}
    for (size_t i = 0; i < K; ++i) {{
        for (size_t j = i + 1; j < K; ++j) {{
            if (result[i] > result[j]) {{
                char t = result[i]; result[i] = result[j]; result[j] = t;
            }}
        }}
    }}
    return result;
}}

// -----------------------------------------------------------------------------
// SubsetFont: Compile-Time Character Subset Generator
// -----------------------------------------------------------------------------
template <typename BaseFont, FixedString Chars>
class SubsetFont {{
public:
    static constexpr size_t char_count = count_unique_chars<Chars>();
    static constexpr uint8_t width = BaseFont::width;
    static constexpr uint8_t height = BaseFont::height;
    static constexpr uint8_t spacing = BaseFont::spacing;
    static constexpr GlyphLayout layout = BaseFont::layout;
    static constexpr size_t bytes_per_glyph = BaseFont::bytes_per_glyph;

    struct Table {{
        std::array<char, char_count> chars;
        std::array<uint8_t, char_count * bytes_per_glyph> data;
    }};

    static consteval Table build_table() {{
        Table t{{}};
        t.chars = get_unique_chars<Chars>();
        for (size_t i = 0; i < char_count; ++i) {{
            char c = t.chars[i];
            const uint8_t* g = BaseFont::glyph(c);
            for (size_t b = 0; b < bytes_per_glyph; ++b) {{
                t.data[i * bytes_per_glyph + b] = (g ? g[b] : 0);
            }}
        }}
        return t;
    }}

    static constexpr Table table = build_table();

    static constexpr const uint8_t* glyph(char c) noexcept {{
        int low = 0;
        int high = static_cast<int>(char_count) - 1;
        while (low <= high) {{
            int mid = low + (high - low) / 2;
            auto umid = static_cast<size_t>(mid);
            if (table.chars[umid] == c) {{
                return &table.data[umid * bytes_per_glyph];
            }}
            if (table.chars[umid] < c) {{
                low = mid + 1;
            }} else {{
                high = mid - 1;
            }}
        }}
        return nullptr;
    }}

    static constexpr bool is_pixel_set(const uint8_t* g, uint8_t px, uint8_t py) noexcept {{
        return BaseFont::is_pixel_set(g, px, py);
    }}
}};

// -----------------------------------------------------------------------------
// RangeFont: Contiguous Range Subset (Zero-overhead O(1) index)
// -----------------------------------------------------------------------------
template <typename BaseFont, char StartChar, char EndChar>
class RangeFont {{
    static_assert(StartChar <= EndChar);
public:
    static constexpr size_t char_count = static_cast<size_t>(EndChar - StartChar + 1);
    static constexpr uint8_t width = BaseFont::width;
    static constexpr uint8_t height = BaseFont::height;
    static constexpr uint8_t spacing = BaseFont::spacing;
    static constexpr GlyphLayout layout = BaseFont::layout;
    static constexpr size_t bytes_per_glyph = BaseFont::bytes_per_glyph;

    static consteval auto build_table() {{
        std::array<uint8_t, char_count * bytes_per_glyph> data{{}};
        for (size_t i = 0; i < char_count; ++i) {{
            char c = static_cast<char>(StartChar + i);
            const uint8_t* g = BaseFont::glyph(c);
            for (size_t b = 0; b < bytes_per_glyph; ++b) {{
                data[i * bytes_per_glyph + b] = (g ? g[b] : 0);
            }}
        }}
        return data;
    }}

    static constexpr auto table = build_table();

    static constexpr const uint8_t* glyph(char c) noexcept {{
        if (c < StartChar || c > EndChar) return nullptr;
        return &table[(c - StartChar) * bytes_per_glyph];
    }}

    static constexpr bool is_pixel_set(const uint8_t* g, uint8_t px, uint8_t py) noexcept {{
        return BaseFont::is_pixel_set(g, px, py);
    }}
}};

// -----------------------------------------------------------------------------
// Font5x7: Standard 5x7 ASCII Bitmap Font (ASCII 32..126)
// -----------------------------------------------------------------------------
struct Font5x7 {{
    static constexpr uint8_t width = 5;
    static constexpr uint8_t height = 7;
    static constexpr uint8_t spacing = 1;
    static constexpr GlyphLayout layout = GlyphLayout::ColumnMajorLsb;
    static constexpr size_t bytes_per_glyph = 5;
    static constexpr char first_char = 32;
    static constexpr char last_char = 126;
    static constexpr size_t total_chars = 95;

    static constexpr uint8_t raw_table[95 * 5] = {{
{font5x7_formatted}
    }};

    static constexpr const uint8_t* glyph(char c) noexcept {{
        if (c < first_char || c > last_char) return nullptr;
        return &raw_table[(c - first_char) * bytes_per_glyph];
    }}

    static constexpr bool is_pixel_set(const uint8_t* g, uint8_t px, uint8_t py) noexcept {{
        if (!g) return false;
        return (g[px] & (1U << py)) != 0;
    }}
}};

// -----------------------------------------------------------------------------
// Font8x16: Classic 8x16 Terminal/OLED Bitmap Font (ASCII 32..126)
// -----------------------------------------------------------------------------
struct Font8x16 {{
    static constexpr uint8_t width = 8;
    static constexpr uint8_t height = 16;
    static constexpr uint8_t spacing = 0;
    static constexpr GlyphLayout layout = GlyphLayout::RowMajorLsb;
    static constexpr size_t bytes_per_glyph = 16;
    static constexpr char first_char = 32;
    static constexpr char last_char = 126;
    static constexpr size_t total_chars = 95;

    static constexpr uint8_t raw_table[95 * 16] = {{
{font8x16_formatted}
    }};

    static constexpr const uint8_t* glyph(char c) noexcept {{
        if (c < first_char || c > last_char) return nullptr;
        return &raw_table[(c - first_char) * bytes_per_glyph];
    }}

    static constexpr bool is_pixel_set(const uint8_t* g, uint8_t px, uint8_t py) noexcept {{
        if (!g) return false;
        return (g[py] & (1U << px)) != 0;
    }}
}};

// -----------------------------------------------------------------------------
// Font16x24: Large Bold 16x24 Font for Numbers, Temperature & Sensor Units
// Supports: ' ', '%', '*', '+', '-', '.', '/', '0'..'9', ':', 'A', 'C', 'F', 'V', 'W', 'm'
// Note: '*' is rendered as a clean degree symbol ('°') for temperatures!
// -----------------------------------------------------------------------------
struct Font16x24 {{
    static constexpr uint8_t width = 16;
    static constexpr uint8_t height = 24;
    static constexpr uint8_t spacing = 1;
    static constexpr GlyphLayout layout = GlyphLayout::RowMajorMsb;
    static constexpr size_t bytes_per_glyph = 48;
    static constexpr size_t char_count = {len(chars_16x24)};

    static constexpr char char_list[char_count] = {{
        {", ".join(f"'{c}'" for c in chars_16x24)}
    }};

    static constexpr uint8_t raw_table[char_count * 48] = {{
{font16x24_formatted}
    }};

    static constexpr const uint8_t* glyph(char c) noexcept {{
        for (size_t i = 0; i < char_count; ++i) {{
            if (char_list[i] == c) return &raw_table[i * bytes_per_glyph];
        }}
        return nullptr;
    }}

    static constexpr bool is_pixel_set(const uint8_t* g, uint8_t px, uint8_t py) noexcept {{
        if (!g) return false;
        size_t byte_idx = static_cast<size_t>(py) * 2U + (static_cast<size_t>(px) / 8U);
        uint8_t bit_idx = static_cast<uint8_t>(7U - (px % 8U));
        return (g[byte_idx] & (1U << bit_idx)) != 0;
    }}
}};

// Specialized convenience aliases
template <typename BaseFont = Font5x7>
using NumericFont = RangeFont<BaseFont, '0', '9'>;

template <FixedString Chars, typename BaseFont = Font5x7>
using SubsetFontT = SubsetFont<BaseFont, Chars>;

template <typename BaseFont, FixedString Chars>
consteval auto make_subset() {{
    return SubsetFont<BaseFont, Chars>{{}};
}}

}} // namespace font
}} // namespace lcd
"""

with open(os.path.join(LCD_INC, "lcd_font.hpp"), "w", encoding="utf-8") as f:
    f.write(font_hpp)
print("Generated lcd_font.hpp")

# 5. Generate lcd_draw.hpp
draw_hpp = """#pragma once
#include <cstdint>
#include <string_view>
#include "lcd_color.hpp"
#include "lcd_font.hpp"

// Forward declarations for LCD hardware primitives
void lcd_setpixel(int x, int y, unsigned short int color);
void lcd_fill_rect(int x, int y, int w, int h, unsigned short int color);

namespace lcd {

/**
 * @brief Draws a single character using the specified font, colors, and scaling factor.
 */
template <typename Font = font::Font5x7>
inline void draw_char(int x, int y, char c, uint16_t fg, uint16_t bg = color::Black, uint8_t scale = 1, const Font& font = Font{}) {
    const uint8_t* glyph = font.glyph(c);
    if (!glyph) {
        if (bg != color::Transparent) {
            int cell_w = (Font::width + Font::spacing) * scale;
            int cell_h = Font::height * scale;
            lcd_fill_rect(x, y, cell_w, cell_h, bg);
        }
        return;
    }

    if (scale <= 1) {
        // High-performance 1:1 pixel rendering
        for (uint8_t col = 0; col < Font::width; ++col) {
            for (uint8_t row = 0; row < Font::height; ++row) {
                bool set = Font::is_pixel_set(glyph, col, row);
                if (set) {
                    lcd_setpixel(x + col, y + row, fg);
                } else if (bg != color::Transparent) {
                    lcd_setpixel(x + col, y + row, bg);
                }
            }
        }
        // Inter-character spacing column
        if (bg != color::Transparent && Font::spacing > 0) {
            for (uint8_t s = 0; s < Font::spacing; ++s) {
                for (uint8_t row = 0; row < Font::height; ++row) {
                    lcd_setpixel(x + Font::width + s, y + row, bg);
                }
            }
        }
    } else {
        // Scaled block rendering
        for (uint8_t col = 0; col < Font::width; ++col) {
            for (uint8_t row = 0; row < Font::height; ++row) {
                bool set = Font::is_pixel_set(glyph, col, row);
                int px = x + col * scale;
                int py = y + row * scale;
                if (set) {
                    lcd_fill_rect(px, py, scale, scale, fg);
                } else if (bg != color::Transparent) {
                    lcd_fill_rect(px, py, scale, scale, bg);
                }
            }
        }
        // Inter-character spacing column
        if (bg != color::Transparent && Font::spacing > 0) {
            int spacing_x = x + Font::width * scale;
            int spacing_w = Font::spacing * scale;
            int spacing_h = Font::height * scale;
            lcd_fill_rect(spacing_x, y, spacing_w, spacing_h, bg);
        }
    }
}

/**
 * @brief Draws a null-terminated string.
 */
template <typename Font = font::Font5x7>
inline void draw_string(int x, int y, const char* str, uint16_t fg, uint16_t bg = color::Black, uint8_t scale = 1, const Font& font = Font{}) {
    if (!str) return;
    int cur_x = x;
    int advance = (Font::width + Font::spacing) * scale;
    while (*str) {
        draw_char(cur_x, y, *str++, fg, bg, scale, font);
        cur_x += advance;
    }
}

/**
 * @brief Draws a string_view.
 */
template <typename Font = font::Font5x7>
inline void draw_string(int x, int y, std::string_view str, uint16_t fg, uint16_t bg = color::Black, uint8_t scale = 1, const Font& font = Font{}) {
    int cur_x = x;
    int advance = (Font::width + Font::spacing) * scale;
    for (char c : str) {
        draw_char(cur_x, y, c, fg, bg, scale, font);
        cur_x += advance;
    }
}

/**
 * @brief Compile-time static text drawer that automatically infers a SubsetFont
 * containing ONLY the unique characters present in Text.
 */
template <font::FixedString Text, typename BaseFont = font::Font5x7>
inline void draw_static_text(int x, int y, uint16_t fg, uint16_t bg = color::Black, uint8_t scale = 1) {
    using Sub = font::SubsetFont<BaseFont, Text>;
    constexpr Sub sub_font{};
    int cur_x = x;
    int advance = (Sub::width + Sub::spacing) * scale;
    for (size_t i = 0; i < Text.size(); ++i) {
        draw_char(cur_x, y, Text[i], fg, bg, scale, sub_font);
        cur_x += advance;
    }
}

/**
 * @brief Measures the pixel width of a string.
 */
template <typename Font = font::Font5x7>
inline int measure_string(const char* str, uint8_t scale = 1, const Font& = Font{}) {
    if (!str) return 0;
    int count = 0;
    while (*str++) ++count;
    return count * (Font::width + Font::spacing) * scale;
}

} // namespace lcd

// Global namespace exports for backwards compatibility across existing applications
using lcd::draw_char;
using lcd::draw_string;
"""

with open(os.path.join(LCD_INC, "lcd_draw.hpp"), "w", encoding="utf-8") as f:
    f.write(draw_hpp)
print("Generated lcd_draw.hpp")
