#include <cstdint>
#include "bsp/board.hpp"
#include "hal/time.hpp"

using bsp::board::LedRed;
using bsp::board::LedGreen;
using bsp::board::LedBlue;

int main() {
    bsp::board::init();

    while (true) {
        // Red
        LedRed::on();
        LedGreen::off();
        LedBlue::off();
        hal::time::delay_ms(500);

        // Green
        LedRed::off();
        LedGreen::on();
        LedBlue::off();
        hal::time::delay_ms(500);

        // Blue
        LedRed::off();
        LedGreen::off();
        LedBlue::on();
        hal::time::delay_ms(500);
    }
}
