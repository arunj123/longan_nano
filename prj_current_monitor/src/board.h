#pragma once

#include <cstdint>

void board_led_init(void);
void board_led_on(void);
void board_led_off(void);
void board_led_toggle(void);
void board_key_init(void);
void board_key_isr(void);