#include "ina219.h"
#include "i2c_hw.h"
#include "drivers/ina219.hpp"

using Ina219Driver = drivers::Ina219<INA219_ADDR>;

bool ina219_init(void) {
    i2c_hw_init();
    // Default config: 32V range, 320mV shunt range, 12-bit ADC
    // Calibration for 0.1 ohm shunt and 3.2A max: Cal = 4096
    return Ina219Driver::init(i2c_hw_write_reg, 4096);
}

bool ina219_read_all(ina219_data_t *data) {
    if (!data) return false;
    return Ina219Driver::read_all(i2c_hw_read_reg, *data);
}
