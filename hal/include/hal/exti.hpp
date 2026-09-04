#pragma once

#include <cstdint>
#include "hal/register.hpp"
#include "hal/gpio.hpp"
#include "hal/rcu.hpp"

namespace hal::exti {

namespace detail {
    inline constexpr uintptr_t kExtiBase = 0x40010400;
    inline constexpr uintptr_t kAfioBase = 0x40010000;

    using RegINTEN = Register<kExtiBase + 0x00, uint32_t>;
    using RegEVEN  = Register<kExtiBase + 0x04, uint32_t>;
    using RegRTEN  = Register<kExtiBase + 0x08, uint32_t>;
    using RegFTEN  = Register<kExtiBase + 0x0C, uint32_t>;
    using RegSWIEV = Register<kExtiBase + 0x10, uint32_t>;
    using RegPD    = Register<kExtiBase + 0x14, uint32_t>;

    inline constexpr uintptr_t afio_extiss_addr(uint8_t line) noexcept {
        return kAfioBase + 0x08U + static_cast<uintptr_t>(line / 4U) * 4U;
    }
} // namespace detail

enum class Trigger : uint8_t {
    Rising  = 0x1,
    Falling = 0x2,
    Both    = 0x3
};

/**
 * @brief Zero-overhead compile-time EXTI (External Interrupt/Event) controller driver.
 */
struct Exti {
    static constexpr uint8_t MaxLines = 19;

    /**
     * @brief Map a GPIO port and pin (0-15) to its corresponding EXTI line using AFIO.
     * Automatically enables the AFIO peripheral clock.
     */
    static inline void map_pin(gpio::Port port, uint8_t pin) noexcept {
        if (pin > 15) return;

        // Ensure AFIO clock is enabled
        rcu::enable(rcu::Peripheral::Afio);

        const uintptr_t reg_addr = detail::afio_extiss_addr(pin);
        const uint8_t shift = (pin % 4U) * 4U;
        const uint32_t mask = 0xFU << shift;
        const uint32_t val = static_cast<uint32_t>(port) << shift;

        auto* reg = reinterpret_cast<volatile uint32_t*>(reg_addr);
        *reg = (*reg & ~mask) | val;
    }

    /**
     * @brief Configure and enable an EXTI line with the given trigger condition.
     */
    static inline void enable_line(uint8_t line, Trigger trigger = Trigger::Falling) noexcept {
        if (line >= MaxLines) return;

        const uint32_t bit = 1U << line;

        // Configure edge triggers
        if (static_cast<uint8_t>(trigger) & static_cast<uint8_t>(Trigger::Rising)) {
            detail::RegRTEN::set_bits(bit);
        } else {
            detail::RegRTEN::clear_bits(bit);
        }

        if (static_cast<uint8_t>(trigger) & static_cast<uint8_t>(Trigger::Falling)) {
            detail::RegFTEN::set_bits(bit);
        } else {
            detail::RegFTEN::clear_bits(bit);
        }

        // Clear any pending flag and unmask interrupt
        clear_pending(line);
        detail::RegINTEN::set_bits(bit);
    }

    /**
     * @brief Disable an EXTI interrupt line.
     */
    static inline void disable_line(uint8_t line) noexcept {
        if (line < MaxLines) {
            const uint32_t bit = 1U << line;
            detail::RegINTEN::clear_bits(bit);
            detail::RegRTEN::clear_bits(bit);
            detail::RegFTEN::clear_bits(bit);
        }
    }

    /**
     * @brief Check if an interrupt pending flag is set for this line.
     */
    [[nodiscard]] static inline bool is_pending(uint8_t line) noexcept {
        if (line >= MaxLines) return false;
        return (detail::RegPD::read() & (1U << line)) != 0;
    }

    /**
     * @brief Clear the pending interrupt flag for this line by writing a 1.
     */
    static inline void clear_pending(uint8_t line) noexcept {
        if (line < MaxLines) {
            detail::RegPD::write(1U << line);
        }
    }

    /**
     * @brief Generate a software interrupt on this line.
     */
    static inline void trigger_software_interrupt(uint8_t line) noexcept {
        if (line < MaxLines) {
            detail::RegSWIEV::set_bits(1U << line);
        }
    }
};

} // namespace hal::exti
