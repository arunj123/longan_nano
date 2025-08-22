#ifndef SHARED_DEFS_H
#define SHARED_DEFS_H

// LCD Geometry
#define LCD_WIDTH               160
#define LCD_HEIGHT              80
#define LCD_FRAMEBUFFER_PIXELS  (LCD_WIDTH * LCD_HEIGHT)
#define LCD_FRAMEBUFFER_BYTES   (LCD_WIDTH * LCD_HEIGHT * 2)

// New Quadrant Geometry
#define QUADRANT_WIDTH          (LCD_WIDTH)
#define QUADRANT_HEIGHT         (LCD_HEIGHT / 4)
#define QUADRANT_PIXELS         (QUADRANT_WIDTH * QUADRANT_HEIGHT)
#define QUADRANT_BYTES          (2 * QUADRANT_PIXELS) // 2 bytes per pixel (RGB565)

#endif // SHARED_DEFS_H