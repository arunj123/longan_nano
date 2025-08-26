/*!
    \file    board.cpp
    \brief   Implementation of board-specific functions (Longan Nano)

    \version 2025-02-10, firmware for GD32VF103
*/

#include "board.h"

volatile bool user_key_pressed = false; // Flag to indicate key press to the application

/*!
    \brief      efficiently toggles the specified GPIO pin
    \param[in]  gpio_periph: where x can be (A, B, C, D, E)
    \param[in]  pin: the pin to toggle
    \retval     none
*/
void gpio_bit_toggle(uint32_t gpio_periph, uint32_t pin)
{
    // The upper 16 bits of the BSHR register are for clearing,
    // but they can also be used for toggling in some contexts.
    // A more direct and standard way is to use the OCTL register.
    if (GPIO_OCTL(gpio_periph) & pin) {
        GPIO_BC(gpio_periph) = pin;
    } else {
        GPIO_BOP(gpio_periph) = pin;
    }
}

void board_led_init(void) {
    rcu_periph_clock_enable(LED_R_GPIO_CLK);
    rcu_periph_clock_enable(LED_G_GPIO_CLK);
    rcu_periph_clock_enable(LED_B_GPIO_CLK);
    
    gpio_init(LED_R_GPIO_PORT, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, LED_R_PIN);
    gpio_init(LED_G_GPIO_PORT, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, LED_G_PIN);
    gpio_init(LED_B_GPIO_PORT, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, LED_B_PIN);

    // LEDs on Longan Nano are common anode, so set high to turn off
    gpio_bit_set(LED_R_GPIO_PORT, LED_R_PIN);
    gpio_bit_set(LED_G_GPIO_PORT, LED_G_PIN);
    gpio_bit_set(LED_B_GPIO_PORT, LED_B_PIN);
}

void board_led_on(void) {
    // Turn on green LED
    gpio_bit_reset(LED_G_GPIO_PORT, LED_G_PIN);
}

void board_led_off(void) {
    gpio_bit_set(LED_G_GPIO_PORT, LED_G_PIN);
}

void board_led_toggle(void) {
    gpio_bit_toggle(LED_G_GPIO_PORT, LED_G_PIN);
}

void board_key_init(void) {
    rcu_periph_clock_enable(USER_KEY_GPIO_CLK);
    rcu_periph_clock_enable(RCU_AF);

    gpio_exti_source_select(USER_KEY_EXTI_PORT_SOURCE, USER_KEY_EXTI_PIN_SOURCE);

    exti_init(USER_KEY_EXTI_LINE, EXTI_INTERRUPT, EXTI_TRIG_FALLING);
    exti_interrupt_flag_clear(USER_KEY_EXTI_LINE);

    eclic_irq_enable(USER_KEY_EXTI_IRQn, 1, 0);
}

void board_key_isr(void) {
    // Perform software debouncing using a static timer.
    static volatile uint64_t last_key_press_time = 0;
    const uint32_t DEBOUNCE_TIME_MS = 50;
    uint64_t now = get_timer_value();

    // Only execute the action if the debounce time has passed.
    if ((now - last_key_press_time) > DEBOUNCE_TIME_MS) {
        last_key_press_time = now; // Update the timer for the *next* valid press
        user_key_pressed = true;   // Set the application flag
    }

    exti_interrupt_flag_clear(USER_KEY_EXTI_LINE);
}