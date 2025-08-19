#ifndef _TF_CARD_H_
#define _TF_CARD_H_

#include "gd32vf103_libopt.h"
#include <stdio.h>

#define PA_OUT(n,s)                                         \
({  if (s%2)        gpio_bit_set(GPIOA, GPIO_PIN_##n);      \
    else            gpio_bit_reset(GPIOA, GPIO_PIN_##n);   })
#define PA_IN(n)    gpio_input_bit_get(GPIOA, GPIO_PIN_##n)

#define PB_OUT(n,s)                                         \
({  if (s%2)        gpio_bit_set(GPIOB, GPIO_PIN_##n);      \
    else            gpio_bit_reset(GPIOB, GPIO_PIN_##n);   })
#define PB_IN(n)    gpio_input_bit_get(GPIOB, GPIO_PIN_##n)

#define PC_OUT(n,s)                                         \
({  if (s%2)        gpio_bit_set(GPIOC, GPIO_PIN_##n);      \
    else            gpio_bit_reset(GPIOC, GPIO_PIN_##n);   })
#define PC_IN(n)    gpio_input_bit_get(GPIOC, GPIO_PIN_##n)

#define LEDR_TOG    gpio_bit_write(GPIOC, GPIO_PIN_13, (bit_status)(1-gpio_input_bit_get(GPIOC, GPIO_PIN_13)))
#define LEDG_TOG    gpio_bit_write(GPIOA, GPIO_PIN_1, (bit_status)(1-gpio_input_bit_get(GPIOA, GPIO_PIN_1)))
#define LEDB_TOG    gpio_bit_write(GPIOA, GPIO_PIN_2, (bit_status)(1-gpio_input_bit_get(GPIOA, GPIO_PIN_2)))

#define LEDR_ON     gpio_bit_set(GPIOC, GPIO_PIN_13)
#define LEDG_ON     gpio_bit_set(GPIOA, GPIO_PIN_1)
#define LEDB_ON     gpio_bit_set(GPIOA, GPIO_PIN_2)

#define LEDR_OFF    gpio_bit_reset(GPIOC, GPIO_PIN_13)
#define LEDG_OFF    gpio_bit_reset(GPIOA, GPIO_PIN_1)
#define LEDB_OFF    gpio_bit_reset(GPIOA, GPIO_PIN_2)
#include "diskio.h"
#include "ff.h"
#include "systick.h"

#endif
