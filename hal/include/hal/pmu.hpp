#pragma once

#include <cstdint>
#include "hal/register.hpp"
#include "hal/core.hpp"

namespace hal::pmu {

inline constexpr uintptr_t kPmuBase = 0x40007000;

using RegCTL = Register<kPmuBase + 0x00, uint32_t>;
using RegCS  = Register<kPmuBase + 0x04, uint32_t>;

// CTL Bit Masks
static constexpr uint32_t CTL_LDOLP   = 1U << 0;  // LDO low power in deep sleep
static constexpr uint32_t CTL_STBMOD  = 1U << 1;  // Standby mode
static constexpr uint32_t CTL_WURST   = 1U << 2;  // Wakeup flag reset
static constexpr uint32_t CTL_STBRST  = 1U << 3;  // Standby flag reset
static constexpr uint32_t CTL_LVDEN   = 1U << 4;  // Low voltage detector enable
static constexpr uint32_t CTL_BKPWEN  = 1U << 8;  // Backup domain write enable

// CS Bit Masks
static constexpr uint32_t CS_WUF      = 1U << 0;  // Wakeup flag
static constexpr uint32_t CS_STBF     = 1U << 1;  // Standby flag
static constexpr uint32_t CS_LVDF     = 1U << 2;  // Low voltage detector status
static constexpr uint32_t CS_WUPEN    = 1U << 8;  // Wakeup pin enable

enum class LdoMode : uint8_t {
    Normal   = 0,
    LowPower = 1
};

enum class SleepCommand : uint8_t {
    Wfi = 0,
    Wfe = 1
};

struct Pmu {
    /// Enable or disable write access to the backup domain (BKP, RTC).
    static inline void enable_backup_write(bool enable = true) noexcept {
        if (enable) {
            RegCTL::set_bits(CTL_BKPWEN);
        } else {
            RegCTL::clear_bits(CTL_BKPWEN);
        }
    }

    /// Enable or disable the wakeup pin (PA0).
    static inline void enable_wakeup_pin(bool enable = true) noexcept {
        if (enable) {
            RegCS::set_bits(CS_WUPEN);
        } else {
            RegCS::clear_bits(CS_WUPEN);
        }
    }

    /// Clear wakeup flag.
    static inline void clear_wakeup_flag() noexcept {
        RegCTL::set_bits(CTL_WURST);
    }

    /// Clear standby flag.
    static inline void clear_standby_flag() noexcept {
        RegCTL::set_bits(CTL_STBRST);
    }

    /// Enter sleep mode (CPU clock stopped, peripherals running).
    static inline void to_sleep_mode(SleepCommand cmd = SleepCommand::Wfi) noexcept {
        // Clear CSR_SLEEPVALUE bit 0 for normal sleep
        asm volatile("csrc 0x811, 0x1" : : : "memory");

        if (cmd == SleepCommand::Wfi) {
            core::wfi();
        } else {
            [[maybe_unused]] const auto prev = core::disable_interrupts();
            asm volatile("csrs 0x810, 0x1" : : : "memory");
            core::wfi();
            asm volatile("csrc 0x810, 0x1" : : : "memory");
            core::enable_interrupts();
        }
    }

    /// Enter deepsleep mode (all 1.2V domain clocks stopped).
    static inline void to_deepsleep_mode(LdoMode ldo = LdoMode::LowPower,
                                        SleepCommand cmd = SleepCommand::Wfi) noexcept {
        // Clear STBMOD and LDOLP bits
        RegCTL::clear_bits(CTL_STBMOD | CTL_LDOLP);

        // Configure LDO mode
        if (ldo == LdoMode::LowPower) {
            RegCTL::set_bits(CTL_LDOLP);
        }

        // Set CSR_SLEEPVALUE bit 0 for deep sleep
        asm volatile("csrs 0x811, 0x1" : : : "memory");

        if (cmd == SleepCommand::Wfi) {
            core::wfi();
        } else {
            [[maybe_unused]] const auto prev = core::disable_interrupts();
            asm volatile("csrs 0x810, 0x1" : : : "memory");
            core::wfi();
            asm volatile("csrc 0x810, 0x1" : : : "memory");
            core::enable_interrupts();
        }

        // Clear CSR_SLEEPVALUE bit 0 upon wakeup
        asm volatile("csrc 0x811, 0x1" : : : "memory");
    }

    /// Enter standby mode (1.2V domain completely powered down).
    static inline void to_standby_mode(SleepCommand cmd = SleepCommand::Wfi) noexcept {
        // Set CSR_SLEEPVALUE bit 0
        asm volatile("csrs 0x811, 0x1" : : : "memory");

        // Set STBMOD and reset wakeup flag
        RegCTL::set_bits(CTL_STBMOD | CTL_WURST);

        if (cmd == SleepCommand::Wfi) {
            core::wfi();
        } else {
            [[maybe_unused]] const auto prev = core::disable_interrupts();
            asm volatile("csrs 0x810, 0x1" : : : "memory");
            core::wfi();
            asm volatile("csrc 0x810, 0x1" : : : "memory");
            core::enable_interrupts();
        }
    }
};

} // namespace hal::pmu

// C compatibility functions for legacy / USB library callers
extern "C" {

void pmu_to_deepsleepmode(uint32_t ldo, uint8_t deepsleepmodecmd);

} // extern "C"
