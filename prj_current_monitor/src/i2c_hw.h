#pragma once

#include <cstdint>

void i2c_hw_init(void);
bool i2c_hw_write_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t value);
bool i2c_hw_read_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t *value);
