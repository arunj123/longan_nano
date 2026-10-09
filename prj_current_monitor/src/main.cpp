#include "lcd.h"
#include "hal/time.hpp"
#include "usb_hid/usb.hpp"
#include "bsp/board.hpp"
#include "ina219.h"
#include "display_manager.h"
#include <stdio.h>

static inline uint32_t get_ms_from_start(void) {
    return hal::time::uptime_ms();
}

int main(void)
{
    bsp::board::init();
    
    // Hardware I2C is initialized inside ina219_init()
    bool sensor_ok = ina219_init();
    if (!sensor_ok) {
        printf("INA219 Init Failed!\n");
        bsp::board::LedRed::on(); // Indicator for error
    } else {
        bsp::board::LedRed::off();
    }

    display::DisplayManager::getInstance().init();
    usb::init();

    printf("INA219 Oscilloscope Current Monitor Started (HW I2C)\n");

    uint32_t last_update = 0;
    uint32_t last_reinit_attempt = 0;
    uint32_t button_press_start = 0;
    bool prev_button_pressed = false;

    while(1) {
        usb::poll();

        // 1. Interactive Button Controls (PA8)
        bool button_pressed = bsp::board::KeyButton::is_active();
        if (button_pressed && !prev_button_pressed) {
            button_press_start = hal::time::uptime_ms();
            bsp::board::LedBlue::on(); // Visual feedback
        } else if (!button_pressed && prev_button_pressed) {
            uint32_t duration = hal::time::uptime_ms() - button_press_start;
            bsp::board::LedBlue::off();
            if (duration >= 1500) {
                // Long press: Reset accumulated energy & capacity!
                display::DisplayManager::getInstance().reset_energy();
            } else if (duration >= 50) {
                // Short press: Toggle screen mode (Graph vs Text)
                display::DisplayManager::getInstance().toggle_screen_mode();
            }
        }
        prev_button_pressed = button_pressed;

        // 2. 10 Hz Periodic Measurement & Oscilloscope Sweep
        uint32_t now = get_ms_from_start();
        if (now - last_update >= 100) {
            last_update = now;
            
            ina219_data_t data{};
            if (sensor_ok) {
                if (ina219_read_all(&data)) {
                    bsp::board::LedRed::off();
                    display::DisplayManager::getInstance().update(data, true);
                    
                    // Stream via USB HID
                    uint8_t report[9];
                    report[0] = 0x01; // Report ID
                    report[1] = static_cast<uint8_t>(data.voltage_mv & 0xFF);
                    report[2] = static_cast<uint8_t>(data.voltage_mv >> 8);
                    report[3] = static_cast<uint8_t>(data.current_ma & 0xFF);
                    report[4] = static_cast<uint8_t>(data.current_ma >> 8);
                    report[5] = static_cast<uint8_t>(data.power_mw & 0xFF);
                    report[6] = static_cast<uint8_t>(data.power_mw >> 8);
                    report[7] = 0;
                    report[8] = 0;
                    usb::send_report(report, 9);
                } else {
                    // I2C read glitch or disconnect
                    sensor_ok = false;
                    bsp::board::LedRed::on();
                    display::DisplayManager::getInstance().update(data, false);
                }
            } else {
                // Handle corner case: Hot-unplug recovery (retry every 1.5s)
                display::DisplayManager::getInstance().update(data, false);
                if (now - last_reinit_attempt >= 1500) {
                    last_reinit_attempt = now;
                    if (ina219_init()) {
                        sensor_ok = true;
                        bsp::board::LedRed::off();
                        printf("INA219 Reconnected!\n");
                    }
                }
            }
            
            bsp::board::LedGreen::toggle(); // Heartbeat
        }
    }
}
