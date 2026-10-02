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

// ------------------------------------------------------------------------
// Modern ST7735 LCD USB MSC Application
// Uses unified C++23 font library from lib/gd32v_lcd
// ------------------------------------------------------------------------

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
    lcd_clear(lcd::color::Black);

    // Title banner
    lcd_fill_rect(0, 0, LCD_WIDTH, 14, lcd::color::DarkNavy);
    lcd::draw_string(6, 3, "USB SD CARD READER", lcd::color::Cyan, lcd::color::DarkNavy);

    // 4. Initialize SD card
    lcd::draw_string(4, 18, "Probing SD card...", lcd::color::Yellow);
    bool sd_ok = msc_disk_init();

    if (sd_ok) {
        LedGreen::reset(); // ON (Green = Card ready)
        lcd::draw_string(4, 18, "SD Card: DETECTED", lcd::color::Green);

        char cap_buf[32];
        using Sd = drivers::sdcard::SdCard<>;
        snprintf(cap_buf, sizeof(cap_buf), "Cap: %lu MB",
                 static_cast<unsigned long>(Sd::sector_count / 2048));
        lcd::draw_string(4, 28, cap_buf, lcd::color::White);

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
        lcd::draw_string(4, 18, "SD Card: NOT FOUND", lcd::color::Red);
        lcd::draw_string(4, 28, "Insert card & reset", lcd::color::Yellow);
    }

    // 5. Initialize USB stack
    printf("CfgDesc [len=%u]: ", static_cast<unsigned>(msc_config_desc.config.wTotalLength));
    const uint8_t *cfg_ptr = reinterpret_cast<const uint8_t*>(&msc_config_desc);
    for (size_t i = 0; i < msc_config_desc.config.wTotalLength; ++i) {
        printf("%02X ", cfg_ptr[i]);
    }
    printf("\n");

    lcd::draw_string(4, 40, "USB: Initializing... ", lcd::color::Yellow);
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
            lcd::draw_string(4, 40, "USB: CONFIGURED OK   ", lcd::color::Green);
            printf("[MAIN] >>> USB CONFIGURED BY HOST! <<<\n");
        } else if (!configured && usb_was_configured) {
            usb_was_configured = false;
            lcd::draw_string(4, 40, "USB: Disconnected... ", lcd::color::Yellow);
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
