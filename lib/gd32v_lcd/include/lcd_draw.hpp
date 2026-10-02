#pragma once
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
