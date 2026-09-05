/*!
    \file    board.cpp
    \brief   Implementation of board-specific functions (Longan Nano)
*/

#include "board.h"
#include "bsp/board.hpp"
#include "hal/exti.hpp"
#include "hal/eclic.hpp"
#include "hal/time.hpp"

volatile bool user_key_pressed = false;

void board_led_init(void) {
    bsp::board::LedRed::init();
    bsp::board::LedGreen::init();
    bsp::board::LedBlue::init();
    bsp::board::LedRed::off();
    bsp::board::LedGreen::off();
    bsp::board::LedBlue::off();
}

void board_led_on(void) {
    bsp::board::LedGreen::on();
}

void board_led_off(void) {
    bsp::board::LedGreen::off();
}

void board_led_toggle(void) {
    bsp::board::LedGreen::toggle();
}

void board_key_init(void) {
    bsp::board::KeyButton::init(); // PA8 with pull-up

    hal::exti::Exti::map_pin(hal::gpio::Port::A, 8);
    hal::exti::Exti::enable_line(8, hal::exti::Trigger::Falling);

    hal::eclic::Eclic::enable(hal::eclic::Irq::Exti5_9, 1, 0);
}

void board_key_isr(void) {
    // Perform software debouncing using typed timer utilities
    static hal::time::Instant last_key_press_time{0};
    const auto debounce_duration = hal::time::Duration::from_ms(50);
    const auto now = hal::time::Instant::now();

    if ((now - last_key_press_time) > debounce_duration) {
        last_key_press_time = now;
        user_key_pressed = true;
    }

    hal::exti::Exti::clear_pending(8);
}