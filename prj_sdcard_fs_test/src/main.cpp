#include <cstdint>
#include <cstdio>
#include <cstring>
#include <array>
#include <span>

#include "lcd.h"
#include "hal/time.hpp"
#include "hal/uart.hpp"
#include "hal/gpio.hpp"
#include "drivers/sdcard.hpp"
#include "drivers/fatfs.hpp"

// ------------------------------------------------------------------------
// Standard 16-bit RGB565 Colors
// ------------------------------------------------------------------------
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

static uint16_t test_run_counter = 0;

static void run_filesystem_test() {
    ++test_run_counter;
    printf("\n=======================================================\n");
    printf(">>> RUNNING SD FAT32 FILESYSTEM TEST (RUN #%u) <<<\n", test_run_counter);
    printf("=======================================================\n");

    LedRed::set();    // OFF
    LedGreen::set();  // OFF
    LedBlue::reset(); // ON (Blue = Running test)

    lcd_fill_rect(0, 16, LCD_WIDTH, LCD_HEIGHT - 16, color::Black);
    draw_string(4, 18, "Mounting FAT32...", color::Yellow);

    auto& fs = drivers::fatfs::FileSystem::instance();
    FRESULT mount_res = fs.mount("0:", true);

    if (mount_res != FR_OK) {
        printf("[ERROR] Failed to mount FAT volume! Res: %d (%s)\n",
               mount_res, drivers::fatfs::result_to_string(mount_res));
        LedBlue::set();  // Blue OFF
        LedRed::reset(); // Red ON (Error)

        draw_string(4, 18, "Mount: FAILED!   ", color::Red);
        char err_msg[32];
        snprintf(err_msg, sizeof(err_msg), "Err: %d", mount_res);
        draw_string(4, 30, err_msg, color::Red);
        return;
    }

    printf("[FS] FAT32 Volume mounted successfully!\n");
    draw_string(4, 18, "Volume: MOUNT OK", color::Green);

    // 1. Generate pseudo-random 8.3 filename based on 64-bit hardware mtime ticks
    uint32_t ticks = static_cast<uint32_t>(hal::time::Instant::now().ticks);
    char filename[32];
    snprintf(filename, sizeof(filename), "/R%04X%02X.TXT",
             static_cast<unsigned int>(ticks & 0xFFFF),
             static_cast<unsigned int>(test_run_counter & 0xFF));

    printf("[FS] Target Filename: %s\n", filename);

    // 2. Prepare test payload
    char write_buf[256];
    int write_len = snprintf(write_buf, sizeof(write_buf),
        "=== Longan Nano FAT32 Test ===\r\n"
        "File: %s\r\n"
        "Test Run: #%u\r\n"
        "Timestamp Ticks: %lu\r\n"
        "Payload: ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\r\n"
        "Verification: GD32VF103 RV32IMAC OK\r\n"
        "==============================\r\n",
        filename, test_run_counter, static_cast<unsigned long>(ticks));

    // 3. Write file to SD card
    printf("[FS] Writing payload (%d bytes) to %s...\n", write_len, filename);
    draw_string(4, 28, "Writing file...", color::Yellow);

    FRESULT write_res = fs.write_file(filename, std::string_view{write_buf, static_cast<size_t>(write_len)});
    if (write_res != FR_OK) {
        printf("[ERROR] Failed to write file: %s (Res: %d -> %s)\n",
               filename, write_res, drivers::fatfs::result_to_string(write_res));
        LedBlue::set();
        LedRed::reset(); // Red ON
        draw_string(4, 28, "Write: FAILED!   ", color::Red);
        return;
    }
    printf("[FS] Successfully wrote %d bytes to %s!\n", write_len, filename);
    draw_string(4, 28, "Write: OK (512B)", color::Green);

    // 4. Re-open file and read back
    printf("[FS] Re-opening %s for read-back verification...\n", filename);
    draw_string(4, 38, "Reading file...", color::Yellow);

    char read_buf[256];
    std::memset(read_buf, 0, sizeof(read_buf));
    UINT bytes_read = 0;

    FRESULT read_res = fs.read_file(filename, std::span<char>{read_buf, sizeof(read_buf)}, bytes_read);
    if (read_res != FR_OK) {
        printf("[ERROR] Failed to read back file: %s (Res: %d -> %s)\n",
               filename, read_res, drivers::fatfs::result_to_string(read_res));
        LedBlue::set();
        LedRed::reset(); // Red ON
        draw_string(4, 38, "Read: FAILED!    ", color::Red);
        return;
    }

    printf("[FS] Successfully read %u bytes from %s.\n", bytes_read, filename);
    draw_string(4, 38, "Read:  OK", color::Green);

    // 5. Print read-back content to UART monitor
    printf("\n---------------- Read-Back File Content ----------------\n");
    printf("%s", read_buf);
    printf("--------------------------------------------------------\n\n");

    // 6. Verify byte-for-byte fidelity
    bool match = (bytes_read == static_cast<UINT>(write_len)) &&
                 (std::memcmp(write_buf, read_buf, static_cast<size_t>(write_len)) == 0);

    if (match) {
        printf(">>> [SUCCESS] 100%% VERIFIED: Read content matches written payload byte-for-byte! <<<\n");
        printf("    File: %s | Length: %u bytes\n\n", filename, bytes_read);

        // List files in root directory to show directory indexing
        DIR dir;
        FILINFO fno;
        if (f_opendir(&dir, "/") == FR_OK) {
            printf("--- Root Directory Listing (/) ---\n");
            while (f_readdir(&dir, &fno) == FR_OK && fno.fname[0] != 0) {
                printf("  %-12s  %6lu bytes%s\n",
                       fno.fname,
                       static_cast<unsigned long>(fno.fsize),
                       (fno.fattrib & AM_DIR) ? "  <DIR>" : "");
            }
            f_closedir(&dir);
            printf("----------------------------------\n\n");
        }

        LedBlue::set();   // Blue OFF
        LedRed::set();    // Red OFF
        LedGreen::reset(); // Green ON (PASS!)

        // Display summary on ST7735 LCD
        lcd_fill_rect(0, 16, LCD_WIDTH, LCD_HEIGHT - 16, color::Black);
        
        char line_buf[32];
        snprintf(line_buf, sizeof(line_buf), "File: %s", filename + 1); // omit leading slash
        draw_string(4, 18, line_buf, color::Cyan);

        snprintf(line_buf, sizeof(line_buf), "Size: %u Bytes OK", bytes_read);
        draw_string(4, 28, line_buf, color::Green);

        draw_string(4, 40, "VERIFY: 100% OK!", color::Yellow);
        draw_string(4, 52, "DATA BYTE MATCH", color::Green);
        draw_string(4, 66, "Press BTN: Retest", color::White);
    } else {
        printf(">>> [FAILURE] Byte mismatch in file read-back! <<<\n");
        LedBlue::set();
        LedGreen::set();
        LedRed::reset(); // Red ON

        draw_string(4, 50, "VERIFY: MISMATCH!", color::Red);
    }
}

int main() {
    // 1. Initialize Debug UART0 (115200 8N1)
    hal::uart::Uart0::init(115200);

    printf("\n\n");
    printf("=======================================================\n");
    printf("   Sipeed Longan Nano - Modern C++23 SD FatFs Test    \n");
    printf("   GD32VF103 RV32IMAC @ %lu MHz                       \n",
           static_cast<unsigned long>(SystemCoreClock / 1000000));
    printf("=======================================================\n");

    // 2. Initialize Status LEDs
    LedRed::init(hal::gpio::Mode::OutputPushPull);
    LedGreen::init(hal::gpio::Mode::OutputPushPull);
    LedBlue::init(hal::gpio::Mode::OutputPushPull);

    LedRed::set();    // Active-low: HIGH = OFF
    LedGreen::set();  // Active-low: HIGH = OFF
    LedBlue::set();   // Active-low: HIGH = OFF

    // 3. Initialize User Button (PA8, active-low with internal pull-up)
    KeyBtn::init(hal::gpio::Mode::InputPullUp);

    // 4. Initialize ST7735 SPI LCD (160x80)
    lcd_init();
    lcd_clear(color::Black);

    // Draw Title Header Bar
    lcd_fill_rect(0, 0, LCD_WIDTH, 14, color::DarkNavy);
    draw_string(14, 3, "SD FAT32 FS TEST", color::White, color::DarkNavy);

    // 5. Run first file system test
    run_filesystem_test();

    // 6. Run second file system test to demonstrate multiple random files
    hal::time::delay_ms(1000);
    run_filesystem_test();

    // 7. Main loop: poll button for repeated tests
    bool last_btn_state = (KeyBtn::read() == hal::gpio::Level::Low);
    auto last_debounce = hal::time::Instant::now();

    while (true) {
        bool current_btn_state = (KeyBtn::read() == hal::gpio::Level::Low);

        // Falling edge with 50ms debounce
        if (current_btn_state && !last_btn_state) {
            if (hal::time::Instant::now() - last_debounce > hal::time::Duration::from_ms(100)) {
                last_debounce = hal::time::Instant::now();
                printf("\n[BUTTON] User Key PA8 Pressed! Initiating new random file test...\n");
                run_filesystem_test();
            }
        }
        last_btn_state = current_btn_state;

        hal::time::delay_ms(10);
    }

    return 0;
}
