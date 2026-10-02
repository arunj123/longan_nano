/*!
    \file  main.c
    \brief running led
*/

#include "hal/time.hpp"
#include "lcd.h"
#include "usb_device.h"
#include "drivers/usb/usbd_transc.h"
#include <stdio.h>
// gpio.h is no longer needed
#include "bsp/board.hpp"
void board_key_init(void);
#include "rotary_encoder.h"
#include <math.h>
#include "shared_defs.h" 
#include "display_manager.h"


// Define some helpful consumer control usage codes
namespace hid_consumer {
    constexpr uint16_t VOLUME_UP   = 0x00E9;
    constexpr uint16_t VOLUME_DOWN = 0x00EA;
    constexpr uint16_t MUTE        = 0x00E2;
    constexpr uint16_t PLAY_PAUSE  = 0x00CD;
    constexpr uint16_t NO_KEY      = 0x0000;
}

// --- State machine for sending HID actions ---
enum class HidActionState {
    IDLE,
    WAITING_FOR_PRESS_CONFIRM,
    WAITING_FOR_RELEASE_CONFIRM
};
static HidActionState hid_state = HidActionState::IDLE;


// Uses unified C++23 font library from lib/gd32v_lcd

/* main function */
int main(void)
{
    bsp::board::init();
    board_key_init();
    encoder::init();
    lcd_init();

    // Visual splash on boot: confirms LCD operation and informs user of USB status
    lcd_clear(lcd::color::DarkNavy);
    lcd_rect(0, 0, 160, 80, lcd::color::Cyan);
    lcd_fill_rect(1, 1, 158, 13, lcd::color::Gray);
    lcd::draw_string(28, 4, "LONGAN NANO USB", lcd::color::Yellow, lcd::color::Gray);
    lcd::draw_string(8, 22, "HID Composite Stack", lcd::color::White, lcd::color::DarkNavy);
    lcd::draw_string(8, 38, "Waiting for Host...", lcd::color::Red, lcd::color::DarkNavy);
    lcd::draw_string(8, 54, "Compile-Time Concept", lcd::color::Cyan, lcd::color::DarkNavy);
    lcd::draw_string(8, 66, "PID 0xABDD @ 96MHz", lcd::color::LightGray, lcd::color::DarkNavy);

    // Pre-initialize DisplayManager singleton to ensure static memory is ready
    display::DisplayManager::getInstance();

    hal::time::delay_ms(100);
    printf("\n\n--- System Initialized with Polling Architecture ---\n");

    printf("Proceeding with USB initialization...\n");
    usb::init();
    printf("USB initialization complete.\n");

    printf("CfgDesc [len=%u]: ", (unsigned)composite_config_desc.config.wTotalLength);
    const uint8_t *cfg_ptr = (const uint8_t *)&composite_config_desc;
    for (size_t i = 0; i < composite_config_desc.config.wTotalLength; ++i) {
        printf("%02X ", cfg_ptr[i]);
    }
    printf("\n");

    printf("Waiting for USB configuration from host...\n");
    auto last_blink = hal::time::Instant::now();
    while (!usb::is_configured()) {
        usb::poll();

        while (g_usb_trace_tail != g_usb_trace_head) {
            auto e = g_usb_trace[g_usb_trace_tail];
            g_usb_trace_tail = (g_usb_trace_tail + 1U) % USB_TRACE_MAX;
            switch(e.type) {
            case 0: // SETUP
                printf("[SETUP] bm=0x%02X bReq=0x%02X val=0x%04X idx=0x%04X len=%u -> %s\n",
                       e.extra[0], e.extra[1], e.val1, (uint16_t)(e.extra[2] | (e.extra[3] << 8)),
                       e.val2, (e.status == 0) ? "SUPP" : "NOTSUPP");
                break;
            case 1: // IN_TF
                printf("  [IN_TF] ctl=%u remain=%u xfer_len=%u\n", e.ctl_state, e.val1, e.val2);
                break;
            case 2: // TXFE
                printf("  [TXFE]  ctl=%u len=%u count=%u\n", e.ctl_state, e.val1, e.val2);
                break;
            case 3: // OUT_TF
                printf("  [OUT_TF] ctl=%u\n", e.ctl_state);
                break;
            case 4: // STATUS_RECV
                printf("  [ST_RCV] ctl=%u\n", e.ctl_state);
                break;
            default:
                break;
            }
        }

        if (last_blink.elapsed() >= hal::time::Duration::from_ms(200)) {
            bsp::board::LedGreen::toggle();
            last_blink = hal::time::Instant::now();
        }
    }
    printf("USB device configured successfully!\n");
    lcd::draw_string(8, 38, "USB Configured: OK! ", lcd::color::Green, lcd::color::DarkNavy);
    bsp::board::LedGreen::on(); // Turn on Green LED to indicate ready state

    // 6. Main application loop with non-blocking state machine
    while(1){
        usb::poll();

        display::DisplayManager::getInstance().processDrawTasks();

        // Step 1: Check for new user input only when idle.
        if (hid_state == HidActionState::IDLE) {
            int8_t rotation = encoder::get_rotation();
            uint16_t action_key = hid_consumer::NO_KEY;

            if (rotation > 0) {
                printf("Action: Sending Volume Up...\n");
                action_key = hid_consumer::VOLUME_UP;
            } else if (rotation < 0) {
                printf("Action: Sending Volume Down...\n");
                action_key = hid_consumer::VOLUME_DOWN;
            } else if (encoder::is_pressed()) {
                printf("Action: Sending Mute...\n");
                action_key = hid_consumer::MUTE;
            }

            // If an action was detected, send it and change state.
            if (action_key != hid_consumer::NO_KEY) {
                usb::send_consumer_report(action_key);
                hid_state = HidActionState::WAITING_FOR_PRESS_CONFIRM;
            }
        }
        
        // Step 2: If we sent a "press", wait for it to complete before sending the "release".
        else if (hid_state == HidActionState::WAITING_FOR_PRESS_CONFIRM) {
            if (usb::is_std_hid_transfer_complete()) {
                printf("Action: Press confirmed. Sending Release.\n");
                usb::send_consumer_report(hid_consumer::NO_KEY);
                hid_state = HidActionState::WAITING_FOR_RELEASE_CONFIRM; // Now wait for the release to complete
            }
        }

        // Step 3: If we sent a "release", wait for it to complete before returning to idle.
        else if (hid_state == HidActionState::WAITING_FOR_RELEASE_CONFIRM) {
            if (usb::is_std_hid_transfer_complete()) {
                printf("Action: Release confirmed. Returning to Idle.\n");
                hid_state = HidActionState::IDLE; // Ready for new input
            }
        }

        extern volatile bool user_key_pressed;
        if (user_key_pressed) {
            // Send theme change request report to host (Report [0x01, 0x01])
            uint8_t report_payload[2] = {0x01, 0x01};
            if (usb::send_custom_hid_report(report_payload, sizeof(report_payload))) {
                printf("User button pressed! Theme change report sent.\n");
                bsp::board::LedGreen::toggle();
                user_key_pressed = false;
            }
        }
    }
}