#include "ina219.h"
#include "i2c_hw.h"

#define REG_CONFIG      0x00
#define REG_SHUNT_V     0x01
#define REG_BUS_V       0x02
#define REG_POWER       0x03
#define REG_CURRENT     0x04
#define REG_CALIBRATION 0x05

bool ina219_init(void) {
    i2c_hw_init();
    // Default config: 32V range, 320mV shunt range, 12-bit ADC
    // Calibration for 0.1 ohm shunt and 3.2A max: Cal = 4096
    return i2c_hw_write_reg(INA219_ADDR, REG_CALIBRATION, 4096);
}

bool ina219_read_all(ina219_data_t *data) {
    uint16_t v, c, p;
    if (!i2c_hw_read_reg(INA219_ADDR, REG_BUS_V, &v)) return false;
    if (!i2c_hw_read_reg(INA219_ADDR, REG_CURRENT, &c)) return false;
    if (!i2c_hw_read_reg(INA219_ADDR, REG_POWER, &p)) return false;

    data->voltage_mv = (uint16_t)((v >> 3) * 4);
    data->current_ma = (int16_t)((int16_t)c / 10);
    data->power_mw   = (uint16_t)(p * 2);
    return true;
}
