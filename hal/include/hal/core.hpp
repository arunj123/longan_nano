#pragma once

#include <cstdint>

namespace hal::core {

/// Read a RISC-V Control and Status Register (CSR).
#define HAL_READ_CSR(reg) ({ \
    uint32_t __val; \
    asm volatile("csrr %0, " #reg : "=r"(__val) : : "memory"); \
    __val; \
})

/// Write a RISC-V Control and Status Register (CSR).
#define HAL_WRITE_CSR(reg, val) ({ \
    asm volatile("csrw " #reg ", %0" : : "r"(val) : "memory"); \
})

/// Set bit(s) in a RISC-V Control and Status Register (CSR).
#define HAL_SET_CSR(reg, bit) ({ \
    asm volatile("csrs " #reg ", %0" : : "r"(bit) : "memory"); \
})

/// Clear bit(s) in a RISC-V Control and Status Register (CSR).
#define HAL_CLEAR_CSR(reg, bit) ({ \
    asm volatile("csrc " #reg ", %0" : : "r"(bit) : "memory"); \
})

/// Atomically disable global interrupts and return the previous mstatus value.
[[nodiscard]] inline uint32_t disable_interrupts() noexcept {
    uint32_t prev;
    // MIE (Machine Interrupt Enable) is bit 3 (0x8) in mstatus
    asm volatile("csrrc %0, mstatus, %1" : "=r"(prev) : "r"(0x8) : "memory");
    return prev;
}

/// Restore mstatus to a previously saved state.
inline void restore_interrupts(uint32_t prev_mstatus) noexcept {
    asm volatile("csrw mstatus, %0" : : "r"(prev_mstatus) : "memory");
}

/// Enable machine global interrupts.
inline void enable_interrupts() noexcept {
    asm volatile("csrsi mstatus, 0x8" : : : "memory");
}

/// Check if global interrupts are currently enabled.
[[nodiscard]] inline bool are_interrupts_enabled() noexcept {
    uint32_t mstatus;
    asm volatile("csrr %0, mstatus" : "=r"(mstatus) : : "memory");
    return (mstatus & 0x8) != 0;
}

/// Memory fence.
inline void fence() noexcept {
    asm volatile("fence" ::: "memory");
}

/// Instruction fence (fence.i).
inline void fence_i() noexcept {
    asm volatile("fence.i" ::: "memory");
}

/// No-operation.
inline void nop() noexcept {
    asm volatile("nop");
}

/// Wait for interrupt.
inline void wfi() noexcept {
    asm volatile("wfi");
}

/**
 * @brief RAII guard for critical sections.
 * Automatically disables interrupts on construction and restores previous interrupt state on destruction.
 */
class [[nodiscard]] CriticalSection {
public:
    inline CriticalSection() noexcept : m_prev_mstatus(disable_interrupts()) {}
    inline ~CriticalSection() noexcept { restore_interrupts(m_prev_mstatus); }

    CriticalSection(const CriticalSection&) = delete;
    CriticalSection& operator=(const CriticalSection&) = delete;
    CriticalSection(CriticalSection&&) = delete;
    CriticalSection& operator=(CriticalSection&&) = delete;

private:
    uint32_t m_prev_mstatus;
};

} // namespace hal::core
