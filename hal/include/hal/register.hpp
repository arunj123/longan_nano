#pragma once

#include <cstdint>
#include <cstddef>
#include <concepts>

namespace hal {

/**
 * @brief Zero-overhead type-safe MMIO register abstraction.
 * @tparam Address Physical memory-mapped address of the register.
 * @tparam T Register data type (typically uint32_t, uint16_t, or uint8_t).
 */
template <uintptr_t Address, std::unsigned_integral T = uint32_t>
struct Register {
    static_assert(Address % alignof(T) == 0, "MMIO register address must be naturally aligned");

    static constexpr uintptr_t address = Address;
    using value_type = T;

    /// Read the register value directly from hardware.
    [[nodiscard]] static inline T read() noexcept {
        return *reinterpret_cast<volatile T*>(Address);
    }

    /// Write a new value directly to hardware.
    static inline void write(T val) noexcept {
        *reinterpret_cast<volatile T*>(Address) = val;
    }

    /// Atomically set specific bit(s) via read-modify-write.
    static inline void set_bits(T mask) noexcept {
        write(read() | mask);
    }

    /// Atomically clear specific bit(s) via read-modify-write.
    static inline void clear_bits(T mask) noexcept {
        write(read() & ~mask);
    }

    /// Clear mask bits and set new field bits in a single read-modify-write.
    static inline void modify(T clear_mask, T set_mask) noexcept {
        write((read() & ~clear_mask) | set_mask);
    }
};

/**
 * @brief Represents a specific bitfield within a register.
 */
template <typename Reg, uint8_t Offset, uint8_t Width, typename EnumType = typename Reg::value_type>
struct BitField {
    using T = typename Reg::value_type;
    static constexpr T mask = ((static_cast<T>(1) << Width) - 1) << Offset;

    [[nodiscard]] static inline T read_raw() noexcept {
        return (Reg::read() & mask) >> Offset;
    }

    static inline void write_raw(T val) noexcept {
        Reg::modify(mask, (val << Offset) & mask);
    }
};

} // namespace hal
