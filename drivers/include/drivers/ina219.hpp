#pragma once

#include <cstdint>
#include "hal/time.hpp"

namespace drivers {

struct Ina219Data {
    uint16_t voltage_mv{0};
    int16_t  current_ma{0};
    uint16_t power_mw{0};
};

/**
 * @brief Modern C++23 fixed-point INA219 driver.
 * Strictly uses integer arithmetic (mV, mA, mW) with zero software float emulation.
 */
template <uint8_t I2cAddress = 0x40>
class Ina219 {
public:
    static constexpr uint8_t address = I2cAddress;

    // INA219 Registers
    static constexpr uint8_t REG_CONFIG      = 0x00;
    static constexpr uint8_t REG_SHUNT_V     = 0x01;
    static constexpr uint8_t REG_BUS_V       = 0x02;
    static constexpr uint8_t REG_POWER       = 0x03;
    static constexpr uint8_t REG_CURRENT     = 0x04;
    static constexpr uint8_t REG_CALIBRATION = 0x05;

    /**
     * @brief Initialize INA219 with 0.1 ohm shunt calibration.
     * @param write_fn Function pointer to write 16-bit register over I2C.
     */
    template <typename WriteFn>
    static bool init(WriteFn write_fn) noexcept {
        // Calibration for 0.1 ohm shunt resistor and 3.2A max range: Cal = 4096
        return write_fn(address, REG_CALIBRATION, 4096);
    }

    /**
     * @brief Read Bus Voltage in millivolts.
     */
    template <typename ReadFn>
    static bool read_voltage_mv(ReadFn read_fn, uint16_t& out_mv) noexcept {
        uint16_t raw;
        if (!read_fn(address, REG_BUS_V, &raw)) return false;
        // Bus voltage is in bits 3-15; 4mV per LSB
        out_mv = static_cast<uint16_t>((raw >> 3) * 4);
        return true;
    }

    /**
     * @brief Read Current in milliamperes (signed).
     */
    template <typename ReadFn>
    static bool read_current_ma(ReadFn read_fn, int16_t& out_ma) noexcept {
        uint16_t raw;
        if (!read_fn(address, REG_CURRENT, &raw)) return false;
        // Cal = 4096 with 0.1 ohm shunt: 1 LSB = 100uA = 0.1mA -> divide by 10
        out_ma = static_cast<int16_t>(static_cast<int16_t>(raw) / 10);
        return true;
    }

    /**
     * @brief Read Power in milliwatts.
     */
    template <typename ReadFn>
    static bool read_power_mw(ReadFn read_fn, uint16_t& out_mw) noexcept {
        uint16_t raw;
        if (!read_fn(address, REG_POWER, &raw)) return false;
        // Power LSB = 20 * Current LSB = 2mW
        out_mw = static_cast<uint16_t>(raw * 2);
        return true;
    }

    /**
     * @brief Read all measurements into an Ina219Data struct.
     */
    template <typename ReadFn>
    static bool read_all(ReadFn read_fn, Ina219Data& out) noexcept {
        uint16_t v, c, p;
        if (!read_fn(address, REG_BUS_V, &v)) return false;
        if (!read_fn(address, REG_CURRENT, &c)) return false;
        if (!read_fn(address, REG_POWER, &p)) return false;

        out.voltage_mv = static_cast<uint16_t>((v >> 3) * 4);
        out.current_ma = static_cast<int16_t>(static_cast<int16_t>(c) / 10);
        out.power_mw   = static_cast<uint16_t>(p * 2);
        return true;
    }
};

} // namespace drivers
