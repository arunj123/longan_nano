#include "systick.h"
#include "hal/time.hpp"

void delay_1ms(uint32_t count) {
    hal::time::delay_ms(count);
}
