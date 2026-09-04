#pragma once

#include "hal/gpio.hpp"
#include "hal/time.hpp"

namespace bsp {

using namespace hal::gpio;

namespace board {

// --- Onboard RGB LEDs ---
// Red LED is on PC13, active-low
using LedRed   = OutputDevice<GpioPin<Port::C, 13>, ActiveLevel::Low>;
// Green LED is on PA1, active-low (anode to 3.3V)
using LedGreen = OutputDevice<GpioPin<Port::A, 1>, ActiveLevel::Low>;
// Blue LED is on PA2, active-low (anode to 3.3V)
using LedBlue  = OutputDevice<GpioPin<Port::A, 2>, ActiveLevel::Low>;

// --- Onboard Buttons ---
// Boot0/Key button on PA8 (active-low with internal pull-up)
using KeyButton = InputDevice<GpioPin<Port::A, 8>, ActiveLevel::Low>;

// --- Onboard ST7735 LCD SPI Pins (SPI0) ---
using LcdCs   = GpioPin<Port::B, 2>;
using LcdDc   = GpioPin<Port::B, 0>;
using LcdRst  = GpioPin<Port::B, 1>;
using LcdSck  = GpioPin<Port::A, 5>;
using LcdMosi = GpioPin<Port::A, 7>;
using LcdMiso = GpioPin<Port::A, 6>;

// --- Debug UART0 Pins ---
using Uart0Tx = GpioPin<Port::A, 9>;
using Uart0Rx = GpioPin<Port::A, 10>;

// --- I2C0 Sensor Pins ---
using I2c0Scl = GpioPin<Port::B, 6>;
using I2c0Sda = GpioPin<Port::B, 7>;

/**
 * @brief Initialize basic board peripherals (LEDs off, GPIO clocks).
 */
inline void init() noexcept {
    LedRed::init();
    LedGreen::init();
    LedBlue::init();
}

/**
 * @brief Turn off all RGB LEDs.
 */
inline void all_leds_off() noexcept {
    LedRed::off();
    LedGreen::off();
    LedBlue::off();
}

} // namespace board
} // namespace bsp
