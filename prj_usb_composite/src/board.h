/*!
    \file    board.h
    \brief   Header for board-specific functions (Longan Nano)

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef BOARD_H
#define BOARD_H

#include "gd32vf103.h"

// Longan Nano User Key is on PA8
#define USER_KEY_PIN              GPIO_PIN_8
#define USER_KEY_GPIO_PORT        GPIOA
#define USER_KEY_GPIO_CLK         RCU_GPIOA
#define USER_KEY_EXTI_LINE        EXTI_8
#define USER_KEY_EXTI_PORT_SOURCE GPIO_PORT_SOURCE_GPIOA
#define USER_KEY_EXTI_PIN_SOURCE  GPIO_PIN_SOURCE_8
#define USER_KEY_EXTI_IRQn        EXTI5_9_IRQn

// Longan Nano RGB LED
#define LED_R_PIN                 GPIO_PIN_13
#define LED_R_GPIO_PORT           GPIOC
#define LED_R_GPIO_CLK            RCU_GPIOC

#define LED_G_PIN                 GPIO_PIN_1
#define LED_G_GPIO_PORT           GPIOA
#define LED_G_GPIO_CLK            RCU_GPIOA

#define LED_B_PIN                 GPIO_PIN_2
#define LED_B_GPIO_PORT           GPIOA
#define LED_B_GPIO_CLK            RCU_GPIOA

void board_led_init(void);
void board_led_on(void);
void board_led_off(void);
void board_led_toggle(void);
void board_key_init(void);

// New helper function for toggling a GPIO pin
void gpio_bit_toggle(uint32_t gpio_periph, uint32_t pin);

#endif // BOARD_H