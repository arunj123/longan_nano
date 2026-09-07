#pragma once

#include <cstdint>
#include "usb_types.hpp"

namespace usb {

/**
 * @brief Type-safe representation of an 8-bit USB Endpoint Address.
 */
class EndpointAddress {
public:
    constexpr EndpointAddress() noexcept : m_raw(0) {}
    constexpr explicit EndpointAddress(uint8_t raw) noexcept : m_raw(raw) {}
    constexpr EndpointAddress(uint8_t ep_num, Direction dir) noexcept
        : m_raw(static_cast<uint8_t>((ep_num & 0x7F) | (dir == Direction::In ? 0x80 : 0x00))) {}

    [[nodiscard]] constexpr uint8_t number() const noexcept {
        return m_raw & 0x7F;
    }

    [[nodiscard]] constexpr Direction direction() const noexcept {
        return (m_raw & 0x80) ? Direction::In : Direction::Out;
    }

    [[nodiscard]] constexpr bool is_in() const noexcept {
        return (m_raw & 0x80) != 0;
    }

    [[nodiscard]] constexpr bool is_out() const noexcept {
        return (m_raw & 0x80) == 0;
    }

    [[nodiscard]] constexpr uint8_t raw() const noexcept {
        return m_raw;
    }

    constexpr bool operator==(const EndpointAddress& rhs) const noexcept {
        return m_raw == rhs.m_raw;
    }

    constexpr bool operator!=(const EndpointAddress& rhs) const noexcept {
        return m_raw != rhs.m_raw;
    }

private:
    uint8_t m_raw{0};
};

} // namespace usb
