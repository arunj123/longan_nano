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
namespace color {
    constexpr uint16_t Black   = 0x0000;
    constexpr uint16_t White   = 0xFFFF;
    constexpr uint16_t Red     = 0xF800;
    constexpr uint16_t Green   = 0x07E0;
    constexpr uint16_t Blue    = 0x001F;
    constexpr uint16_t Yellow  = 0xFFE0;
    constexpr uint16_t Cyan    = 0x07FF;
    constexpr uint16_t DarkNavy= 0x000B;
}

// ------------------------------------------------------------------------
// Crisp 5x7 ASCII Bitmap Font
// ------------------------------------------------------------------------
static const uint8_t font5x7[] = {
    0x00, 0x00, 0x00, 0x00, 0x00, // 32 (space)
    0x00, 0x00, 0x5F, 0x00, 0x00, // 33 !
    0x00, 0x07, 0x00, 0x07, 0x00, // 34 "
    0x14, 0x7F, 0x14, 0x7F, 0x14, // 35 #
    0x24, 0x2A, 0x7F, 0x2A, 0x12, // 36 $
    0x23, 0x13, 0x08, 0x64, 0x62, // 37 %
    0x36, 0x49, 0x55, 0x22, 0x50, // 38 &
    0x00, 0x05, 0x03, 0x00, 0x00, // 39 '
    0x00, 0x1C, 0x22, 0x41, 0x00, // 40 (
    0x00, 0x41, 0x22, 0x1C, 0x00, // 41 )
    0x14, 0x08, 0x3E, 0x08, 0x14, // 42 *
    0x08, 0x08, 0x3E, 0x08, 0x08, // 43 +
    0x00, 0x50, 0x30, 0x00, 0x00, // 44 ,
    0x08, 0x08, 0x08, 0x08, 0x08, // 45 -
    0x00, 0x60, 0x60, 0x00, 0x00, // 46 .
    0x20, 0x10, 0x08, 0x04, 0x02, // 47 /
    0x3E, 0x51, 0x49, 0x45, 0x3E, // 48 0
    0x00, 0x42, 0x7F, 0x40, 0x00, // 49 1
    0x42, 0x61, 0x51, 0x49, 0x46, // 50 2
    0x21, 0x41, 0x45, 0x4B, 0x31, // 51 3
    0x18, 0x14, 0x12, 0x7F, 0x10, // 52 4
    0x27, 0x45, 0x45, 0x45, 0x39, // 53 5
    0x3C, 0x4A, 0x49, 0x49, 0x30, // 54 6
    0x01, 0x71, 0x09, 0x05, 0x03, // 55 7
    0x36, 0x49, 0x49, 0x49, 0x36, // 56 8
    0x06, 0x49, 0x49, 0x29, 0x1E, // 57 9
    0x00, 0x36, 0x36, 0x00, 0x00, // 58 :
    0x00, 0x56, 0x36, 0x00, 0x00, // 59 ;
    0x08, 0x14, 0x22, 0x41, 0x00, // 60 <
    0x14, 0x14, 0x14, 0x14, 0x14, // 61 =
    0x00, 0x41, 0x22, 0x14, 0x08, // 62 >
    0x02, 0x01, 0x51, 0x09, 0x06, // 63 ?
    0x32, 0x49, 0x79, 0x41, 0x3E, // 64 @
    0x7E, 0x11, 0x11, 0x11, 0x7E, // 65 A
    0x7F, 0x49, 0x49, 0x49, 0x36, // 66 B
    0x3E, 0x41, 0x41, 0x41, 0x22, // 67 C
    0x7F, 0x41, 0x41, 0x22, 0x1C, // 68 D
    0x7F, 0x49, 0x49, 0x49, 0x41, // 69 E
    0x7F, 0x09, 0x09, 0x09, 0x01, // 70 F
    0x3E, 0x41, 0x49, 0x49, 0x7A, // 71 G
    0x7F, 0x08, 0x08, 0x08, 0x7F, // 72 H
    0x00, 0x41, 0x7F, 0x41, 0x00, // 73 I
    0x20, 0x40, 0x41, 0x3F, 0x01, // 74 J
    0x7F, 0x08, 0x14, 0x22, 0x41, // 75 K
    0x7F, 0x40, 0x40, 0x40, 0x40, // 76 L
    0x7F, 0x02, 0x0C, 0x02, 0x7F, // 77 M
    0x7F, 0x04, 0x08, 0x10, 0x7F, // 78 N
    0x3E, 0x41, 0x41, 0x41, 0x3E, // 79 O
    0x7F, 0x09, 0x09, 0x09, 0x06, // 80 P
    0x3E, 0x41, 0x51, 0x21, 0x5E, // 81 Q
    0x7F, 0x09, 0x19, 0x29, 0x46, // 82 R
    0x46, 0x49, 0x49, 0x49, 0x31, // 83 S
    0x01, 0x01, 0x7F, 0x01, 0x01, // 84 T
    0x3F, 0x40, 0x40, 0x40, 0x3F, // 85 U
    0x1F, 0x20, 0x40, 0x20, 0x1F, // 86 V
    0x3F, 0x40, 0x38, 0x40, 0x3F, // 87 W
    0x63, 0x14, 0x08, 0x14, 0x63, // 88 X
    0x07, 0x08, 0x70, 0x08, 0x07, // 89 Y
    0x61, 0x51, 0x49, 0x45, 0x43, // 90 Z
    0x00, 0x7F, 0x41, 0x41, 0x00, // 91 [
    0x02, 0x04, 0x08, 0x10, 0x20, // 92 '\'
    0x00, 0x41, 0x41, 0x7F, 0x00, // 93 ]
    0x04, 0x02, 0x01, 0x02, 0x04, // 94 ^
    0x40, 0x40, 0x40, 0x40, 0x40, // 95 _
    0x00, 0x01, 0x02, 0x04, 0x00, // 96 `
    0x20, 0x54, 0x54, 0x54, 0x78, // 97 a
    0x7F, 0x48, 0x44, 0x44, 0x38, // 98 b
    0x38, 0x44, 0x44, 0x44, 0x20, // 99 c
    0x38, 0x44, 0x44, 0x48, 0x7F, // 100 d
    0x38, 0x54, 0x54, 0x54, 0x18, // 101 e
    0x08, 0x7E, 0x09, 0x01, 0x02, // 102 f
    0x0C, 0x52, 0x52, 0x52, 0x3E, // 103 g
    0x7F, 0x08, 0x04, 0x04, 0x78, // 104 h
    0x00, 0x44, 0x7D, 0x40, 0x00, // 105 i
    0x20, 0x40, 0x44, 0x3D, 0x00, // 106 j
    0x7F, 0x10, 0x28, 0x44, 0x00, // 107 k
    0x00, 0x41, 0x7F, 0x40, 0x00, // 108 l
    0x7C, 0x04, 0x18, 0x04, 0x78, // 109 m
    0x7C, 0x08, 0x04, 0x04, 0x78, // 110 n
    0x38, 0x44, 0x44, 0x44, 0x38, // 111 o
    0x7C, 0x14, 0x14, 0x14, 0x08, // 112 p
    0x08, 0x14, 0x14, 0x18, 0x7C, // 113 q
    0x7C, 0x08, 0x04, 0x04, 0x08, // 114 r
    0x48, 0x54, 0x54, 0x54, 0x20, // 115 s
    0x04, 0x3F, 0x44, 0x40, 0x20, // 116 t
    0x3C, 0x40, 0x40, 0x20, 0x7C, // 117 u
    0x1C, 0x20, 0x40, 0x20, 0x1C, // 118 v
    0x3C, 0x40, 0x30, 0x40, 0x3C, // 119 w
    0x44, 0x28, 0x10, 0x28, 0x44, // 120 x
    0x0C, 0x50, 0x50, 0x50, 0x3C, // 121 y
    0x44, 0x64, 0x54, 0x4C, 0x44, // 122 z
    0x00, 0x08, 0x36, 0x41, 0x00, // 123 {
    0x00, 0x00, 0x7F, 0x00, 0x00, // 124 |
    0x00, 0x41, 0x36, 0x08, 0x00, // 125 }
    0x10, 0x08, 0x08, 0x10, 0x08  // 126 ~
};

static void draw_char(int x, int y, char c, uint16_t fg, uint16_t bg = color::Black) {
    if (c < 32 || c > 126) return;
    const uint8_t* glyph = &font5x7[(c - 32) * 5];

    for (int col = 0; col < 5; ++col) {
        uint8_t line = glyph[col];
        for (int row = 0; row < 8; ++row) {
            uint16_t pixel_color = (line & (1U << row)) ? fg : bg;
            lcd_setpixel(x + col, y + row, pixel_color);
        }
    }
    for (int row = 0; row < 8; ++row) {
        lcd_setpixel(x + 5, y + row, bg);
    }
}

static void draw_string(int x, int y, const char* str, uint16_t fg, uint16_t bg = color::Black) {
    int cur_x = x;
    while (*str) {
        if (cur_x + 6 > LCD_WIDTH) break;
        draw_char(cur_x, y, *str++, fg, bg);
        cur_x += 6;
    }
}

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
    printf("Sipeed Longan Nano -- Modern C++23 SD Card Test\n");
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
