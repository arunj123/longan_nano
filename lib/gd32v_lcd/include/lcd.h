#ifndef __LCD_H__
#define __LCD_H__

// ------------------------------------------------------------------------

#define LCD_WIDTH               160
#define LCD_HEIGHT              80
#define LCD_FRAMEBUFFER_PIXELS  (LCD_WIDTH * LCD_HEIGHT)
#define LCD_FRAMEBUFFER_BYTES   (LCD_WIDTH * LCD_HEIGHT * 2)

// ------------------------------------------------------------------------
// Public API for the LCD Driver
// ------------------------------------------------------------------------

/**
 * @brief Initializes the GPIO, SPI, and DMA for the LCD and sends the
 *        initialization sequence to the display controller.
 */
void lcd_init(void);

/**
 * @brief Fills the entire screen with a single color. Asynchronous.
 * @param color The 16-bit RGB565 color to use.
 */
void lcd_clear(unsigned short int color);

/**
 * @brief Draws a single pixel at the specified coordinates. Asynchronous.
 * @param x The x-coordinate.
 * @param y The y-coordinate.
 * @param color The 16-bit RGB565 color of the pixel.
 */
void lcd_setpixel(int x, int y, unsigned short int color);

/**
 * @brief Fills a rectangular area with a single color. Asynchronous.
 * @param x The starting x-coordinate.
 * @param y The starting y-coordinate.
 * @param w The width of the rectangle.
 * @param h The height of the rectangle.
 * @param color The 16-bit RGB565 color to fill with.
 */
void lcd_fill_rect(int x, int y, int w, int h, unsigned short int color);

/**
 * @brief Draws a rectangular outline. Asynchronous.
 * @param x The starting x-coordinate.
 * @param y The starting y-coordinate.
 * @param w The width of the rectangle.
 * @param h The height of the rectangle.
 * @param color The 16-bit RGB565 color of the outline.
 */
void lcd_rect(int x, int y, int w, int h, unsigned short int color);

/**
 * @brief Writes a buffer of 16-bit RGB565 pixel data to a specified rectangle on the screen. Asynchronous.
 * @param x The starting x-coordinate.
 * @param y The starting y-coordinate.
 * @param w The width of the area to write.
 * @param h The height of the area to write.
 * @param buffer A pointer to the pixel data. Buffer size must be w * h * 2 bytes.
 */
void lcd_write_u16(int x, int y, int w, int h, const void* buffer);

/**
 * @brief Waits for any pending DMA operation to complete. This is a blocking call.
 *        It should be called before the CPU accesses a buffer that was just used
 *        in a DMA operation to avoid race conditions.
 */
void lcd_wait(void);

/**
 * @brief Checks if a DMA transfer is currently in progress.
 * @return 1 if busy, 0 if idle.
 */
int lcd_is_dma_busy(void);

// ------------------------------------------------------------------------
// Framebuffer functions.
// ------------------------------------------------------------------------

/**
 * @brief Sets the memory address of the framebuffer for auto-refresh mode.
 *        This does not enable the refresh; it only configures the source address.
 * @param buffer Pointer to the framebuffer. Size must be LCD_FRAMEBUFFER_BYTES.
 */
void lcd_fb_setaddr(const void* buffer);

/**
 * @brief Enables automatic, continuous refreshing of the LCD from the framebuffer
 *        set by lcd_fb_setaddr(). Uses DMA with interrupts.
 * @note When framebuffer mode is enabled, other drawing functions (lcd_clear,
 *       lcd_fill_rect, etc.) should not be used.
 */
void lcd_fb_enable(void);

/**
 * @brief Disables automatic framebuffer refreshing.
 *        This function will wait for any in-progress refresh to complete before returning.
 */
void lcd_fb_disable(void);

// ------------------------------------------------------------------------

#endif // __LCD_H__
