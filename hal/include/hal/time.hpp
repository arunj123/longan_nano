#pragma once

#include <cstdint>
#include "hal/core.hpp"

// SystemCoreClock global symbol provided by system_gd32vf103.cpp
extern uint32_t SystemCoreClock;

namespace hal::time {

/// Memory addresses of the GD32VF103 ECLIC / Core Timer 64-bit mtime registers.
inline constexpr uintptr_t MTIME_LO_ADDR = 0xD1000000;
inline constexpr uintptr_t MTIME_HI_ADDR = 0xD1000004;

/**
 * @brief Returns the core timer frequency in Hz.
 * On GD32VF103, mtime clock frequency is SYSCLK / 4.
 */
[[nodiscard]] inline uint32_t timer_frequency_hz() noexcept {
    return SystemCoreClock / 4;
}

/**
 * @brief Atomically reads the 64-bit mtime register without roll-over race condition.
 */
[[nodiscard]] inline uint64_t get_raw_ticks() noexcept {
    volatile const uint32_t* const mtime_lo = reinterpret_cast<const volatile uint32_t*>(MTIME_LO_ADDR);
    volatile const uint32_t* const mtime_hi = reinterpret_cast<const volatile uint32_t*>(MTIME_HI_ADDR);
    uint32_t hi, lo;
    do {
        hi = *mtime_hi;
        lo = *mtime_lo;
    } while (hi != *mtime_hi);
    return (static_cast<uint64_t>(hi) << 32) | lo;
}

/**
 * @brief Returns system uptime in milliseconds since boot.
 */
[[nodiscard]] inline uint32_t millis() noexcept {
    const uint64_t freq = SystemCoreClock / 4;
    return static_cast<uint32_t>((get_raw_ticks() * 1000ULL) / freq);
}

[[nodiscard]] inline uint32_t uptime_ms() noexcept {
    return millis();
}

struct Duration;

/**
 * @brief Represents a monotonic point in time based on the 64-bit hardware mtime counter.
 */
struct Instant {
    uint64_t ticks{0};

    [[nodiscard]] static inline Instant now() noexcept {
        return Instant{get_raw_ticks()};
    }

    [[nodiscard]] inline Duration elapsed() const noexcept;

    inline Instant& operator+=(const Duration& rhs) noexcept;
    inline Instant& operator-=(const Duration& rhs) noexcept;

    constexpr bool operator==(const Instant& rhs) const noexcept { return ticks == rhs.ticks; }
    constexpr bool operator!=(const Instant& rhs) const noexcept { return ticks != rhs.ticks; }
    constexpr bool operator<(const Instant& rhs) const noexcept  { return ticks < rhs.ticks; }
    constexpr bool operator<=(const Instant& rhs) const noexcept { return ticks <= rhs.ticks; }
    constexpr bool operator>(const Instant& rhs) const noexcept  { return ticks > rhs.ticks; }
    constexpr bool operator>=(const Instant& rhs) const noexcept { return ticks >= rhs.ticks; }
};

/**
 * @brief Represents an elapsed duration of time.
 */
struct Duration {
    uint64_t ticks{0};

    [[nodiscard]] static constexpr Duration from_ticks(uint64_t t) noexcept {
        return Duration{t};
    }

    [[nodiscard]] static inline Duration from_ms(uint32_t ms) noexcept {
        return Duration{(static_cast<uint64_t>(ms) * (SystemCoreClock / 4)) / 1000ULL};
    }

    [[nodiscard]] static inline Duration from_us(uint32_t us) noexcept {
        return Duration{(static_cast<uint64_t>(us) * (SystemCoreClock / 4)) / 1000000ULL};
    }

    [[nodiscard]] inline uint32_t to_ms() const noexcept {
        const uint64_t freq = SystemCoreClock / 4;
        return static_cast<uint32_t>((ticks * 1000ULL) / freq);
    }

    [[nodiscard]] inline uint32_t to_us() const noexcept {
        const uint64_t freq = SystemCoreClock / 4;
        return static_cast<uint32_t>((ticks * 1000000ULL) / freq);
    }

    constexpr Duration operator+(const Duration& rhs) const noexcept {
        return Duration{ticks + rhs.ticks};
    }

    constexpr Duration operator-(const Duration& rhs) const noexcept {
        return Duration{ticks - rhs.ticks};
    }

    constexpr Duration& operator+=(const Duration& rhs) noexcept {
        ticks += rhs.ticks;
        return *this;
    }

    constexpr Duration& operator-=(const Duration& rhs) noexcept {
        ticks -= rhs.ticks;
        return *this;
    }

    constexpr bool operator==(const Duration& rhs) const noexcept { return ticks == rhs.ticks; }
    constexpr bool operator!=(const Duration& rhs) const noexcept { return ticks != rhs.ticks; }
    constexpr bool operator<(const Duration& rhs) const noexcept  { return ticks < rhs.ticks; }
    constexpr bool operator<=(const Duration& rhs) const noexcept { return ticks <= rhs.ticks; }
    constexpr bool operator>(const Duration& rhs) const noexcept  { return ticks > rhs.ticks; }
    constexpr bool operator>=(const Duration& rhs) const noexcept { return ticks >= rhs.ticks; }
};

inline Duration Instant::elapsed() const noexcept {
    return Duration{get_raw_ticks() - ticks};
}

inline Instant& Instant::operator+=(const Duration& rhs) noexcept {
    ticks += rhs.ticks;
    return *this;
}

inline Instant& Instant::operator-=(const Duration& rhs) noexcept {
    ticks -= rhs.ticks;
    return *this;
}

[[nodiscard]] inline Duration operator-(const Instant& lhs, const Instant& rhs) noexcept {
    return Duration{lhs.ticks - rhs.ticks};
}

[[nodiscard]] inline Instant operator+(const Instant& lhs, const Duration& rhs) noexcept {
    return Instant{lhs.ticks + rhs.ticks};
}

[[nodiscard]] inline Instant operator-(const Instant& lhs, const Duration& rhs) noexcept {
    return Instant{lhs.ticks - rhs.ticks};
}

/**
 * @brief Busy-wait blocking delay for a specified duration.
 */
inline void delay(Duration d) noexcept {
    const Instant deadline = Instant::now() + d;
    while (Instant::now() < deadline) {
        core::nop();
    }
}

/**
 * @brief Busy-wait blocking delay for milliseconds.
 */
inline void delay_ms(uint32_t ms) noexcept {
    delay(Duration::from_ms(ms));
}

/**
 * @brief Busy-wait blocking delay for microseconds.
 */
inline void delay_us(uint32_t us) noexcept {
    delay(Duration::from_us(us));
}

} // namespace hal::time
