/*!
    \file  main.c
    \brief running led
    
    \version 
*/

extern "C" {
#include "gd32vf103.h"
#include "systick.h"
#include "lcd.h"
}
#include "usb_device.h"
#include <stdio.h>
#include "gpio.h"
#include "usb.hpp"
#include "board.h"
#include "rotary_encoder.h" // Add the new header
#include <math.h>
#include "shared_defs.h" 

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

// --- State machine for sending HID actions ---
enum class HidActionState {
    IDLE,
    WAITING_FOR_PRESS_CONFIRM,
    WAITING_FOR_RELEASE_CONFIRM
};
static HidActionState hid_state = HidActionState::IDLE;

// --- Define the two quarter-sized framebuffers ---
uint8_t g_framebuffers[2][QUADRANT_BYTES];

// --- State variables for double buffering ---
volatile bool quadrant_ready_for_dma = false;
volatile int dma_buffer_idx = 0; // The buffer that is ready to be sent to the LCD
volatile int usb_buffer_idx = 0; // The buffer that USB is currently writing to

extern volatile bool is_new_frame_ready;
extern volatile int g_current_quadrant_idx;
extern volatile bool user_key_pressed;

// Create instances of the Led class for the onboard LEDs
// LEDR (Red LED) is on GPIOC, PIN_13 and is active low (lights up when pin is low)
static Led led_red(GPIOC, GPIO_PIN_13, true);
// LEDG (Green LED) is on GPIOA, PIN_1 and is active high
static Led led_green(GPIOA, GPIO_PIN_1);
// LEDB (Blue LED) is on GPIOA, PIN_2 and is active high
static Led led_blue(GPIOA, GPIO_PIN_2);

/*!
    \brief      main function
    \param[in]  none
    \param[out] none
    \retval     none
*/
int main(void)
{
    // 1. All initializations are the same
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

    // The call to usb::init will now always get `false`, correctly
    // configuring a 2-interface (HID-only) device.
    printf("Proceeding with USB initialization...\n");
    usb::init(sd_card_is_ok);
    printf("USB initialization complete.\n");
    
    // 5. Wait for configuration
    printf("Waiting for USB configuration from host...\n");
    while (!usb::is_configured()) {
        board_led_toggle();
        delay_1ms(200);
    }
    printf("USB device configured successfully!\n");
    board_led_on();

    // --- Variables for periodic event processing ---
    uint32_t last_action_time = 0;

    // 6. Main application loop with non-blocking state machine
    while(1){
        usb::poll();

        // Check if a quadrant has been fully received AND if the DMA is not busy
        if (quadrant_ready_for_dma/* && !lcd_is_dma_busy()*/) {
            printf("INFO: Quadrant %d received. Drawing to buffer %d.\n", g_current_quadrant_idx, dma_buffer_idx);
            // Calculate the Y coordinate for this quadrant
            int quadrant_y = g_current_quadrant_idx * QUADRANT_HEIGHT;
            printf("MAIN: DMA ready. Drawing quadrant %d to Y=%d\n", g_current_quadrant_idx, quadrant_y);

            // Start a non-blocking DMA transfer from the completed buffer
            lcd_write_u16(0, quadrant_y, QUADRANT_WIDTH, QUADRANT_HEIGHT, g_framebuffers[dma_buffer_idx]);
            
            quadrant_ready_for_dma = false;
        } else if(quadrant_ready_for_dma) {
            printf("INFO: Quadrant %d ready but DMA is busy. Skipping draw.\n", g_current_quadrant_idx);
        }

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

        // A minimal, non-blocking delay is still good practice to yield CPU time.
        delay_1ms(1);
    }
}