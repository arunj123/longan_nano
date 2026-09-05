#ifndef I2C_HW_H
#define I2C_HW_H

#include <stdint.h>
#include <stdbool.h>

void i2c_hw_init(void);
bool i2c_hw_write_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t value);
bool i2c_hw_read_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t *value);

#endif // I2C_HW_H
