#include <cstdint>
#include "hal/uart.hpp"

extern "C" {
int _write(int file, char *ptr, int len);
void initialise_debug_uart(void);
}

void initialise_debug_uart(void) {
    hal::uart::Uart0::init(115200);
}

__attribute__((used)) int _write(int file, char *ptr, int len) {
    (void)file;
    if (!ptr || len <= 0) return 0;
    for (int i = 0; i < len; ++i) {
        hal::uart::Uart0::putc(ptr[i]);
    }
    return len;
}