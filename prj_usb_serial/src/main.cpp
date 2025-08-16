/*!
    \file  main.c
    \brief running led
    
    \version 2019-6-5, V1.0.0, firmware for GD32VF103
*/

/*
    Copyright (c) 2019, GigaDevice Semiconductor Inc.

    Redistribution and use in source and binary forms, with or without modification, 
are permitted provided that the following conditions are met:

    1. Redistributions of source code must retain the above copyright notice, this 
       list of conditions and the following disclaimer.
    2. Redistributions in binary form must reproduce the above copyright notice, 
       this list of conditions and the following disclaimer in the documentation 
       and/or other materials provided with the distribution.
    3. Neither the name of the copyright holder nor the names of its contributors 
       may be used to endorse or promote products derived from this software without 
       specific prior written permission.

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" 
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED 
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. 
IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, 
INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT 
NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR 
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, 
WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) 
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY 
OF SUCH DAMAGE.
*/
extern "C" {
#include "gd32vf103.h"
#include "systick.h"

int _write(int file, char *ptr, int len);
}
#include <stdio.h>
#include "gpio.h"
#include "usb.hpp"


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
    /* enable the LED clock */
    rcu_periph_clock_enable(RCU_GPIOC);
    rcu_periph_clock_enable(RCU_GPIOA);
    // Initialize LED pins as push-pull outputs
    // Red LED (PC13)
    gpio_init(GPIOC, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_13);
    // Green (PA1) and Blue (PA2) LEDs
    gpio_init(GPIOA, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_1 | GPIO_PIN_2);

    // Configure USART0 with the correct pins and parameters.
    usart0_config();
    delay_1ms(100);

    // Initialize the USB device with a single call
    usb::init();
    
    uint32_t counter = 0;
    while(1){
        // Print a message with a counter to show the program is running.
        printf("Counter value: %lu\n", counter++);    
        /* turn on LED1, turn off LED4 */
        led_red.on();
        led_green.on();
        led_blue.off();
        
        delay_1ms(100);

        // Handle periodic USB tasks
        usb::poll();

        /* turn off LED1, turn on LED4 */
        led_red.off();
        led_green.off();
        led_blue.off();
        delay_1ms(100);

        led_red.on();
        led_green.off();
        led_blue.on();
        delay_1ms(100);
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