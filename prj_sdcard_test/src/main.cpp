#include <cstdint>
#include <cstdio>
#include <array>
#include <span>

#include "lcd.h"
#include "hal/time.hpp"
#include "hal/uart.hpp"
#include "hal/gpio.hpp"
#include "drivers/sdcard.hpp"

// ------------------------------------------------------------------------
// Standard 16-bit RGB565 Colors
// ------------------------------------------------------------------------
// ------------------------------------------------------------------------
// Modern ST7735 LCD SD Card Raw SPI Test
// Uses unified C++23 font library from lib/gd32v_lcd
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// Hardware Definitions
// ------------------------------------------------------------------------
using LedRed   = hal::gpio::GpioPin<hal::gpio::Port::C, 13>;
using LedGreen = hal::gpio::GpioPin<hal::gpio::Port::A, 1>;
using LedBlue  = hal::gpio::GpioPin<hal::gpio::Port::A, 2>;
using KeyBtn   = hal::gpio::GpioPin<hal::gpio::Port::A, 8>;

using Sd = drivers::sdcard::SdCard<>;

static void print_hexdump(const uint8_t* data, size_t len) {
    for (size_t i = 0; i < len; i += 16) {
        printf("  %04lX:  ", static_cast<unsigned long>(i));
        for (size_t j = 0; j < 16; ++j) {
            if (i + j < len) {
                printf("%02X ", data[i + j]);
            } else {
                printf("   ");
            }
            if (j == 7) printf(" ");
        }
        printf(" |");
        for (size_t j = 0; j < 16; ++j) {
            if (i + j < len) {
                char c = static_cast<char>(data[i + j]);
                printf("%c", (c >= 32 && c <= 126) ? c : '.');
            }
        }
        printf("|\n");
    }
}

static void run_sd_test() {
    printf("\n>>> STARTING SD CARD DIAGNOSTIC SEQUENCE <<<\n");

    lcd_fill_rect(0, 16, LCD_WIDTH, LCD_HEIGHT - 16, color::Black);
    draw_string(4, 18, "Probing SD card...", color::Yellow);

    LedRed::set();   // OFF
    LedGreen::set(); // OFF
    LedBlue::reset(); // ON (Blue = Testing)

    auto init_res = Sd::init(true);

    if (init_res == drivers::sdcard::SdResult::Success) {
        LedBlue::set();   // OFF
        LedGreen::reset(); // ON (Green = Success)

        draw_string(4, 18, "SD Init: SUCCESS! ", color::Green);
        
        char type_str[26];
        snprintf(type_str, sizeof(type_str), "Type: %s", 
                 (Sd::card_type == drivers::sdcard::CardType::SD2HC) ? "SDHC/SDXC" : "SDSC/MMC");
        draw_string(4, 30, type_str, color::White);

        draw_string(4, 42, "Reading Sector 0...", color::Cyan);
        printf("[TEST] Reading Sector 0 (Master Boot Record / Boot Sector)...\n");

        alignas(4) uint8_t sector_buf[512] = {0};
        auto read_res = Sd::read_sector(0, sector_buf);

        if (read_res == drivers::sdcard::SdResult::Success) {
            printf("[TEST] Sector 0 read successfully! Printing Hexdump (512 bytes):\n");
            print_hexdump(sector_buf, 512);

            uint16_t sig = (static_cast<uint16_t>(sector_buf[510]) << 8) | sector_buf[511];
            printf("\n[CHECK] Boot Record Signature: 0x%04X (Expected: 0x55AA)\n", sig);

            if (sector_buf[510] == 0x55 && sector_buf[511] == 0xAA) {
                printf("[SUCCESS] >>> 0x55AA VALID BOOT RECORD SIGNATURE CONFIRMED! <<<\n");
                draw_string(4, 42, "Sector 0: 0x55AA OK! ", color::Green);
                draw_string(4, 54, "SD Card 100% OPERATIONAL", color::Green);

                // Inspect partition 1 type
                uint8_t part_type = sector_buf[446 + 4];
                printf("          Partition 1 Type: 0x%02X\n", part_type);
            } else {
                printf("[INFO] Sector 0 read OK (Unpartitioned / Non-MBR format, Sig: 0x%04X)\n", sig);
                draw_string(4, 42, "Sector 0: Read OK    ", color::Yellow);
                draw_string(4, 54, "Card operational", color::White);
            }
        } else {
            printf("[ERROR] Sector 0 read failed: %s\n", drivers::sdcard::result_to_string(read_res));
            draw_string(4, 42, "Read Sec 0: FAILED", color::Red);
        }

    } else {
        LedBlue::set();  // OFF
        LedRed::reset(); // ON (Red = Error)

        draw_string(4, 18, "SD Init: FAILED!   ", color::Red);
        draw_string(4, 30, drivers::sdcard::result_to_string(init_res), color::Red);

        if (init_res == drivers::sdcard::SdResult::Cmd0Fail) {
            draw_string(4, 44, "Check TF card seating", color::Yellow);
            draw_string(4, 56, "Press PA8 to retry", color::White);
        }
    }
}

int main(void) {
    // 1. Initialize Debug UART0
    hal::uart::Uart0::init(115200);

    // 2. Initialize Status LEDs & Button
    LedRed::init(hal::gpio::Mode::OutputPushPull);
    LedGreen::init(hal::gpio::Mode::OutputPushPull);
    LedBlue::init(hal::gpio::Mode::OutputPushPull);
    LedRed::set();   // Active-low: High = OFF
    LedGreen::set();
    LedBlue::set();

    KeyBtn::init(hal::gpio::Mode::InputPullUp);

    // 3. Initialize ST7735 LCD
    lcd_init();
    lcd_clear(color::Black);

    // Title banner on LCD
    lcd_fill_rect(0, 0, LCD_WIDTH, 14, color::DarkNavy);
    draw_string(4, 3, "LONGAN NANO: SD TEST", color::Cyan, color::DarkNavy);

    printf("\n==================================================\n");
    printf("Longan Nano -- Modern C++23 SD Card Test\n");
    printf("CPU: GD32VF103 RV32IMAC @ %lu MHz\n", (unsigned long)(SystemCoreClock / 1000000));
    printf("Interface: SPI1 (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI)\n");
    printf("User Button: PA8 (Press to re-run test at any time)\n");
    printf("==================================================\n");

    // Run first test immediately
    run_sd_test();

    uint32_t loop_counter = 0;
    bool prev_button_pressed = false;

    while (true) {
        hal::time::delay_ms(100);
        ++loop_counter;

        // Heartbeat on Blue LED (blink every 2 seconds if not currently active)
        if (loop_counter % 20 == 0) {
            LedBlue::toggle();
            hal::time::delay_ms(20);
            LedBlue::toggle();
        }

        // Check PA8 button (Active Low)
        bool button_pressed = (KeyBtn::read() == hal::gpio::Level::Low);
        if (button_pressed && !prev_button_pressed) {
            printf("\n[USER] PA8 Button Pressed! Re-running SD Card Probe...\n");
            run_sd_test();
        }
        prev_button_pressed = button_pressed;
    }

    return 0;
}
