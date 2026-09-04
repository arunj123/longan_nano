#pragma once

#include <cstdint>
#include "hal/register.hpp"
#include "hal/rcu.hpp"

namespace hal::i2c {

enum class Direction : uint8_t {
    Transmitter = 0,
    Receiver    = 1
};

enum class DutyCycle : uint8_t {
    Ratio2    = 0,
    Ratio16_9 = 1
};

template <uintptr_t Base>
struct I2cPeripheral {
    static constexpr uintptr_t base = Base;

    using RegCTL0   = Register<Base + 0x00, uint32_t>;
    using RegCTL1   = Register<Base + 0x04, uint32_t>;
    using RegSADDR0 = Register<Base + 0x08, uint32_t>;
    using RegSADDR1 = Register<Base + 0x0C, uint32_t>;
    using RegDATA   = Register<Base + 0x10, uint32_t>;
    using RegSTAT0  = Register<Base + 0x14, uint32_t>;
    using RegSTAT1  = Register<Base + 0x18, uint32_t>;
    using RegCKCFG  = Register<Base + 0x1C, uint32_t>;
    using RegRT     = Register<Base + 0x20, uint32_t>;
    using RegFMPCFG = Register<Base + 0x90, uint32_t>;

    // CTL0 Bit Masks
    static constexpr uint32_t CTL0_I2CEN    = 1U << 0;
    static constexpr uint32_t CTL0_SMBEN    = 1U << 1;
    static constexpr uint32_t CTL0_SMBSEL   = 1U << 3;
    static constexpr uint32_t CTL0_ARPEN    = 1U << 4;
    static constexpr uint32_t CTL0_PECEN    = 1U << 5;
    static constexpr uint32_t CTL0_GCEN     = 1U << 6;
    static constexpr uint32_t CTL0_SS       = 1U << 7;
    static constexpr uint32_t CTL0_START    = 1U << 8;
    static constexpr uint32_t CTL0_STOP     = 1U << 9;
    static constexpr uint32_t CTL0_ACKEN    = 1U << 10;
    static constexpr uint32_t CTL0_POAP     = 1U << 11;
    static constexpr uint32_t CTL0_PECTRANS = 1U << 12;
    static constexpr uint32_t CTL0_SALT     = 1U << 13;
    static constexpr uint32_t CTL0_SRESET   = 1U << 15;

    // CTL1 Bit Masks
    static constexpr uint32_t CTL1_I2CCLK_MASK = 0x3FU;
    static constexpr uint32_t CTL1_ERRIE       = 1U << 8;
    static constexpr uint32_t CTL1_EVIE        = 1U << 9;
    static constexpr uint32_t CTL1_BUFIE       = 1U << 10;
    static constexpr uint32_t CTL1_DMAON       = 1U << 11;
    static constexpr uint32_t CTL1_DMALST      = 1U << 12;

    // STAT0 Bit Masks
    static constexpr uint32_t STAT0_SBSEND     = 1U << 0;
    static constexpr uint32_t STAT0_ADDSEND    = 1U << 1;
    static constexpr uint32_t STAT0_BTC        = 1U << 2;
    static constexpr uint32_t STAT0_ADD10SEND  = 1U << 3;
    static constexpr uint32_t STAT0_STPDET     = 1U << 4;
    static constexpr uint32_t STAT0_RBNE       = 1U << 6;
    static constexpr uint32_t STAT0_TBE        = 1U << 7;
    static constexpr uint32_t STAT0_BERR       = 1U << 8;
    static constexpr uint32_t STAT0_LOSTARB    = 1U << 9;
    static constexpr uint32_t STAT0_AERR       = 1U << 10;
    static constexpr uint32_t STAT0_OUERR      = 1U << 11;
    static constexpr uint32_t STAT0_PECERR     = 1U << 12;
    static constexpr uint32_t STAT0_SMBTO      = 1U << 14;
    static constexpr uint32_t STAT0_SMBALT     = 1U << 15;

    // STAT1 Bit Masks
    static constexpr uint32_t STAT1_MASTER     = 1U << 0;
    static constexpr uint32_t STAT1_I2CBSY     = 1U << 1;
    static constexpr uint32_t STAT1_TR         = 1U << 2;

    // CKCFG Bit Masks
    static constexpr uint32_t CKCFG_CLKC_MASK  = 0xFFFU;
    static constexpr uint32_t CKCFG_DTCY       = 1U << 14;
    static constexpr uint32_t CKCFG_FAST       = 1U << 15;

    static constexpr uint32_t kDefaultTimeout = 50000;

    /// Enable the I2C peripheral
    static inline void enable() noexcept {
        RegCTL0::set_bits(CTL0_I2CEN);
    }

    /// Disable the I2C peripheral
    static inline void disable() noexcept {
        RegCTL0::clear_bits(CTL0_I2CEN);
    }

    /// Enable or disable ACK generation
    static inline void set_ack(bool enable) noexcept {
        if (enable) {
            RegCTL0::set_bits(CTL0_ACKEN);
        } else {
            RegCTL0::clear_bits(CTL0_ACKEN);
        }
    }

    /// Generate START condition on the bus
    static inline void start() noexcept {
        RegCTL0::set_bits(CTL0_START);
    }

    /// Generate STOP condition on the bus
    static inline void stop() noexcept {
        RegCTL0::set_bits(CTL0_STOP);
    }

    /// Clear ADDSEND flag by reading STAT0 then STAT1
    static inline void clear_addsend() noexcept {
        [[maybe_unused]] volatile uint32_t s0 = RegSTAT0::read();
        [[maybe_unused]] volatile uint32_t s1 = RegSTAT1::read();
    }

    /// Wait for a bitmask in Reg to become set
    template <typename Reg>
    [[nodiscard]] static inline bool wait_set(uint32_t mask, uint32_t timeout = kDefaultTimeout) noexcept {
        while ((Reg::read() & mask) == 0) {
            if (--timeout == 0) return false;
        }
        return true;
    }

    /// Wait for a bitmask in Reg to become cleared
    template <typename Reg>
    [[nodiscard]] static inline bool wait_clear(uint32_t mask, uint32_t timeout = kDefaultTimeout) noexcept {
        while ((Reg::read() & mask) != 0) {
            if (--timeout == 0) return false;
        }
        return true;
    }

    /**
     * @brief Initialize the I2C peripheral clock and timing.
     * @param clkspeed Bus speed in Hz (e.g., 100000 for standard, 400000 for fast mode).
     * @param duty Duty cycle selection for fast mode.
     */
    static inline void init(uint32_t clkspeed = 100000, DutyCycle duty = DutyCycle::Ratio2) noexcept {
        const uint32_t pclk1 = hal::rcu::get_apb1_clock_frequency();

        // Calculate peripheral clock frequency in MHz (2 to 54 MHz for GD32VF103)
        uint32_t freq = pclk1 / 1000000U;
        if (freq > 54U) freq = 54U;
        if (freq < 2U)  freq = 2U;

        // Configure peripheral clock in CTL1
        RegCTL1::modify(CTL1_I2CCLK_MASK, freq);

        if (clkspeed <= 100000U) {
            // Standard mode: maximum rise time is 1000ns
            uint32_t risetime = (pclk1 / 1000000U) + 1U;
            if (risetime > 54U) risetime = 54U;
            if (risetime < 2U)  risetime = 2U;
            RegRT::write(risetime);

            uint32_t clkc = pclk1 / (clkspeed * 2U);
            if (clkc < 4U) clkc = 4U;
            RegCKCFG::write(clkc & CKCFG_CLKC_MASK);
        } else {
            // Fast mode: maximum rise time is 300ns
            uint32_t risetime = ((freq * 300U) / 1000U) + 1U;
            RegRT::write(risetime);

            uint32_t clkc = 0;
            uint32_t ckcfg = CKCFG_FAST;
            if (duty == DutyCycle::Ratio2) {
                clkc = pclk1 / (clkspeed * 3U);
            } else {
                clkc = pclk1 / (clkspeed * 25U);
                ckcfg |= CKCFG_DTCY;
            }
            if (clkc < 1U) clkc = 1U;
            ckcfg |= (clkc & CKCFG_CLKC_MASK);
            RegCKCFG::write(ckcfg);
        }

        // Configure 7-bit addressing, slave address 0
        RegCTL0::clear_bits(CTL0_SMBEN);
        RegSADDR0::write(0);

        // Enable peripheral and ACK
        enable();
        set_ack(true);
    }

    /**
     * @brief Perform robust 16-bit register write.
     * @param dev_addr 7-bit I2C device address.
     * @param reg_addr 8-bit internal register pointer.
     * @param value 16-bit value to write (transmitted MSB first).
     * @return true on success, false on bus error/timeout.
     */
    static inline bool write_reg16(uint8_t dev_addr, uint8_t reg_addr, uint16_t value) noexcept {
        // Wait until I2C bus is idle
        if (!wait_clear<RegSTAT1>(STAT1_I2CBSY)) return false;

        // Generate START condition
        start();
        if (!wait_set<RegSTAT0>(STAT0_SBSEND)) {
            stop();
            return false;
        }

        // Send 7-bit device address in transmitter mode
        RegDATA::write(static_cast<uint32_t>(dev_addr << 1));
        if (!wait_set<RegSTAT0>(STAT0_ADDSEND)) {
            stop();
            return false;
        }
        clear_addsend();

        // Send register address
        if (!wait_set<RegSTAT0>(STAT0_TBE)) { stop(); return false; }
        RegDATA::write(reg_addr);

        // Send MSB
        if (!wait_set<RegSTAT0>(STAT0_TBE)) { stop(); return false; }
        RegDATA::write(static_cast<uint32_t>(value >> 8));

        // Send LSB
        if (!wait_set<RegSTAT0>(STAT0_TBE)) { stop(); return false; }
        RegDATA::write(static_cast<uint32_t>(value & 0xFF));

        // Wait for byte transfer complete
        if (!wait_set<RegSTAT0>(STAT0_TBE)) { stop(); return false; }

        // Send STOP condition
        stop();
        return wait_clear<RegCTL0>(CTL0_STOP);
    }

    /**
     * @brief Perform robust 16-bit register read.
     * @param dev_addr 7-bit I2C device address.
     * @param reg_addr 8-bit internal register pointer.
     * @param value Pointer to store the 16-bit received value (MSB first).
     * @return true on success, false on bus error/timeout.
     */
    static inline bool read_reg16(uint8_t dev_addr, uint8_t reg_addr, uint16_t* value) noexcept {
        if (!value) return false;

        // Wait until I2C bus is idle
        if (!wait_clear<RegSTAT1>(STAT1_I2CBSY)) return false;

        // Generate START condition
        start();
        if (!wait_set<RegSTAT0>(STAT0_SBSEND)) {
            stop();
            return false;
        }

        // Send device address in transmitter mode
        RegDATA::write(static_cast<uint32_t>(dev_addr << 1));
        if (!wait_set<RegSTAT0>(STAT0_ADDSEND)) {
            stop();
            return false;
        }
        clear_addsend();

        // Send register address
        if (!wait_set<RegSTAT0>(STAT0_TBE)) { stop(); return false; }
        RegDATA::write(reg_addr);
        if (!wait_set<RegSTAT0>(STAT0_TBE)) { stop(); return false; }

        // Generate Repeated START
        start();
        if (!wait_set<RegSTAT0>(STAT0_SBSEND)) {
            stop();
            return false;
        }

        // Send device address in receiver mode
        RegDATA::write(static_cast<uint32_t>((dev_addr << 1) | 1U));
        if (!wait_set<RegSTAT0>(STAT0_ADDSEND)) {
            stop();
            return false;
        }
        clear_addsend();

        // Read MSB
        if (!wait_set<RegSTAT0>(STAT0_RBNE)) { stop(); return false; }
        const uint8_t msb = static_cast<uint8_t>(RegDATA::read() & 0xFFU);

        // Disable ACK before receiving the last byte
        set_ack(false);

        // Generate STOP condition
        stop();

        // Read LSB
        if (!wait_set<RegSTAT0>(STAT0_RBNE)) {
            set_ack(true);
            return false;
        }
        const uint8_t lsb = static_cast<uint8_t>(RegDATA::read() & 0xFFU);

        // Wait for STOP to clear
        const bool stop_ok = wait_clear<RegCTL0>(CTL0_STOP);

        // Re-enable ACK for subsequent transfers
        set_ack(true);

        if (!stop_ok) return false;

        *value = static_cast<uint16_t>((static_cast<uint16_t>(msb) << 8) | lsb);
        return true;
    }
};

// Typed peripheral instances
using I2c0 = I2cPeripheral<0x40005400>;
using I2c1 = I2cPeripheral<0x40005800>;

} // namespace hal::i2c
