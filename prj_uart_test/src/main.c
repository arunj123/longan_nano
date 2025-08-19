#include "gd32vf103.h"
#include <stdio.h>

int main(void)
{
    // Uart initialization is called before main loop.
    // Hence, we can use printf and other UART functions here.
    printf("Hello, Longan Nano!\n");
    while (1);
}
