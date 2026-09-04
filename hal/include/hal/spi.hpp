#pragma once

#include <cstdint>
#include "hal/register.hpp"

namespace hal::spi {

enum class Prescaler : uint8_t {
    Div2   = 0,
    Div4   = 1,
    Div8   = 2,
    Div16  = 3,
    Div32  = 4,
    Div64  = 5,
    Div128 = 6,
    Div256 = 7
};

enum class FrameFormat : uint8_t {
    Bits8  = 0,
    Bits16 = 1
};

enum class ClockPolarity : uint8_t {
    Low  = 0,
    High = 1
};

enum class ClockPhase : uint8_t {
    Edge1 = 0,
    Edge2 = 1
};

template <uintptr_t Base>
struct SpiPeripheral {
    static constexpr uintptr_t base = Base;

    using RegCTL0 = Register<Base + 0x00, uint32_t>;
    using RegCTL1 = Register<Base + 0x04, uint32_t>;
    using RegSTAT = Register<Base + 0x08, uint32_t>;
    using RegDATA = Register<Base + 0x0C, uint32_t>;

    // Bit masks for CTL0
    static constexpr uint32_t CTL0_CKPH     = 1U << 0;
    static constexpr uint32_t CTL0_CKPL     = 1U << 1;
    static constexpr uint32_t CTL0_MSTMOD   = 1U << 2;
    static constexpr uint32_t CTL0_PSC_MASK = 7U << 3;
    static constexpr uint32_t CTL0_SPIEN    = 1U << 6;
    static constexpr uint32_t CTL0_LF       = 1U << 7;
    static constexpr uint32_t CTL0_SWNSS    = 1U << 8;
    static constexpr uint32_t CTL0_SWNSSEN  = 1U << 9;
    static constexpr uint32_t CTL0_RO       = 1U << 10;
    static constexpr uint32_t CTL0_FF16     = 1U << 11;

    // Bit masks for CTL1
    static constexpr uint32_t CTL1_DMAREN   = 1U << 0;
    static constexpr uint32_t CTL1_DMATEN   = 1U << 1;

    // Bit masks for STAT
    static constexpr uint32_t STAT_RBNE     = 1U << 0;
    static constexpr uint32_t STAT_TBE      = 1U << 1;
    static constexpr uint32_t STAT_TRANS    = 1U << 7;

    [[nodiscard]] static constexpr uintptr_t data_register_address() noexcept {
        return Base + 0x0C;
    }

    /// Enable the SPI peripheral
    static inline void enable() noexcept {
        RegCTL0::set_bits(CTL0_SPIEN);
    }

    /// Disable the SPI peripheral
    static inline void disable() noexcept {
        RegCTL0::clear_bits(CTL0_SPIEN);
    }

    [[nodiscard]] static inline bool is_enabled() noexcept {
        return (RegCTL0::read() & CTL0_SPIEN) != 0;
    }

    /// Initialize as SPI Master with software NSS
    static inline void init_master(Prescaler psc = Prescaler::Div8,
                                  ClockPolarity cpol = ClockPolarity::Low,
                                  ClockPhase cpha = ClockPhase::Edge1,
                                  FrameFormat fmt = FrameFormat::Bits8) noexcept {
        disable();
        uint32_t ctl0 = CTL0_MSTMOD | CTL0_SWNSS | CTL0_SWNSSEN;
        ctl0 |= static_cast<uint32_t>(psc) << 3;
        if (cpol == ClockPolarity::High) ctl0 |= CTL0_CKPL;
        if (cpha == ClockPhase::Edge2)  ctl0 |= CTL0_CKPH;
        if (fmt == FrameFormat::Bits16)  ctl0 |= CTL0_FF16;
        RegCTL0::write(ctl0);
        RegCTL1::write(0);
        enable();
    }

    /// Set 8-bit frame format dynamically (re-enables SPI if currently enabled)
    static inline void set_8bit() noexcept {
        if (RegCTL0::read() & CTL0_FF16) {
            RegCTL0::modify(CTL0_SPIEN | CTL0_FF16, 0);
            RegCTL0::set_bits(CTL0_SPIEN);
        }
    }

    /// Set 16-bit frame format dynamically (re-enables SPI if currently enabled)
    static inline void set_16bit() noexcept {
        if (!(RegCTL0::read() & CTL0_FF16)) {
            RegCTL0::clear_bits(CTL0_SPIEN);
            RegCTL0::set_bits(CTL0_FF16 | CTL0_SPIEN);
        }
    }

    /// Dynamically change prescaler
    static inline void set_prescaler(Prescaler psc) noexcept {
        bool was_enabled = is_enabled();
        if (was_enabled) disable();
        RegCTL0::modify(CTL0_PSC_MASK, static_cast<uint32_t>(psc) << 3);
        if (was_enabled) enable();
    }

    /// Clear receive buffer / FIFO
    static inline void flush_rx() noexcept {
        while ((RegSTAT::read() & STAT_RBNE) != 0) {
            (void)read_raw();
        }
    }

    /// Wait until SPI is completely idle (no transmission ongoing)
    static inline void wait_idle() noexcept {
        while ((RegSTAT::read() & STAT_TRANS) != 0);
    }

    /// Wait until Transmit Buffer is Empty
    static inline void wait_tbe() noexcept {
        while ((RegSTAT::read() & STAT_TBE) == 0);
    }

    /// Wait until Receive Buffer is Not Empty
    static inline void wait_rbne() noexcept {
        while ((RegSTAT::read() & STAT_RBNE) == 0);
    }

    /// Send 8-bit data (waits for TBE)
    static inline void send_8(uint8_t data) noexcept {
        wait_tbe();
        RegDATA::write(data);
    }

    /// Send 16-bit data (waits for TBE)
    static inline void send_16(uint16_t data) noexcept {
        wait_tbe();
        RegDATA::write(data);
    }

    /// Read data from receive buffer
    [[nodiscard]] static inline uint16_t read_raw() noexcept {
        return static_cast<uint16_t>(RegDATA::read());
    }

    /// Full duplex 8-bit transfer
    static inline uint8_t transfer_8(uint8_t data) noexcept {
        send_8(data);
        wait_rbne();
        return static_cast<uint8_t>(read_raw() & 0xFF);
    }

    /// Full duplex 16-bit transfer
    static inline uint16_t transfer_16(uint16_t data) noexcept {
        send_16(data);
        wait_rbne();
        return read_raw();
    }

    /// Enable DMA Transmit trigger
    static inline void enable_dma_tx() noexcept {
        RegCTL1::set_bits(CTL1_DMATEN);
    }

    /// Disable DMA Transmit trigger
    static inline void disable_dma_tx() noexcept {
        RegCTL1::clear_bits(CTL1_DMATEN);
    }

    /// Enable DMA Receive trigger
    static inline void enable_dma_rx() noexcept {
        RegCTL1::set_bits(CTL1_DMAREN);
    }

    /// Disable DMA Receive trigger
    static inline void disable_dma_rx() noexcept {
        RegCTL1::clear_bits(CTL1_DMAREN);
    }
};

using Spi0 = SpiPeripheral<0x40013000>;
using Spi1 = SpiPeripheral<0x40003800>;
using Spi2 = SpiPeripheral<0x40003C00>;

} // namespace hal::spi
