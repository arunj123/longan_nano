#include <cstdio>
#include <cstdint>
#include "bsp/board.hpp"
#include "hal/time.hpp"
#include "hal/uart.hpp"

using bsp::board::LedRed;
using bsp::board::LedGreen;
using bsp::board::LedBlue;
using bsp::board::KeyButton;

int main() {
    // Zero-overhead board initialization: sets up LEDs and user key pin
    bsp::board::init();

    // Give UART hardware and host monitor a moment to settle
    hal::time::delay_ms(100);

    printf("\r\n==================================================\r\n");
    printf("  Sipeed Longan Nano -- Modern C++23 UART Engine  \r\n");
    printf("  CPU: GD32VF103 RV32IMAC @ 108 MHz               \r\n");
    printf("  Standard: C++23 (-std=gnu++23, -Os, -flto)      \r\n");
    printf("  USART0: 115200 baud (PA9 TX, PA10 RX)           \r\n");
    printf("==================================================\r\n\r\n");

    uint32_t counter = 0;
    bool last_button_state = false;

    while (true) {
        const auto now = hal::time::Instant::now();
        const uint32_t ms = hal::time::Duration::from_ticks(now.ticks).to_ms();

        // Cycle through RGB LEDs on each heartbeat
        switch (counter % 3) {
            case 0: LedRed::on();   LedGreen::off(); LedBlue::off(); break;
            case 1: LedRed::off();  LedGreen::on();  LedBlue::off(); break;
            case 2: LedRed::off();  LedGreen::off(); LedBlue::on();  break;
        }

        // Check user button (PA8) with active-low handling
        const bool button_pressed = KeyButton::is_active();
        if (button_pressed && !last_button_state) {
            printf(">>> User Button Pressed! [Time: %lu ms] <<<\r\n", static_cast<unsigned long>(ms));
        } else if (!button_pressed && last_button_state) {
            printf(">>> User Button Released! [Time: %lu ms] <<<\r\n", static_cast<unsigned long>(ms));
        }
        last_button_state = button_pressed;

        // Periodic heartbeat output
        printf("[Heartbeat #%04lu] Longan Nano alive! Time: %lu ms\r\n",
               static_cast<unsigned long>(counter++),
               static_cast<unsigned long>(ms));

        hal::time::delay_ms(500);
    }
}
