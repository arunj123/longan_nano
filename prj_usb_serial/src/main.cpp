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
