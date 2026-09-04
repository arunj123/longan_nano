#ifndef __FONT_H__
#define __FONT_H__

#include <stdint.h>

/**
 * @brief Draws a single character to the framebuffer.
 *
 * @param x The x-coordinate of the top-left corner of the character.
 * @param y The y-coordinate of the top-left corner of the character.
 * @param c The character to draw.
 * @param color The color of the character.
 */
void draw_char(int x, int y, char c, uint16_t color);

/**
 * @brief Draws a string to the framebuffer.
 *
 * @param x The x-coordinate of the top-left corner of the first character.
 * @param y The y-coordinate of the top-left corner of the first character.
 * @param str The string to draw.
 * @param color The color of the string.
 */
void draw_string(int x, int y, const char* str, uint16_t color);

#endif // __FONT_H__