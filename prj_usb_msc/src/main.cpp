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
    printf("Sipeed Longan Nano -- USB MSC SD Card Reader\n");
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
    printf("[MAIN] USB stack initialized. Waiting for host enumeration...\n");

    auto last_display_update = hal::time::Instant::now();
    auto last_heartbeat = hal::time::Instant::now();
    bool usb_was_configured = false;

    while (true) {
        usb::poll();

        // Drain and print USB control setup trace
        while (g_usb_trace_tail != g_usb_trace_head) {
            auto e = g_usb_trace[g_usb_trace_tail];
            g_usb_trace_tail = (g_usb_trace_tail + 1U) % USB_TRACE_MAX;
            switch (e.type) {
                case 0: // SETUP
                    printf("[SETUP] bm=0x%02X bReq=0x%02X val=0x%04X idx=0x%04X len=%u -> %s\n",
                           e.extra[0], e.extra[1], e.val1,
                           static_cast<uint16_t>(e.extra[2] | (e.extra[3] << 8)),
                           e.val2, (e.status == 0) ? "SUPP" : "NOTSUPP");
                    break;
                case 1: // IN_TF
                    printf("  [IN_TF] ctl=%u remain=%u xfer_len=%u\n", e.ctl_state, e.val1, e.val2);
                    break;
                case 3: // OUT_TF
                    printf("  [OUT_TF] ctl=%u\n", e.ctl_state);
                    break;
                case 4: // STATUS_RECV
                    printf("  [STATUS_RECV] ctl=%u\n", e.ctl_state);
                    break;
            }
        }

        // Drain and print MSC SCSI/BOT trace
        while (msc::g_msc_trace_tail != msc::g_msc_trace_head) {
            auto e = msc::g_msc_trace[msc::g_msc_trace_tail];
            msc::g_msc_trace_tail = (msc::g_msc_trace_tail + 1U) % MSC_TRACE_MAX;
            switch (e.type) {
                case 1: // CBW
                    printf("[MSC:CBW] Opcode=0x%02X CBLen=%u rx=%u len=%lu tag=0x%08lX CDB:[%02X %02X %02X %02X %02X %02X %02X %02X %02X %02X]\n",
                           e.opcode, e.val8, e.status,
                           static_cast<unsigned long>(e.val32_1),
                           static_cast<unsigned long>(e.val32_2),
                           e.cdb[0], e.cdb[1], e.cdb[2], e.cdb[3], e.cdb[4],
                           e.cdb[5], e.cdb[6], e.cdb[7], e.cdb[8], e.cdb[9]);
                    break;
                case 3: // CSW
                    printf("  [MSC:CSW] Opcode=0x%02X Status=%u Residue=%lu\n",
                           e.opcode, e.val8, static_cast<unsigned long>(e.val32_1));
                    break;
                case 4: // Abort
                    printf("  [MSC:ABORT] Opcode=0x%02X Step=%u Err=%u val1=%lu val2=0x%08lX\n",
                           e.opcode, e.val8, e.status,
                           static_cast<unsigned long>(e.val32_1),
                           static_cast<unsigned long>(e.val32_2));
                    break;
                case 5: // Class / Endpoint Req
                    printf("[MSC:REQ] bReq=0x%02X val=%u\n", e.opcode, e.val8);
                    break;
            }
        }

        bool configured = usb::is_configured();
        if (configured && !usb_was_configured) {
            usb_was_configured = true;
            printf("[MAIN] >>> USB CONFIGURED BY HOST! <<<\n");
            draw_string(4, 40, "USB: CONFIGURED OK ", color::Green);
        } else if (!configured && usb_was_configured) {
            usb_was_configured = false;
            printf("[MAIN] USB disconnected / reset.\n");
            draw_string(4, 40, "USB: DISCONNECTED  ", color::Red);
        }

        // Periodic telemetry display update (~100 ms)
        auto now = hal::time::Instant::now();
        if (now - last_display_update >= hal::time::Duration::from_ms(100)) {
            last_display_update = now;

            if (g_msc_stats.is_active) {
                g_msc_stats.is_active = false;
                LedBlue::toggle(); // Blink blue on disk access

                char r_buf[32], w_buf[32], lba_buf[32];
                snprintf(r_buf, sizeof(r_buf), "Read:  %lu blks", static_cast<unsigned long>(g_msc_stats.sectors_read));
                snprintf(w_buf, sizeof(w_buf), "Write: %lu blks", static_cast<unsigned long>(g_msc_stats.sectors_written));
                snprintf(lba_buf, sizeof(lba_buf), "LBA:   0x%08lX", static_cast<unsigned long>(g_msc_stats.last_sector));

                draw_string(4, 50, r_buf, color::Cyan);
                draw_string(4, 60, w_buf, color::Yellow);
                draw_string(4, 70, lba_buf, color::White);
            }
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
