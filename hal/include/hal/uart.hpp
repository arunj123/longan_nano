#pragma once

#include <cstdint>
#include <string_view>
#include "hal/register.hpp"
#include "hal/gpio.hpp"

namespace hal::uart {

namespace detail {
    inline constexpr uintptr_t kUsart0Base = 0x40013800;
    inline constexpr uintptr_t kRcuApb2En = 0x40021018;
    inline constexpr uint32_t kUsart0RcuBit = 1U << 14; // USART0EN on APB2
} // namespace detail

/**
 * @brief Zero-overhead compile-time UART driver for GD32VF103.
 * Transmit and receive compile down to single inline machine instructions.
 */
template <uintptr_t BaseAddr>
struct UartDevice {
    static constexpr uintptr_t base = BaseAddr;

    using RegSTAT = Register<base + 0x00, uint32_t>;
    using RegDATA = Register<base + 0x04, uint32_t>;
    using RegBAUD = Register<base + 0x08, uint32_t>;
    using RegCTL0 = Register<base + 0x0C, uint32_t>;

    static constexpr uint32_t STAT_TBE  = 1U << 7;  // Transmit buffer empty
    static constexpr uint32_t STAT_TC   = 1U << 6;  // Transmission complete
    static constexpr uint32_t STAT_RBNE = 1U << 5;  // Read buffer not empty

    static constexpr uint32_t CTL0_UEN = 1U << 13; // USART enable
    static constexpr uint32_t CTL0_TEN = 1U << 3;  // Transmitter enable
    static constexpr uint32_t CTL0_REN = 1U << 2;  // Receiver enable

    /// Initialize USART with specified baud rate assuming APB2 clock (108MHz or 96MHz).
    static inline void init(uint32_t baud, uint32_t apb_clock_hz = 108000000) noexcept {
        // Enable USART0 clock
        Register<detail::kRcuApb2En, uint32_t>::set_bits(detail::kUsart0RcuBit);

        // Configure PA9 (TX) as Alternate Push-Pull, PA10 (RX) as Input Floating
        using TxPin = gpio::GpioPin<gpio::Port::A, 9>;
        using RxPin = gpio::GpioPin<gpio::Port::A, 10>;
        TxPin::init(gpio::Mode::AlternatePushPull, gpio::Speed::Speed50MHz);
        RxPin::init(gpio::Mode::InputFloating);

        // Set baud rate divisor
        const uint32_t udiv = (apb_clock_hz + baud / 2) / baud;
        RegBAUD::write(udiv);

        // Enable transmitter, receiver, and USART peripheral
        RegCTL0::write(CTL0_UEN | CTL0_TEN | CTL0_REN);
    }

    /// Transmit a single byte (blocking until buffer ready).
    static inline void putc(char ch) noexcept {
        while ((RegSTAT::read() & STAT_TBE) == 0) {}
        RegDATA::write(static_cast<uint32_t>(static_cast<uint8_t>(ch)));
    }

    /// Transmit a string_view.
    static inline void print(std::string_view str) noexcept {
        for (char ch : str) {
            putc(ch);
        }
    }

    /// Check if a byte is available to read.
    [[nodiscard]] static inline bool available() noexcept {
        return (RegSTAT::read() & STAT_RBNE) != 0;
    }

    /// Read a byte (blocking).
    [[nodiscard]] static inline char getc() noexcept {
        while (!available()) {}
        return static_cast<char>(RegDATA::read() & 0xFF);
    }
};

using Uart0 = UartDevice<detail::kUsart0Base>;

} // namespace hal::uart
