/*!
    \file  main.c
    \brief running led
*/

extern "C" {
#include "gd32vf103.h"
#include "systick.h"
#include "lcd.h"
}
#include "usb_device.h"
#include <stdio.h>
// gpio.h is no longer needed
#include "usb.hpp"
#include "board.h"
#include "rotary_encoder.h"
#include <math.h>
#include "shared_defs.h" 
#include "display_manager.h"

#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    #include "sd_card.h"
    #include "sd_test.hpp"
    #include "usbd_msc_mem.h"
#endif

// Define some helpful consumer control usage codes
namespace hid_consumer {
    constexpr uint16_t VOLUME_UP   = 0x00E9;
    constexpr uint16_t VOLUME_DOWN = 0x00EA;
    constexpr uint16_t MUTE        = 0x00E2;
    constexpr uint16_t PLAY_PAUSE  = 0x00CD;
    constexpr uint16_t NO_KEY      = 0x0000;
}

extern volatile bool user_key_pressed;

// --- State machine for sending HID actions ---
enum class HidActionState {
    IDLE,
    WAITING_FOR_PRESS_CONFIRM,
    WAITING_FOR_RELEASE_CONFIRM
};
static HidActionState hid_state = HidActionState::IDLE;


/*!
    \brief      main function
    \param[in]  none
    \param[out] none
    \retval     none
*/
int main(void)
{
    board_led_init();
    board_key_init();
    encoder::init();
    lcd_init();

    delay_1ms(100);
    printf("\n\n--- System Initialized with Polling Architecture ---\n");

    bool sd_card_is_ok = false;

// The preprocessor will now skip this whole block
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    printf("Attempting to initialize SD Card...\n");
    if (!(sd_init() & STA_NOINIT)) {
        printf("INFO: SD Card initialized successfully.\n");
        sd_card_is_ok = true;
    } else {
        printf("WARN: SD Card initialization failed or card not present.\n");
    }
    
    if (sd_card_is_ok) {
        msc_mem_pre_init();
    }
#else
    printf("INFO: SD Card MSC feature is disabled in this build.\n");
#endif

    printf("Proceeding with USB initialization...\n");
    usb::init(false); // MSC is disabled
    printf("USB initialization complete.\n");
    
    printf("Waiting for USB configuration from host...\n");
    while (!usb::is_configured()) {
        board_led_toggle();
        delay_1ms(200);
    }
    printf("USB device configured successfully!\n");
    board_led_on(); // Turn on Green LED to indicate ready state

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

        if(user_key_pressed) {
            printf("User button pressed!\n");
            board_led_toggle();
            user_key_pressed = false;
        }

        // A minimal, non-blocking delay is still good practice to yield CPU time.
        delay_1ms(1);
    }
}