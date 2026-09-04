#pragma once

#include <cstdint>
#include "hal/register.hpp"

namespace hal::gpio {

enum class Port : uint8_t {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4
};

enum class Mode : uint8_t {
    InputAnalog,
    InputFloating,
    InputPullUp,
    InputPullDown,
    OutputPushPull,
    OutputOpenDrain,
    AlternatePushPull,
    AlternateOpenDrain
};

enum class Speed : uint8_t {
    Speed10MHz = 1,
    Speed2MHz  = 2,
    Speed50MHz = 3
};

enum class Level : bool {
    Low  = false,
    High = true
};

enum class ActiveLevel : uint8_t {
    High = 0,
    Low  = 1
};

// Internal address mapping constants
namespace detail {
    inline constexpr uintptr_t kGpioBase = 0x40010800;
    inline constexpr uintptr_t kPortStride = 0x400;

    inline constexpr uintptr_t port_base(Port p) noexcept {
        return kGpioBase + static_cast<uintptr_t>(p) * kPortStride;
    }

    // RCU APB2EN register address
    inline constexpr uintptr_t kRcuApb2En = 0x40021018;

    inline constexpr uint32_t port_rcu_bit(Port p) noexcept {
        // Bit 2 is PAEN, bit 3 is PBEN, etc.
        return 1U << (2 + static_cast<uint32_t>(p));
    }
} // namespace detail

/**
 * @brief Zero-overhead compile-time GPIO pin abstraction for GD32VF103.
 * All operations compile directly to single LUI + SW/LW instructions.
 */
template <Port P, uint8_t PinNum>
struct GpioPin {
    static_assert(PinNum < 16, "GD32 GPIO pin index must be 0-15");

    static constexpr Port port = P;
    static constexpr uint8_t pin = PinNum;
    static constexpr uintptr_t base = detail::port_base(P);

    // Hardware register definitions for this port
    using RegCTL0 = Register<base + 0x00, uint32_t>;
    using RegCTL1 = Register<base + 0x04, uint32_t>;
    using RegISTAT = Register<base + 0x08, uint32_t>;
    using RegOCTL = Register<base + 0x0C, uint32_t>;
    using RegBOP  = Register<base + 0x10, uint32_t>;
    using RegBC   = Register<base + 0x14, uint32_t>;

    /// Enable peripheral clock for this pin's GPIO port.
    static inline void enable_clock() noexcept {
        Register<detail::kRcuApb2En, uint32_t>::set_bits(detail::port_rcu_bit(P));
    }

    /// Configure pin mode and output drive speed.
    static inline void init(Mode mode, Speed speed = Speed::Speed50MHz) noexcept {
        enable_clock();

        uint32_t config_bits = 0;
        switch (mode) {
            case Mode::InputAnalog:
                config_bits = 0x0; // CTL=00, MD=00
                break;
            case Mode::InputFloating:
                config_bits = 0x4; // CTL=01, MD=00
                break;
            case Mode::InputPullUp:
            case Mode::InputPullDown:
                config_bits = 0x8; // CTL=10, MD=00
                break;
            case Mode::OutputPushPull:
                config_bits = (0x0 << 2) | static_cast<uint32_t>(speed); // CTL=00
                break;
            case Mode::OutputOpenDrain:
                config_bits = (0x1 << 2) | static_cast<uint32_t>(speed); // CTL=01
                break;
            case Mode::AlternatePushPull:
                config_bits = (0x2 << 2) | static_cast<uint32_t>(speed); // CTL=10
                break;
            case Mode::AlternateOpenDrain:
                config_bits = (0x3 << 2) | static_cast<uint32_t>(speed); // CTL=11
                break;
        }

        const uint8_t bit_offset = (PinNum % 8) * 4;
        const uint32_t mask = 0xFU << bit_offset;
        const uint32_t val = config_bits << bit_offset;

        if constexpr (PinNum < 8) {
            RegCTL0::modify(mask, val);
        } else {
            RegCTL1::modify(mask, val);
        }

        // Set or clear OCTL bit for Pull-Up / Pull-Down configuration
        if (mode == Mode::InputPullUp) {
            RegBOP::write(1U << PinNum);
        } else if (mode == Mode::InputPullDown) {
            RegBC::write(1U << PinNum);
        }
    }

    /// Set pin output high (logical 1).
    static inline void set() noexcept {
        RegBOP::write(1U << PinNum);
    }

    /// Reset pin output low (logical 0).
    static inline void reset() noexcept {
        RegBC::write(1U << PinNum);
    }

    /// Write logical level directly.
    static inline void write(Level lvl) noexcept {
        if (lvl == Level::High) {
            set();
        } else {
            reset();
        }
    }

    static inline void write(bool high) noexcept {
        write(high ? Level::High : Level::Low);
    }

    /// Toggle pin output value.
    static inline void toggle() noexcept {
        if ((RegOCTL::read() & (1U << PinNum)) != 0) {
            reset();
        } else {
            set();
        }
    }

    /// Read current logic level on the pin.
    [[nodiscard]] static inline Level read() noexcept {
        return (RegISTAT::read() & (1U << PinNum)) ? Level::High : Level::Low;
    }

    [[nodiscard]] static inline bool read_bool() noexcept {
        return read() == Level::High;
    }
};

/**
 * @brief Output device wrapper (e.g. LED, Relay, Buzzer) with active level handling.
 */
template <typename Pin, ActiveLevel Active = ActiveLevel::Low>
struct OutputDevice {
    using Gpio = Pin;
    static constexpr ActiveLevel active_level = Active;

    static inline void init() noexcept {
        Pin::init(Mode::OutputPushPull, Speed::Speed50MHz);
        off();
    }

    static inline void on() noexcept {
        if constexpr (Active == ActiveLevel::High) {
            Pin::set();
        } else {
            Pin::reset();
        }
    }

    static inline void off() noexcept {
        if constexpr (Active == ActiveLevel::High) {
            Pin::reset();
        } else {
            Pin::set();
        }
    }

    static inline void set(bool state) noexcept {
        if (state) on(); else off();
    }

    static inline void toggle() noexcept {
        Pin::toggle();
    }

    [[nodiscard]] static inline bool is_on() noexcept {
        const bool pin_high = Pin::read_bool();
        return (Active == ActiveLevel::High) ? pin_high : !pin_high;
    }
};

/**
 * @brief Input device wrapper (e.g. Pushbutton, Switch) with active level handling.
 */
template <typename Pin, ActiveLevel Active = ActiveLevel::Low>
struct InputDevice {
    using Gpio = Pin;
    static constexpr ActiveLevel active_level = Active;

    static inline void init() noexcept {
        constexpr Mode pull_mode = (Active == ActiveLevel::Low) ? Mode::InputPullUp : Mode::InputPullDown;
        Pin::init(pull_mode);
    }

    [[nodiscard]] static inline bool is_active() noexcept {
        const bool pin_high = Pin::read_bool();
        return (Active == ActiveLevel::High) ? pin_high : !pin_high;
    }
};

} // namespace hal::gpio
