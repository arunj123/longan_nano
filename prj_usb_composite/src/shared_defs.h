#pragma once

#include <cstdint>
#include "lcd.h"

// Quadrant Geometry
inline constexpr uint16_t QUADRANT_WIDTH          = LCD_WIDTH;
inline constexpr uint16_t QUADRANT_HEIGHT         = LCD_HEIGHT / 4;
inline constexpr uint32_t QUADRANT_PIXELS         = QUADRANT_WIDTH * QUADRANT_HEIGHT;
inline constexpr uint32_t QUADRANT_BYTES          = 2 * QUADRANT_PIXELS; // 2 bytes per pixel (RGB565)