#pragma once

#include <cstdint>
#include "hal/register.hpp"

namespace hal::rcu {

inline constexpr uintptr_t kRcuBase = 0x40021000;

using RegCTL     = Register<kRcuBase + 0x00, uint32_t>;
using RegCFG0    = Register<kRcuBase + 0x04, uint32_t>;
using RegINT     = Register<kRcuBase + 0x08, uint32_t>;
using RegAPB2RST = Register<kRcuBase + 0x0C, uint32_t>;
using RegAPB1RST = Register<kRcuBase + 0x10, uint32_t>;
using RegAHBEN   = Register<kRcuBase + 0x14, uint32_t>;
using RegAPB2EN  = Register<kRcuBase + 0x18, uint32_t>;
using RegAPB1EN  = Register<kRcuBase + 0x1C, uint32_t>;
using RegBDCTL   = Register<kRcuBase + 0x20, uint32_t>;
using RegRSTSCK  = Register<kRcuBase + 0x24, uint32_t>;
using RegAHBRST  = Register<kRcuBase + 0x28, uint32_t>;
using RegCFG1    = Register<kRcuBase + 0x2C, uint32_t>;

// Peripheral identifiers with embedded register offset and bit position
enum class Peripheral : uint32_t {
    // AHB peripherals (high word = 0x14 for AHBEN)
    Dma0    = (0x14U << 16) | 0,
    Dma1    = (0x14U << 16) | 1,
    Crc     = (0x14U << 16) | 6,
    Exmc    = (0x14U << 16) | 8,
    Usbfs   = (0x14U << 16) | 12,

    // APB2 peripherals (high word = 0x18 for APB2EN)
    Afio    = (0x18U << 16) | 0,
    GpioA   = (0x18U << 16) | 2,
    GpioB   = (0x18U << 16) | 3,
    GpioC   = (0x18U << 16) | 4,
    GpioD   = (0x18U << 16) | 5,
    GpioE   = (0x18U << 16) | 6,
    Adc0    = (0x18U << 16) | 9,
    Adc1    = (0x18U << 16) | 10,
    Timer0  = (0x18U << 16) | 11,
    Spi0    = (0x18U << 16) | 12,
    Usart0  = (0x18U << 16) | 14,

    // APB1 peripherals (high word = 0x1CU for APB1EN)
    Timer1  = (0x1CU << 16) | 0,
    Timer2  = (0x1CU << 16) | 1,
    Timer3  = (0x1CU << 16) | 2,
    Timer4  = (0x1CU << 16) | 3,
    Timer5  = (0x1CU << 16) | 4,
    Timer6  = (0x1CU << 16) | 5,
    Wwdgt   = (0x1CU << 16) | 11,
    Spi1    = (0x1CU << 16) | 14,
    Spi2    = (0x1CU << 16) | 15,
    Usart1  = (0x1CU << 16) | 17,
    Usart2  = (0x1CU << 16) | 18,
    Uart3   = (0x1CU << 16) | 19,
    Uart4   = (0x1CU << 16) | 20,
    I2c0    = (0x1CU << 16) | 21,
    I2c1    = (0x1CU << 16) | 22,
    Can0    = (0x1CU << 16) | 25,
    Can1    = (0x1CU << 16) | 26,
    Bkpi    = (0x1CU << 16) | 27,
    Pmu     = (0x1CU << 16) | 28,
    Dac     = (0x1CU << 16) | 29
};

/**
 * @brief Enable clock for a peripheral.
 */
inline void enable(Peripheral p) noexcept {
    const uint32_t val = static_cast<uint32_t>(p);
    const uint32_t reg_offset = (val >> 16) & 0xFFFFU;
    const uint32_t bit = val & 0x1FU;
    const uintptr_t reg_addr = kRcuBase + reg_offset;
    *reinterpret_cast<volatile uint32_t*>(reg_addr) |= (1U << bit);
}

/**
 * @brief Disable clock for a peripheral.
 */
inline void disable(Peripheral p) noexcept {
    const uint32_t val = static_cast<uint32_t>(p);
    const uint32_t reg_offset = (val >> 16) & 0xFFFFU;
    const uint32_t bit = val & 0x1FU;
    const uintptr_t reg_addr = kRcuBase + reg_offset;
    *reinterpret_cast<volatile uint32_t*>(reg_addr) &= ~(1U << bit);
}

/**
 * @brief Check if clock is enabled for a peripheral.
 */
[[nodiscard]] inline bool is_enabled(Peripheral p) noexcept {
    const uint32_t val = static_cast<uint32_t>(p);
    const uint32_t reg_offset = (val >> 16) & 0xFFFFU;
    const uint32_t bit = val & 0x1FU;
    const uintptr_t reg_addr = kRcuBase + reg_offset;
    return (*reinterpret_cast<volatile uint32_t*>(reg_addr) & (1U << bit)) != 0;
}

/**
 * @brief Reset an APB1 or APB2 peripheral (pulse reset bit high then low).
 */
inline void reset(Peripheral p) noexcept {
    const uint32_t val = static_cast<uint32_t>(p);
    const uint32_t reg_offset = (val >> 16) & 0xFFFFU;
    const uint32_t bit = val & 0x1FU;

    // Determine reset register offset based on enable register offset
    uint32_t rst_offset = 0;
    if (reg_offset == 0x18U) { // APB2EN -> APB2RST is 0x0CU
        rst_offset = 0x0CU;
    } else if (reg_offset == 0x1CU) { // APB1EN -> APB1RST is 0x10U
        rst_offset = 0x10U;
    } else if (reg_offset == 0x14U) { // AHBEN -> AHBRST is 0x28U
        rst_offset = 0x28U;
    } else {
        return;
    }

    const uintptr_t rst_addr = kRcuBase + rst_offset;
    *reinterpret_cast<volatile uint32_t*>(rst_addr) |= (1U << bit);
    *reinterpret_cast<volatile uint32_t*>(rst_addr) &= ~(1U << bit);
}

/**
 * @brief Configure the USBFS prescaler in CFG0.
 */
inline void set_usb_clock_prescaler(uint32_t psc_bits) noexcept {
    constexpr uint32_t kUsbfsPscMask = 0x3U << 22;
    RegCFG0::modify(kUsbfsPscMask, psc_bits);
}

extern "C" {
extern uint32_t SystemCoreClock;
}

/**
 * @brief Get the APB1 peripheral bus clock frequency in Hz.
 */
[[nodiscard]] inline uint32_t get_apb1_clock_frequency() noexcept {
    const uint32_t psc_bits = (RegCFG0::read() >> 8) & 0x7U;
    if ((psc_bits & 0x4U) == 0) {
        return SystemCoreClock;
    }
    const uint8_t shift = static_cast<uint8_t>((psc_bits & 0x3U) + 1);
    return SystemCoreClock >> shift;
}

/**
 * @brief Get the APB2 peripheral bus clock frequency in Hz.
 */
[[nodiscard]] inline uint32_t get_apb2_clock_frequency() noexcept {
    const uint32_t psc_bits = (RegCFG0::read() >> 11) & 0x7U;
    if ((psc_bits & 0x4U) == 0) {
        return SystemCoreClock;
    }
    const uint8_t shift = static_cast<uint8_t>((psc_bits & 0x3U) + 1);
    return SystemCoreClock >> shift;
}

} // namespace hal::rcu
