extern "C" {
#include "gd32vf103.h"
#include "systick.h"
#include "lcd.h"
}
#include "usb_hid/usb.hpp"
#include "board.h"
#include "ina219.h"
#include "display_manager.h"
#include <stdio.h>

static uint32_t get_ms_from_start(void) {
    return (uint32_t)(get_timer_value() / (SystemCoreClock / 4000));
}

int main(void)
{
    board_led_init();
    board_key_init();
    
    // Hardware I2C is initialized inside ina219_init()
    if (!ina219_init()) {
        printf("INA219 Init Failed!\n");
        board_led_on(); // Indicator for error
    }

    display::DisplayManager::getInstance().init();
    usb::init();

    printf("INA219 Current Monitor Started (HW I2C)\n");

    uint32_t last_update = 0;
    while(1) {
        usb::poll();

        uint32_t now = get_ms_from_start();
        if (now - last_update >= 100) { // Update 10 times a second
            last_update = now;
            
            ina219_data_t data;
            if (ina219_read_all(&data)) {
                // Update LCD
                display::DisplayManager::getInstance().update(data);
                
                // Stream via USB HID
                uint8_t report[9];
                report[0] = 0x01; // Report ID
                report[1] = (uint8_t)(data.voltage_mv & 0xFF);
                report[2] = (uint8_t)(data.voltage_mv >> 8);
                report[3] = (uint8_t)(data.current_ma & 0xFF);
                report[4] = (uint8_t)(data.current_ma >> 8);
                report[5] = (uint8_t)(data.power_mw & 0xFF);
                report[6] = (uint8_t)(data.power_mw >> 8);
                report[7] = 0; // Padding
                report[8] = 0; // Padding
                
                usb::send_report(report, 9);
            }
            
            board_led_toggle();
        }
    }
}
