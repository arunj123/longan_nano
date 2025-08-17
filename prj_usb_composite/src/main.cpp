/*!
    \file  main.c
    \brief running led
    
    \version 2019-6-5, firmware for GD32VF103
*/

extern "C" {
#include "gd32vf103.h"
#include "systick.h"

int _write(int file, char *ptr, int len);
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

extern volatile bool user_key_pressed;

// Create instances of the Led class for the onboard LEDs
// LEDR (Red LED) is on GPIOC, PIN_13 and is active low (lights up when pin is low)
static Led led_red(GPIOC, GPIO_PIN_13, true);
// LEDG (Green LED) is on GPIOA, PIN_1 and is active high
static Led led_green(GPIOA, GPIO_PIN_1);
// LEDB (Blue LED) is on GPIOA, PIN_2 and is active high
static Led led_blue(GPIOA, GPIO_PIN_2);

// Function prototypes
void usart0_config(void);

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
    usart0_config();
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

        // Add a slow heartbeat to show the main loop is not blocked
        delay_1ms(100);
        if(loop_counter % 10 == 0) {
            printf("Main loop heartbeat: %lu\n", loop_counter++);
        }
    }
}

/**
 * @brief      Configures the USART0 peripheral and its GPIO pins.
 * @param[in]  none
 * @param[out] none
 * @retval     none
 */
void usart0_config(void)
{
    /* --- Step 1: Enable peripheral clocks --- */

    // Enable the clock for GPIOA. The USART0 pins we are using (PA9, PA10) are on this port.
    rcu_periph_clock_enable(RCU_GPIOA);
    // Enable the clock for the USART0 peripheral itself.
    rcu_periph_clock_enable(RCU_USART0);

    /* --- Step 2: Configure GPIO pins for USART0 --- */

    // Configure PA9 (USART0_TX) as an alternate function push-pull output.
    // This allows the USART peripheral to take control of the pin and drive it.
    gpio_init(GPIOA, GPIO_MODE_AF_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_9);

    // Configure PA10 (USART0_RX) as a floating input.
    // This is the standard configuration for a UART receive pin.
    gpio_init(GPIOA, GPIO_MODE_IN_FLOATING, GPIO_OSPEED_50MHZ, GPIO_PIN_10);

    /* --- Step 3: Configure USART0 parameters --- */

    // Reset the USART0 peripheral to its default state.
    usart_deinit(USART0);
    // Set the baud rate to 115200.
    usart_baudrate_set(USART0, 115200U);
    // Set the word length to 8 data bits.
    usart_word_length_set(USART0, USART_WL_8BIT);
    // Set the stop bit length to 1 bit.
    usart_stop_bit_set(USART0, USART_STB_1BIT);
    // Configure no parity.
    usart_parity_config(USART0, USART_PM_NONE);
    // Disable hardware flow control (RTS/CTS).
    usart_hardware_flow_rts_config(USART0, USART_RTS_DISABLE);
    usart_hardware_flow_cts_config(USART0, USART_CTS_DISABLE);

    /* --- Step 4: Enable USART0 Transmit and Receive --- */

    // Enable the receiver.
    usart_receive_config(USART0, USART_RECEIVE_ENABLE);
    // Enable the transmitter.
    usart_transmit_config(USART0, USART_TRANSMIT_ENABLE);

    /* --- Step 5: Enable the USART0 peripheral --- */
    usart_enable(USART0);
}

/**
 * @brief      Retargets the C library `printf` function to USART0.
 * @details    This function is a standard C library hook. When `printf` is called,
 *             the C library eventually calls this `_write` function to output the
 *             formatted string. We implement it here to send the characters
 *             one by one over USART0.
 * @param[in]  file: File descriptor (unused).
 * @param[in]  ptr: Pointer to the buffer of characters to write.
 * @param[in]  len: The number of characters to write.
 * @retval     The number of characters successfully written.
 */
int _write(int file, char *ptr, int len)
{
    (void)file; // Unused parameter, silence compiler warning
    int i;
    for (i = 0; i < len; i++) {
        // Send the character.
        usart_data_transmit(USART0, (uint8_t)ptr[i]);
        // Wait until the data has been transmitted.
        // The USART_FLAG_TBE (Transmit Buffer Empty) flag is set by hardware
        // when the transmit shift register is empty and ready for new data.
        // Polling this flag ensures we don't try to send a new character
        // before the previous one has finished sending.
        while (usart_flag_get(USART0, USART_FLAG_TBE) == RESET);
    }
    return len;
}