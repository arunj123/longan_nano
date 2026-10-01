#ifndef INA219_H
#define INA219_H

#include <stdint.h>
#include <stdbool.h>
#include "drivers/ina219.hpp"

#define INA219_ADDR 0x40

// Modern C++23 type alias mapping to drivers::Ina219Data
using ina219_data_t = drivers::Ina219Data;

bool ina219_init(void);
bool ina219_read_all(ina219_data_t *data);

#endif // INA219_H
