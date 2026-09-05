#pragma once

#include <cstdint>

namespace hal::csr {

/**
 * @brief Standard and Nuclei Bumblebee N200 specific Control and Status Registers (CSRs).
 * Privilege Architecture: RISC-V Privileged Spec v1.9.1 + Nuclei N200 Extensions.
 */
enum class Csr : uint16_t {
    // Machine Information Registers
    Mvendorid       = 0xF11,
    Marchid         = 0xF12,
    Mimpid          = 0xF13,
    Mhartid         = 0xF14,

    // Machine Trap Setup
    Mstatus         = 0x300,
    Misa            = 0x301,
    Medeleg         = 0x302,
    Mideleg         = 0x303,
    Mie             = 0x304,
    Mtvec           = 0x305,
    Mcounteren      = 0x306,

    // Nuclei N200 Trap Vector Extensions
    Mtvt            = 0x307, // Machine Trap Vector Table base address

    // Machine Trap Handling
    Mscratch        = 0x340,
    Mepc            = 0x341,
    Mcause          = 0x342,
    Mtval           = 0x343,
    Mip             = 0x344,

    // Machine Counter / Timer Setup
    Mcycle          = 0xB00,
    Minstret        = 0xB02,
    Mcountinhibit   = 0x320,

    // Nuclei N200 Custom Machine CSRs
    Msubm           = 0x7C4, // Machine Sub-Mode register
    MmiscCtl        = 0x7D0, // Machine Miscellaneous Control
    Mtvt2           = 0x7EC, // Machine Trap Vector Table 2 (non-vectored IRQ entry)
    Wfe             = 0x810, // Wait-For-Event control
};

/**
 * @brief Bitmask definitions for Machine Status Register (mstatus).
 */
namespace mstatus {
    inline constexpr uint32_t Uie  = 1U << 0;
    inline constexpr uint32_t Sie  = 1U << 1;
    inline constexpr uint32_t Mie  = 1U << 3;  // Global Machine Interrupt Enable
    inline constexpr uint32_t Upie = 1U << 4;
    inline constexpr uint32_t Spie = 1U << 5;
    inline constexpr uint32_t Mpie = 1U << 7;  // Previous Machine Interrupt Enable
    inline constexpr uint32_t Spp  = 1U << 8;
    inline constexpr uint32_t Mpp  = 3U << 11; // Previous Privilege Mode mask
    inline constexpr uint32_t Fs   = 3U << 13; // Floating-point status
}

/**
 * @brief Read a CSR by enum constant.
 */
template <Csr Reg>
[[nodiscard]] inline uint32_t read() noexcept {
    uint32_t val;
    asm volatile("csrr %0, %1" : "=r"(val) : "i"(static_cast<uint16_t>(Reg)) : "memory");
    return val;
}

/**
 * @brief Write a CSR by enum constant.
 */
template <Csr Reg>
inline void write(uint32_t val) noexcept {
    asm volatile("csrw %1, %0" : : "r"(val), "i"(static_cast<uint16_t>(Reg)) : "memory");
}

/**
 * @brief Set bit(s) in a CSR by enum constant.
 */
template <Csr Reg>
inline void set_bits(uint32_t mask) noexcept {
    asm volatile("csrs %1, %0" : : "r"(mask), "i"(static_cast<uint16_t>(Reg)) : "memory");
}

/**
 * @brief Clear bit(s) in a CSR by enum constant.
 */
template <Csr Reg>
inline void clear_bits(uint32_t mask) noexcept {
    asm volatile("csrc %1, %0" : : "r"(mask), "i"(static_cast<uint16_t>(Reg)) : "memory");
}

/**
 * @brief Atomically read and clear bit(s) in a CSR.
 */
template <Csr Reg>
[[nodiscard]] inline uint32_t read_and_clear_bits(uint32_t mask) noexcept {
    uint32_t prev;
    asm volatile("csrrc %0, %1, %2" : "=r"(prev) : "i"(static_cast<uint16_t>(Reg)), "r"(mask) : "memory");
    return prev;
}

/**
 * @brief Atomically read and set bit(s) in a CSR.
 */
template <Csr Reg>
[[nodiscard]] inline uint32_t read_and_set_bits(uint32_t mask) noexcept {
    uint32_t prev;
    asm volatile("csrrs %0, %1, %2" : "=r"(prev) : "i"(static_cast<uint16_t>(Reg)), "r"(mask) : "memory");
    return prev;
}

} // namespace hal::csr
