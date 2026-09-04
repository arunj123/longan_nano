#ifndef __FONT_BIG_H__
#define __FONT_BIG_H__

#include <stdint.h>

/**
 * @brief Draws a single character to the framebuffer using the large font.
 *
 * @param x The x-coordinate of the top-left corner of the character.
 * @param y The y-coordinate of the top-left corner of the character.
 * @param c The character to draw.
 * @param color The color of the character.
 */
void draw_char_big(int x, int y, char c, uint16_t color);

/**
 * @brief Draws a string to the framebuffer using the large font.
 *
 * @param x The x-coordinate of the top-left corner of the first character.
 * @param y The y-coordinate of the top-left corner of the first character.
 * @param str The string to draw.
 * @param color The color of the string.
 */
void draw_string_big(int x, int y, const char* str, uint16_t color);

#endif // __FONT_BIG_H__