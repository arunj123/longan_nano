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
#include <cstdio>
#include <cstdint>
#include "bsp/board.hpp"
#include "hal/time.hpp"
#include "usb.hpp"

using bsp::board::LedRed;
using bsp::board::LedGreen;
using bsp::board::LedBlue;
using bsp::board::KeyButton;

int main() {
    // Zero-overhead board initialization: sets up LEDs and user key pin
    bsp::board::init();

    // Give UART hardware and host monitor a moment to settle
    hal::time::delay_ms(100);

    printf("\r\n==================================================\r\n");
    printf("  Sipeed Longan Nano -- USB CDC ACM Serial Device  \r\n");
    printf("  CPU: GD32VF103 RV32IMAC @ 96 MHz (48 MHz USB)   \r\n");
    printf("  Standard: C++23 (-std=gnu++23, -Os, -flto)      \r\n");
    printf("  Debug UART0: 115200 baud (PA9 TX, PA10 RX)       \r\n");
    printf("==================================================\r\n\r\n");

    // Initialize USB CDC-ACM peripheral & stack
    usb::init();

    uint32_t heartbeat_counter = 0;
    uint32_t led_step = 0;
    bool last_button_state = false;
    bool last_usb_state = false;

    auto last_led_time = hal::time::Instant::now();
    auto last_heartbeat_time = hal::time::Instant::now();

    while (true) {
        // High-frequency non-blocking USB polling
        usb::poll();

        const auto now = hal::time::Instant::now();

        // Check user button (PA8)
        const bool button_pressed = KeyButton::is_active();
        if (button_pressed && !last_button_state) {
            const uint32_t ms = hal::time::Duration::from_ticks(now.ticks).to_ms();
            printf(">>> User Button Pressed! [Time: %lu ms] <<<\r\n", static_cast<unsigned long>(ms));
        } else if (!button_pressed && last_button_state) {
            const uint32_t ms = hal::time::Duration::from_ticks(now.ticks).to_ms();
            printf(">>> User Button Released! [Time: %lu ms] <<<\r\n", static_cast<unsigned long>(ms));
        }
        last_button_state = button_pressed;

        // Check USB configured state changes
        const bool usb_configured = usb::is_configured();
        if (usb_configured != last_usb_state) {
            const uint32_t ms = hal::time::Duration::from_ticks(now.ticks).to_ms();
            printf(">>> USB State: %s [Time: %lu ms] <<<\r\n",
                   usb_configured ? "CONFIGURED (CDC Ready)" : "DISCONNECTED / RESET",
                   static_cast<unsigned long>(ms));
            last_usb_state = usb_configured;
        }

        // Non-blocking LED animation every 200 ms
        if (now.ticks - last_led_time.ticks >= hal::time::Duration::from_ms(200).ticks) {
            last_led_time = now;
            switch (led_step % 3) {
                case 0: LedRed::on();  LedGreen::off(); LedBlue::off(); break;
                case 1: LedRed::off(); LedGreen::on();  LedBlue::off(); break;
                case 2: LedRed::off(); LedGreen::off(); LedBlue::on();  break;
            }
            led_step++;
        }

        // Non-blocking heartbeat status output every 1000 ms
        if (now.ticks - last_heartbeat_time.ticks >= hal::time::Duration::from_ms(1000).ticks) {
            last_heartbeat_time = now;
            const uint32_t ms = hal::time::Duration::from_ticks(now.ticks).to_ms();
            printf("[Heartbeat #%04lu] USB: %s | Time: %lu ms\r\n",
                   static_cast<unsigned long>(heartbeat_counter++),
                   usb_configured ? "CONFIGURED" : "ENUMERATING...",
                   static_cast<unsigned long>(ms));
        }
    }
}
