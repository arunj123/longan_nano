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
#include "bsp/board.hpp"
#include "hal/time.hpp"
#include "usb.hpp"

using bsp::board::LedRed;
using bsp::board::LedGreen;
using bsp::board::LedBlue;

/*!
    \brief      main function
    \param[in]  none
    \param[out] none
    \retval     none
*/
int main(void)
{
    // Initialize board LEDs with zero RAM overhead
    bsp::board::init();

    // Initialize the USB device with a single call
    usb::init();
    
    uint32_t counter = 0;
    while (1) {
        // Print a message with a counter to show the program is running.
        printf("Counter value: %lu\n", counter++);    
        
        LedRed::on();
        LedGreen::on();
        LedBlue::off();
        hal::time::delay_ms(100);

        // Handle periodic USB tasks
        usb::poll();

        LedRed::off();
        LedGreen::off();
        LedBlue::off();
        hal::time::delay_ms(100);

        LedRed::on();
        LedGreen::off();
        LedBlue::on();
        hal::time::delay_ms(100);
    }
}
