#pragma once

#include <cstdint>
#include <concepts>
#include <utility>
#include <compare>
#include "hal/time.hpp"

namespace drivers {

/**
 * @brief Measurement snapshot from INA219.
 * Uses integer fixed-point arithmetic with defaulted C++20/23 comparisons.
 */
struct Ina219Data {
    uint16_t voltage_mv{0};
    int16_t  current_ma{0};
    uint16_t power_mw{0};

    constexpr auto operator<=>(const Ina219Data&) const noexcept = default;
    constexpr bool operator==(const Ina219Data&) const noexcept = default;
};

/**
 * @brief Concepts constraining I2C read and write operations.
 */
template <typename Fn>
concept I2cWriteFn = requires(Fn fn, uint8_t addr, uint8_t reg, uint16_t val) {
    { fn(addr, reg, val) } -> std::convertible_to<bool>;
};

template <typename Fn>
concept I2cReadFn = requires(Fn fn, uint8_t addr, uint8_t reg, uint16_t* val) {
    { fn(addr, reg, val) } -> std::convertible_to<bool>;
};

/**
 * @brief Modern C++23 fixed-point INA219 driver.
 * Strictly uses integer arithmetic (mV, mA, mW) with zero software float emulation.
 */
template <uint8_t I2cAddress = 0x40>
class Ina219 {
public:
    static constexpr uint8_t address = I2cAddress;

    // INA219 Hardware Registers
    enum class Register : uint8_t {
        Config       = 0x00,
        ShuntVoltage = 0x01,
        BusVoltage   = 0x02,
        Power        = 0x03,
        Current      = 0x04,
        Calibration  = 0x05,
    };

    // Legacy register constant definitions for backwards compatibility
    static constexpr uint8_t REG_CONFIG      = std::to_underlying(Register::Config);
    static constexpr uint8_t REG_SHUNT_V     = std::to_underlying(Register::ShuntVoltage);
    static constexpr uint8_t REG_BUS_V       = std::to_underlying(Register::BusVoltage);
    static constexpr uint8_t REG_POWER       = std::to_underlying(Register::Power);
    static constexpr uint8_t REG_CURRENT     = std::to_underlying(Register::Current);
    static constexpr uint8_t REG_CALIBRATION = std::to_underlying(Register::Calibration);

    /**
     * @brief Initialize INA219 with shunt calibration.
     * @param write_fn Callable satisfying I2cWriteFn.
     * @param cal Calibration value (default 4096 for 0.1 ohm shunt and 3.2A max range).
     */
    template <I2cWriteFn WriteFn>
    static bool init(WriteFn write_fn, uint16_t cal = 4096) noexcept {
        return write_fn(address, std::to_underlying(Register::Calibration), cal);
    }

    /**
     * @brief Read Bus Voltage in millivolts.
     */
    template <I2cReadFn ReadFn>
    static bool read_voltage_mv(ReadFn read_fn, uint16_t& out_mv) noexcept {
        uint16_t raw{0};
        if (!read_fn(address, std::to_underlying(Register::BusVoltage), &raw)) return false;
        // Bus voltage is in bits 3-15; 4mV per LSB
        out_mv = static_cast<uint16_t>((raw >> 3) * 4);
        return true;
    }

    /**
     * @brief Read Current in milliamperes (signed).
     */
    template <I2cReadFn ReadFn>
    static bool read_current_ma(ReadFn read_fn, int16_t& out_ma) noexcept {
        uint16_t raw{0};
        if (!read_fn(address, std::to_underlying(Register::Current), &raw)) return false;
        // Cal = 4096 with 0.1 ohm shunt: 1 LSB = 100uA = 0.1mA -> divide by 10
        out_ma = static_cast<int16_t>(static_cast<int16_t>(raw) / 10);
        return true;
    }

    /**
     * @brief Read Power in milliwatts.
     */
    template <I2cReadFn ReadFn>
    static bool read_power_mw(ReadFn read_fn, uint16_t& out_mw) noexcept {
        uint16_t raw{0};
        if (!read_fn(address, std::to_underlying(Register::Power), &raw)) return false;
        // Power LSB = 20 * Current LSB = 2mW
        out_mw = static_cast<uint16_t>(raw * 2);
        return true;
    }

    /**
     * @brief Read all measurements into an Ina219Data struct.
     */
    template <I2cReadFn ReadFn>
    static bool read_all(ReadFn read_fn, Ina219Data& out) noexcept {
        uint16_t v{0}, c{0}, p{0};
        if (!read_fn(address, std::to_underlying(Register::BusVoltage), &v)) return false;
        if (!read_fn(address, std::to_underlying(Register::Current), &c)) return false;
        if (!read_fn(address, std::to_underlying(Register::Power), &p)) return false;

        out.voltage_mv = static_cast<uint16_t>((v >> 3) * 4);
        out.current_ma = static_cast<int16_t>(static_cast<int16_t>(c) / 10);
        out.power_mw   = static_cast<uint16_t>(p * 2);
        return true;
    }
};

} // namespace drivers
