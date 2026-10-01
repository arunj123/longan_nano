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
// ------------------------------------------------------------------------
// Modern ST7735 LCD SD Card FatFs Test
// Uses unified C++23 font library from lib/gd32v_lcd
// ------------------------------------------------------------------------

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
    printf("   Longan Nano - Modern C++23 SD FatFs Test           \n");
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
