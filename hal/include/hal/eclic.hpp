#pragma once

#include <cstdint>
#include "hal/core.hpp"
#include "hal/register.hpp"

namespace hal::eclic {

namespace detail {
    inline constexpr uintptr_t kEclicBase = 0xD2000000;
    inline constexpr uintptr_t kEclicCfg  = kEclicBase + 0x0000;
    inline constexpr uintptr_t kEclicInfo = kEclicBase + 0x0004;
    inline constexpr uintptr_t kEclicMth  = kEclicBase + 0x000B;
    inline constexpr uintptr_t kEclicInts = kEclicBase + 0x1000;

    inline constexpr uint32_t kNumInterrupts = 87;
    inline constexpr uint8_t  kEclicIntCtlBits = 4;

    struct alignas(4) EclicIntEntry {
        volatile uint8_t ip;
        volatile uint8_t ie;
        volatile uint8_t attr;
        volatile uint8_t ctl;
    };

    static_assert(sizeof(EclicIntEntry) == 4, "EclicIntEntry must be exactly 4 bytes");

    inline volatile EclicIntEntry* int_entries() noexcept {
        return reinterpret_cast<volatile EclicIntEntry*>(kEclicInts);
    }
} // namespace detail

enum class Trigger : uint8_t {
    Level        = 0x00,
    EdgePositive = 0x02, // Rising edge
    EdgeNegative = 0x06  // Falling edge
};

enum class PriorityGroup : uint8_t {
    Level0Prio4 = 0, // 0 bits level, 4 bits priority
    Level1Prio3 = 1, // 1 bit level, 3 bits priority
    Level2Prio2 = 2, // 2 bits level, 2 bits priority
    Level3Prio1 = 3, // 3 bits level, 1 bit priority
    Level4Prio0 = 4  // 4 bits level, 0 bits priority
};

/**
 * @brief Zero-overhead compile-time ECLIC driver for GD32VF103.
 * Adheres strictly to GEMINI.md: Direct array indexing base[i] for all 87 interrupts.
 */
struct Eclic {
    static constexpr uint32_t NumInterrupts = detail::kNumInterrupts;

    /// Initialize ECLIC controller, resetting all 87 interrupts to disabled, non-vectored state.
    static inline void init(uint32_t num_irqs = detail::kNumInterrupts) noexcept {
        // Global configuration register: clear nlbits
        *reinterpret_cast<volatile uint8_t*>(detail::kEclicCfg) = 0;

        // Machine interrupt threshold: allow all levels
        *reinterpret_cast<volatile uint8_t*>(detail::kEclicMth) = 0;

        // Direct array indexing: reset all interrupts
        volatile uint32_t* base = reinterpret_cast<volatile uint32_t*>(detail::kEclicInts);
        for (uint32_t i = 0; i < num_irqs && i < detail::kNumInterrupts; ++i) {
            base[i] = 0;
        }
    }

    /// Put RISC-V core MTVEC CSR into ECLIC mode (bits 1:0 = 0b11).
    static inline void enable_mode() noexcept {
        uint32_t mtvec;
        asm volatile("csrr %0, mtvec" : "=r"(mtvec) : : "memory");
        mtvec = (mtvec & ~0x3FU) | 0x03U;
        asm volatile("csrw mtvec, %0" : : "r"(mtvec) : "memory");
    }

    /// Set ECLIC priority group (number of level bits).
    static inline void set_priority_group(PriorityGroup group) noexcept {
        volatile uint8_t* cfg = reinterpret_cast<volatile uint8_t*>(detail::kEclicCfg);
        *cfg = static_cast<uint8_t>((*cfg & 0xE1U) | (static_cast<uint8_t>(group) << 1));
    }

    /// Get current nlbits (number of level bits).
    [[nodiscard]] static inline uint8_t nlbits() noexcept {
        const uint8_t cfg = *reinterpret_cast<volatile uint8_t*>(detail::kEclicCfg);
        return (cfg & 0x1EU) >> 1;
    }

    /// Enable interrupt request with level, priority, and vector mode.
    /// GD32VF103 standard C handlers require non-vectored mode (vectored = false)
    /// to dispatch properly through the Nuclei irq_entry wrapper (CSR_JALMNXTI).
    static inline void enable(uint32_t irq, uint8_t level = 1, uint8_t priority = 0, bool vectored = false) noexcept {
        if (irq >= detail::kNumInterrupts) return;

        auto* const entry = &detail::int_entries()[irq];

        // Configure attribute: bit 0 = SHV (Selective Hardware Vector)
        entry->attr = vectored ? 0x01U : 0x00U;

        // Configure level and priority control register
        const uint8_t nl = (nlbits() > detail::kEclicIntCtlBits) ? detail::kEclicIntCtlBits : nlbits();
        uint8_t ctl = 0;
        if (nl > 0) {
            ctl |= static_cast<uint8_t>(level << (8 - nl));
        }
        if (nl < detail::kEclicIntCtlBits) {
            ctl |= static_cast<uint8_t>(priority << (8 - detail::kEclicIntCtlBits));
        }
        entry->ctl = ctl;

        // Clear any pending flag and enable
        entry->ip = 0;
        entry->ie = 1;
    }

    /// Disable interrupt request.
    static inline void disable(uint32_t irq) noexcept {
        if (irq < detail::kNumInterrupts) {
            detail::int_entries()[irq].ie = 0;
        }
    }

    /// Check if interrupt is pending.
    [[nodiscard]] static inline bool is_pending(uint32_t irq) noexcept {
        if (irq >= detail::kNumInterrupts) return false;
        return (detail::int_entries()[irq].ip & 0x01U) != 0;
    }

    /// Clear pending flag.
    static inline void clear_pending(uint32_t irq) noexcept {
        if (irq < detail::kNumInterrupts) {
            detail::int_entries()[irq].ip = 0;
        }
    }

    /// Set pending flag (trigger software interrupt).
    static inline void set_pending(uint32_t irq) noexcept {
        if (irq < detail::kNumInterrupts) {
            detail::int_entries()[irq].ip = 1;
        }
    }

    /// Set trigger sensitivity (Level, Edge Positive, Edge Negative).
    static inline void set_trigger(uint32_t irq, Trigger trigger) noexcept {
        if (irq < detail::kNumInterrupts) {
            auto* const entry = &detail::int_entries()[irq];
            entry->attr = static_cast<uint8_t>((entry->attr & 0xF9U) | static_cast<uint8_t>(trigger));
        }
    }

    /// Globally enable interrupts via MSTATUS.MIE.
    static inline void enable_global_interrupts() noexcept {
        core::enable_interrupts();
    }

    /// Globally disable interrupts via MSTATUS.MIE.
    static inline void disable_global_interrupts() noexcept {
        (void)core::disable_interrupts();
    }
};

} // namespace hal::eclic
