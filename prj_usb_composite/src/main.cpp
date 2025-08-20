/*!
    \file  main.c
    \brief running led
    
    \version 
*/

extern "C" {
#include "gd32vf103.h"
#include "systick.h"
}
#include <stdio.h>
#include "gpio.h"
#include "usb.hpp"
#include "board.h"
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    #include "sd_card.h"
#endif
#include "sd_test.hpp"
#include "usbd_msc_mem.h"
#include "rotary_encoder.h" // Add the new header

// Define some helpful consumer control usage codes
namespace hid_consumer {
    constexpr uint16_t VOLUME_UP   = 0x00E9;
    constexpr uint16_t VOLUME_DOWN = 0x00EA;
    constexpr uint16_t MUTE        = 0x00E2;
    constexpr uint16_t PLAY_PAUSE  = 0x00CD;
    constexpr uint16_t NO_KEY      = 0x0000;
}

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
    // 1. Initialize board-specific hardware and basic peripherals
    board_led_init();
    board_key_init();
    encoder::init();  // Initialize our new rotary encoder

    delay_1ms(100);
    printf("\n\n--- System Initialized ---\n");

    bool sd_card_is_ok = false;

    // 2. Conditionally initialize the SD Card hardware
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    // 2. Initialize the SD Card hardware
    printf("Attempting to initialize SD Card...\n");
    if (!(sd_init() & STA_NOINIT)) {
        printf("INFO: SD Card initialized successfully.\n");
        sd_card_is_ok = true;
    } else {
        printf("WARN: SD Card initialization failed or card not present.\n");
    }
    
    // 3. Pre-cache MSC properties (only if card is ok)
    if (sd_card_is_ok) {
        msc_mem_pre_init();
    }
#else
    printf("INFO: SD Card MSC feature is disabled in this build.\n");
#endif

    // 4. Initialize the USB stack, telling it whether to enable the MSC interface.
    printf("Proceeding with USB initialization...\n");
    usb::init(sd_card_is_ok);

    // 5. Wait for configuration
    printf("Waiting for USB configuration from host...\n");
    while (!usb::is_configured()) {
        board_led_toggle();
        delay_1ms(200);
    }
    printf("USB device configured successfully!\n");
    board_led_on();

    // 6. Main application loop
    uint32_t loop_counter = 0;
    while(1){
        usb::poll();

        if (usb::is_configured() and user_key_pressed) {
            delay_1ms(50); 

            static uint8_t custom_report_counter = 0;

            // --- 1. Standard HID Demo ---

            // Action 1: Type 'H'
            usb::send_keyboard_report(0x02, 0x0B); // Press Shift + 'h'
            delay_1ms(20);                         // Wait for transfer to complete
            usb::send_keyboard_report(0x00, 0x00); // Release all keys
            delay_1ms(20);                         // Wait for transfer to complete

            // Action 2: Type 'i'
            usb::send_keyboard_report(0x00, 0x0C); // Press 'i'
            delay_1ms(20);                         // Wait for transfer to complete
            usb::send_keyboard_report(0x00, 0x00); // Release all keys
            delay_1ms(20);                         // Wait for transfer to complete

            // Action 3: Type '!'
            usb::send_keyboard_report(0x02, 0x1E); // Press Shift + '1'
            delay_1ms(20);                         // Wait for transfer to complete
            usb::send_keyboard_report(0x00, 0x00); // Release all keys
            delay_1ms(20);                         // Wait for transfer to complete

            // Action 4: Move Mouse
            usb::send_mouse_report(10, 10, 0, 0);
            delay_1ms(20);                         // Wait for transfer to complete
            // Mouse movement is relative, so no "release" report is needed.

            // Action 5: Volume Down
            usb::send_consumer_report(0x00EA); // Press Volume Down
            delay_1ms(20);                         // Wait for transfer to complete
            usb::send_consumer_report(0x0000); // Release Volume Down
            delay_1ms(20);                         // Wait for transfer to complete

            // --- 2. Custom HID Demo ---

            // Action 6: Send Custom Report
            usb::send_custom_hid_report(0x15, custom_report_counter++);
            // This uses a different endpoint, so it does not interfere with the standard HID endpoint.

            user_key_pressed = false; // Reset the key press flag
        }

        int8_t rotation = encoder::get_rotation();
        if (rotation > 0) {
            printf("Encoder: Volume Up\n");
            usb::send_consumer_report(hid_consumer::VOLUME_UP);
            delay_1ms(20);
            usb::send_consumer_report(hid_consumer::NO_KEY);
        } else if (rotation < 0) {
            printf("Encoder: Volume Down\n");
            usb::send_consumer_report(hid_consumer::VOLUME_DOWN);
            delay_1ms(20);
            usb::send_consumer_report(hid_consumer::NO_KEY);
        }

        if (encoder::is_pressed()) {
            printf("Encoder: Mute Toggled\n");
            usb::send_consumer_report(hid_consumer::MUTE);
            delay_1ms(20);
            usb::send_consumer_report(hid_consumer::NO_KEY);
        }

        delay_1ms(10); // Keep this delay!
    }
}
