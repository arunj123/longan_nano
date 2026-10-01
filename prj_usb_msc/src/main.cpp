#include <cstdint>
#include <cstdio>
#include <cstring>

#include "lcd.h"
#include "hal/time.hpp"
#include "hal/uart.hpp"
#include "hal/gpio.hpp"
#include "drivers/sdcard.hpp"
#include "usb_msc/usb.hpp"
#include "usb_msc/msc_core.hpp"
#include "usb_msc/msc_disk.hpp"
#include "usb_msc/usbd_descriptors.hpp"
#include "drivers/usb/usbd_transc.h"

// Colors
namespace color {
    constexpr uint16_t Black    = 0x0000;
    constexpr uint16_t White    = 0xFFFF;
    constexpr uint16_t Red      = 0xF800;
    constexpr uint16_t Green    = 0x07E0;
    constexpr uint16_t Blue     = 0x001F;
    constexpr uint16_t Yellow   = 0xFFE0;
    constexpr uint16_t Cyan     = 0x07FF;
    constexpr uint16_t DarkNavy = 0x000B;
}

// 5x7 ASCII Font
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

// Hardware LEDs
using LedRed   = hal::gpio::GpioPin<hal::gpio::Port::C, 13>;
using LedGreen = hal::gpio::GpioPin<hal::gpio::Port::A, 1>;
using LedBlue  = hal::gpio::GpioPin<hal::gpio::Port::A, 2>;

int main() {
    // 1. Initialize Debug UART0 @ 115200
    hal::uart::Uart0::init(115200);

    printf("\n===================================================\n");
    printf("Longan Nano -- USB MSC SD Card Reader\n");
    printf("GD32VF103 RV32IMAC @ %lu MHz (USB Clock: 48 MHz)\n",
           static_cast<unsigned long>(SystemCoreClock / 1000000));
    printf("Mass Storage Class (SCSI transparent / Bulk-Only Transport)\n");
    printf("===================================================\n");

    // 2. Initialize LEDs
    LedRed::init(hal::gpio::Mode::OutputPushPull);
    LedGreen::init(hal::gpio::Mode::OutputPushPull);
    LedBlue::init(hal::gpio::Mode::OutputPushPull);
    LedRed::set();   // Active-low: HIGH = OFF
    LedGreen::set();
    LedBlue::set();

    // 3. Initialize ST7735 LCD
    lcd_init();
    lcd_clear(color::Black);

    // Title banner
    lcd_fill_rect(0, 0, LCD_WIDTH, 14, color::DarkNavy);
    draw_string(6, 3, "USB SD CARD READER", color::Cyan, color::DarkNavy);

    // 4. Initialize SD card
    draw_string(4, 18, "Probing SD card...", color::Yellow);
    bool sd_ok = msc_disk_init();

    if (sd_ok) {
        LedGreen::reset(); // ON (Green = Card ready)
        draw_string(4, 18, "SD Card: DETECTED", color::Green);

        char cap_buf[32];
        using Sd = drivers::sdcard::SdCard<>;
        snprintf(cap_buf, sizeof(cap_buf), "Cap: %lu MB",
                 static_cast<unsigned long>(Sd::sector_count / 2048));
        draw_string(4, 28, cap_buf, color::White);

        alignas(4) uint8_t test_sec[512];
        auto t0 = hal::time::Instant::now();
        auto res_0 = Sd::read_sector(0, test_sec);
        auto dt_0 = hal::time::Instant::now() - t0;
        printf("[MAIN] Real SD Sector 0: %s (%lu us, Sig: 0x%02X%02X)\n",
               drivers::sdcard::result_to_string(res_0),
               static_cast<unsigned long>(dt_0.to_us()),
               test_sec[510], test_sec[511]);

        auto t1 = hal::time::Instant::now();
        auto res_496 = Sd::read_sector(496, test_sec);
        auto dt_496 = hal::time::Instant::now() - t1;
        printf("[MAIN] Real SD Sector 496 (VBR): %s (%lu us, Sig: 0x%02X%02X)\n",
               drivers::sdcard::result_to_string(res_496),
               static_cast<unsigned long>(dt_496.to_us()),
               test_sec[510], test_sec[511]);

        auto t2 = hal::time::Instant::now();
        auto res_560 = Sd::read_sector(560, test_sec);
        auto dt_560 = hal::time::Instant::now() - t2;
        printf("[MAIN] Real SD Sector 560 (FAT): %s (%lu us, Sig: 0x%02X%02X)\n",
               drivers::sdcard::result_to_string(res_560),
               static_cast<unsigned long>(dt_560.to_us()),
               test_sec[510], test_sec[511]);

        for (uint32_t s = 1024; s <= 1031; ++s) {
            auto ts = hal::time::Instant::now();
            auto res_s = Sd::read_sector(s, test_sec);
            auto dt_s = hal::time::Instant::now() - ts;
            printf("[MAIN] Real SD Sector %lu: %s (%lu us, Sig: 0x%02X%02X)\n",
                   static_cast<unsigned long>(s),
                   drivers::sdcard::result_to_string(res_s),
                   static_cast<unsigned long>(dt_s.to_us()),
                   test_sec[510], test_sec[511]);
        }

        g_msc_stats.sectors_read = 0;
        g_msc_stats.is_active = false;
    } else {
        LedRed::reset(); // ON (Red = Card error)
        draw_string(4, 18, "SD Card: NOT FOUND", color::Red);
        draw_string(4, 28, "Insert card & reset", color::Yellow);
    }

    // 5. Initialize USB stack
    printf("CfgDesc [len=%u]: ", static_cast<unsigned>(msc_config_desc.config.wTotalLength));
    const uint8_t *cfg_ptr = reinterpret_cast<const uint8_t*>(&msc_config_desc);
    for (size_t i = 0; i < msc_config_desc.config.wTotalLength; ++i) {
        printf("%02X ", cfg_ptr[i]);
    }
    printf("\n");

    draw_string(4, 40, "USB: Initializing... ", color::Yellow);
    usb::init();
    extern usb_core_driver msc_udev;
    printf("[FIFO_CFG] GRFLEN=0x%08lX DIEP0=0x%08lX DIEP1=0x%08lX DIEPTFSTAT1=0x%08lX\n",
           static_cast<unsigned long>(msc_udev.regs.gr->GRFLEN),
           static_cast<unsigned long>(msc_udev.regs.gr->DIEP0TFLEN_HNPTFLEN),
           static_cast<unsigned long>(msc_udev.regs.gr->DIEPTFLEN[0]),
           static_cast<unsigned long>(msc_udev.regs.er_in[1]->DIEPTFSTAT));
    printf("[MAIN] USB stack initialized. Waiting for host enumeration...\n");

    auto last_heartbeat = hal::time::Instant::now();
    bool usb_was_configured = false;

    while (true) {
        usb::poll();

        auto now = hal::time::Instant::now();

        // In-memory trace buffers (g_usb_trace, g_msc_trace) remain accessible via OpenOCD
        // (dump_trace.py). Runtime printing over UART0 is disabled to eliminate busy-wait
        // TX polling latency during active USB communications.

        bool configured = usb::is_configured();
        if (configured && !usb_was_configured) {
            usb_was_configured = true;
            draw_string(4, 40, "USB: CONFIGURED OK   ", color::Green);
            printf("[MAIN] >>> USB CONFIGURED BY HOST! <<<\n");
        } else if (!configured && usb_was_configured) {
            usb_was_configured = false;
            draw_string(4, 40, "USB: Disconnected... ", color::Yellow);
            printf("[MAIN] USB disconnected / reset.\n");
        }

        // Telemetry activity LED
        if (g_msc_stats.is_active) {
            g_msc_stats.is_active = false;
            LedBlue::toggle();
        }

        // Heartbeat LED
        if (now - last_heartbeat >= hal::time::Duration::from_ms(1000)) {
            last_heartbeat = now;
            if (sd_ok) {
                LedGreen::toggle();
            }
        }
    }

    return 0;
}
