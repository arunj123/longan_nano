#include <cstdint>
#include <cstdio>
#include <array>

#include "lcd.h"
#include "hal/time.hpp"
#include "hal/uart.hpp"
#include "bsp/board.hpp"

// ------------------------------------------------------------------------
// Modern ST7735 LCD Test Application
// Uses unified C++23 font library from lib/gd32v_lcd
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// Main Test Application
// ------------------------------------------------------------------------
int main() {
    // 1. Initialize Board Peripherals
    bsp::board::init();
    bsp::board::all_leds_off();
    bsp::board::LedGreen::on();

    hal::time::delay_ms(100);

    printf(
        "\r\n"
        "==================================================\r\n"
        "Longan Nano -- Modern C++23 ST7735 LCD Test\r\n"
        "CPU: GD32VF103 RV32IMAC @ 108 MHz\r\n"
        "Standard: C++23 (-std=gnu++23, -Os, -flto)\r\n"
        "LCD Controller: ST7735 (160x80) via SPI0 (DMA0_CH2)\r\n"
        "Debug UART0: 115200 baud (PA9 TX, PA10 RX)\r\n"
        "==================================================\r\n"
        "\r\n"
    );

    // 2. Initialize ST7735 LCD via modern C++23 HAL
    lcd_init();

    // 3. Draw Initial Test Pattern
    lcd_clear(lcd::color::DarkNavy);

    // Border frame
    lcd_rect(0, 0, LCD_WIDTH, LCD_HEIGHT, lcd::color::Cyan);

    // Header bar
    lcd_fill_rect(1, 1, LCD_WIDTH - 2, 13, lcd::color::Gray);
    lcd::draw_string(8, 4, "LONGAN NANO ST7735", lcd::color::Yellow, lcd::color::Gray);

    // Subtitle
    lcd::draw_string(6, 20, "RV32IMAC @ 108MHz", lcd::color::White, lcd::color::DarkNavy);
    lcd::draw_string(6, 32, "Modern C++23 HAL", lcd::color::Green, lcd::color::DarkNavy);

    // Color Swatches at bottom (8 colors)
    constexpr std::array<uint16_t, 8> swatches = {
        lcd::color::Red, lcd::color::Green, lcd::color::Blue, lcd::color::Yellow,
        lcd::color::Cyan, lcd::color::Magenta, lcd::color::White, lcd::color::Gray
    };
    constexpr int swatch_w = 18;
    constexpr int swatch_h = 10;
    constexpr int swatch_y = 66;

    for (size_t i = 0; i < swatches.size(); ++i) {
        int swatch_x = 8 + static_cast<int>(i * (swatch_w + 1));
        lcd_fill_rect(swatch_x, swatch_y, swatch_w, swatch_h, swatches[i]);
        lcd_rect(swatch_x, swatch_y, swatch_w, swatch_h, lcd::color::Black);
    }

    uint32_t heartbeat_count = 0;
    auto last_heartbeat = hal::time::Instant::now();
    const auto kHeartbeatInterval = hal::time::Duration::from_ms(500);

    bool button_prev = false;

    printf("LCD initialized and test pattern rendered.\r\n");

    while (true) {
        const auto now = hal::time::Instant::now();

        // Heartbeat timer (500 ms)
        if ((now - last_heartbeat) >= kHeartbeatInterval) {
            last_heartbeat = now;
            heartbeat_count++;

            // Toggle Green LED
            bsp::board::LedGreen::toggle();

            // Update live counter on LCD
            char buf[32];
            snprintf(buf, sizeof(buf), "Tick: %05lu s", heartbeat_count / 2);
            lcd::draw_string(6, 48, buf, lcd::color::Cyan, lcd::color::DarkNavy);

            // Log heartbeat to UART0
            printf("[LCD TEST #%04lu] Running. System ticks: %lu\r\n",
                   heartbeat_count, static_cast<uint32_t>(now.ticks));
        }

        // Check user button (PA8)
        bool button_pressed = bsp::board::KeyButton::is_active();
        if (button_pressed && !button_prev) {
            printf(">>> User Button Pressed! Highlight center box! <<<\r\n");
            bsp::board::LedRed::toggle();

            // Visual feedback: highlight center box
            lcd_fill_rect(20, 20, 120, 40, lcd::color::Blue);
            lcd::draw_string(26, 32, "BUTTON PRESSED!", lcd::color::Yellow, lcd::color::Blue);
            lcd::draw_string(30, 44, "DMA Transfer OK", lcd::color::White, lcd::color::Blue);

            hal::time::delay_ms(400);

            // Restore screen
            lcd_fill_rect(20, 20, 120, 40, lcd::color::DarkNavy);
            lcd::draw_string(6, 20, "RV32IMAC @ 108MHz", lcd::color::White, lcd::color::DarkNavy);
            lcd::draw_string(6, 32, "Modern C++23 HAL", lcd::color::Green, lcd::color::DarkNavy);
        }
        button_prev = button_pressed;

        hal::time::delay_ms(10);
    }
}
