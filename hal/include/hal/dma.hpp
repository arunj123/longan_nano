#pragma once

#include <cstdint>
#include "hal/register.hpp"

namespace hal::dma {

enum class Direction : uint8_t {
    PeripheralToMemory = 0,
    MemoryToPeripheral = 1
};

enum class Width : uint8_t {
    Bits8  = 0,
    Bits16 = 1,
    Bits32 = 2
};

enum class Priority : uint8_t {
    Low       = 0,
    Medium    = 1,
    High      = 2,
    UltraHigh = 3
};

template <uintptr_t DmaBase, uint8_t ChannelIndex>
struct DmaChannel {
    static_assert(ChannelIndex < 7, "GD32VF103 DMA supports channels 0 to 6");

    static constexpr uintptr_t dma_base = DmaBase;
    static constexpr uint8_t channel_idx = ChannelIndex;
    static constexpr uintptr_t ch_base = DmaBase + 0x08 + ChannelIndex * 0x14;

    using RegINTF  = Register<DmaBase + 0x00, uint32_t>;
    using RegINTC  = Register<DmaBase + 0x04, uint32_t>;
    using RegCTL   = Register<ch_base + 0x00, uint32_t>;
    using RegCNT   = Register<ch_base + 0x04, uint32_t>;
    using RegPADDR = Register<ch_base + 0x08, uint32_t>;
    using RegMADDR = Register<ch_base + 0x0C, uint32_t>;

    // CTL bit masks
    static constexpr uint32_t CTL_CHEN   = 1U << 0;
    static constexpr uint32_t CTL_FTFIE  = 1U << 1;
    static constexpr uint32_t CTL_HTFIE  = 1U << 2;
    static constexpr uint32_t CTL_ERRIE  = 1U << 3;
    static constexpr uint32_t CTL_DIR    = 1U << 4;
    static constexpr uint32_t CTL_CMEN   = 1U << 5;
    static constexpr uint32_t CTL_PNAGA  = 1U << 6;
    static constexpr uint32_t CTL_MNAGA  = 1U << 7;
    static constexpr uint32_t CTL_M2M    = 1U << 14;

    // Flag shifts in INTF / INTC
    static constexpr uint8_t FLAG_SHIFT  = ChannelIndex * 4;
    static constexpr uint32_t FLAG_GIF   = 1U << (FLAG_SHIFT + 0);
    static constexpr uint32_t FLAG_FTFIF = 1U << (FLAG_SHIFT + 1);
    static constexpr uint32_t FLAG_HTFIF = 1U << (FLAG_SHIFT + 2);
    static constexpr uint32_t FLAG_ERRIF = 1U << (FLAG_SHIFT + 3);
    static constexpr uint32_t FLAG_ALL   = 0xFU << FLAG_SHIFT;

    /// Enable DMA channel
    static inline void enable() noexcept {
        RegCTL::set_bits(CTL_CHEN);
    }

    /// Disable DMA channel
    static inline void disable() noexcept {
        RegCTL::clear_bits(CTL_CHEN);
    }

    [[nodiscard]] static inline bool is_enabled() noexcept {
        return (RegCTL::read() & CTL_CHEN) != 0;
    }

    /// Set peripheral address
    static inline void set_peripheral_address(uintptr_t addr) noexcept {
        RegPADDR::write(static_cast<uint32_t>(addr));
    }

    /// Set memory address
    static inline void set_memory_address(uintptr_t addr) noexcept {
        RegMADDR::write(static_cast<uint32_t>(addr));
    }

    /// Set number of transfers
    static inline void set_transfer_count(uint32_t count) noexcept {
        RegCNT::write(count);
    }

    /// Get remaining number of transfers
    [[nodiscard]] static inline uint32_t get_transfer_count() noexcept {
        return RegCNT::read();
    }

    [[nodiscard]] static inline bool is_busy() noexcept {
        return get_transfer_count() != 0;
    }

    /// Wait until transfer count reaches zero
    static inline void wait_complete() noexcept {
        while (is_busy());
    }

    /// Check if Full Transfer Finish flag is set
    [[nodiscard]] static inline bool is_transfer_complete() noexcept {
        return (RegINTF::read() & FLAG_FTFIF) != 0;
    }

    /// Clear transfer finish flag
    static inline void clear_transfer_complete() noexcept {
        RegINTC::write(FLAG_FTFIF);
    }

    /// Clear all channel interrupt flags
    static inline void clear_all_flags() noexcept {
        RegINTC::write(FLAG_ALL);
    }

    /// Enable transfer finish interrupt
    static inline void enable_interrupt() noexcept {
        RegCTL::set_bits(CTL_FTFIE);
    }

    /// Disable transfer finish interrupt
    static inline void disable_interrupt() noexcept {
        RegCTL::clear_bits(CTL_FTFIE);
    }

    /// Configure for Memory-to-Peripheral (TX)
    static inline void configure_tx(uintptr_t periph_addr,
                                    uintptr_t mem_addr,
                                    uint32_t count,
                                    Width width = Width::Bits8,
                                    bool memory_increment = true,
                                    Priority prio = Priority::UltraHigh) noexcept {
        disable();
        clear_all_flags();

        set_peripheral_address(periph_addr);
        set_memory_address(mem_addr);
        set_transfer_count(count);

        uint32_t ctl = CTL_DIR; // Memory to Peripheral
        if (memory_increment) ctl |= CTL_MNAGA;
        ctl |= (static_cast<uint32_t>(width) << 8);  // PWIDTH
        ctl |= (static_cast<uint32_t>(width) << 10); // MWIDTH
        ctl |= (static_cast<uint32_t>(prio) << 12);  // PRIO

        RegCTL::write(ctl);
    }

    /// Configure for Peripheral-to-Memory (RX)
    static inline void configure_rx(uintptr_t periph_addr,
                                    uintptr_t mem_addr,
                                    uint32_t count,
                                    Width width = Width::Bits8,
                                    bool memory_increment = true,
                                    Priority prio = Priority::UltraHigh) noexcept {
        disable();
        clear_all_flags();

        set_peripheral_address(periph_addr);
        set_memory_address(mem_addr);
        set_transfer_count(count);

        uint32_t ctl = 0; // Peripheral to Memory
        if (memory_increment) ctl |= CTL_MNAGA;
        ctl |= (static_cast<uint32_t>(width) << 8);  // PWIDTH
        ctl |= (static_cast<uint32_t>(width) << 10); // MWIDTH
        ctl |= (static_cast<uint32_t>(prio) << 12);  // PRIO

        RegCTL::write(ctl);
    }
};

// Common GD32VF103 DMA Channel instantiations
using Dma0Channel0 = DmaChannel<0x40020000, 0>;
using Dma0Channel1 = DmaChannel<0x40020000, 1>; // Hardwired: SPI0_RX
using Dma0Channel2 = DmaChannel<0x40020000, 2>; // Hardwired: SPI0_TX
using Dma0Channel3 = DmaChannel<0x40020000, 3>; // Hardwired: SPI1_RX / USART0_TX
using Dma0Channel4 = DmaChannel<0x40020000, 4>; // Hardwired: SPI1_TX / USART0_RX
using Dma0Channel5 = DmaChannel<0x40020000, 5>; // Hardwired: I2C0_TX
using Dma0Channel6 = DmaChannel<0x40020000, 6>; // Hardwired: I2C0_RX

} // namespace hal::dma
