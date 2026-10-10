#include "i2c_hw.h"
#include "bsp/board.hpp"
#include "hal/i2c.hpp"

static constexpr uint32_t kI2c0Speed = 100000;

void i2c_hw_init(void) {
    /* Initialize PB6 (SCL) and PB7 (SDA) as Alternate Open Drain 50MHz */
    bsp::board::I2c0Scl::init(hal::gpio::Mode::AlternateOpenDrain, hal::gpio::Speed::Speed50MHz);
    bsp::board::I2c0Sda::init(hal::gpio::Mode::AlternateOpenDrain, hal::gpio::Speed::Speed50MHz);

    /* Enable I2C0 peripheral clock */
    hal::rcu::enable(hal::rcu::Peripheral::I2c0);

    /* Initialize I2C0 in master mode at 100 kHz */
    hal::i2c::I2c0::init(kI2c0Speed, hal::i2c::DutyCycle::Ratio2);
}

bool i2c_hw_write_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t value) {
    return hal::i2c::I2c0::write_reg16(dev_addr, reg_addr, value);
}

bool i2c_hw_read_reg(uint8_t dev_addr, uint8_t reg_addr, uint16_t *value) {
    return hal::i2c::I2c0::read_reg16(dev_addr, reg_addr, value);
}
