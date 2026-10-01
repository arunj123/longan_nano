#pragma once

#include "usb_ch9_std.h"
#include <cstddef>
#include <cstdint>
#include <utility>

/**
 * @brief Modern C++23 compile-time USB UTF-16LE String Descriptor.
 * Enforces 4-byte alignment to prevent RV32IMAC bus traps and optimize DWC2 FIFO transfers.
 */
template <size_t N>
struct alignas(4) StringDescriptor {
    usb_desc_header header;
    char16_t        unicode_string[N];

    [[nodiscard]] constexpr const uint8_t* data() const noexcept {
        return reinterpret_cast<const uint8_t*>(this);
    }
    [[nodiscard]] constexpr size_t size() const noexcept {
        return header.bLength;
    }
};

/**
 * @brief Consteval helper to generate UTF-16LE string descriptors directly from ASCII string literals.
 * Eliminates manual character counting and comma-separated unicode character arrays.
 */
template <size_t N>
consteval auto make_string_descriptor(const char (&str)[N]) noexcept {
    StringDescriptor<N - 1> desc{};
    desc.header.bLength = static_cast<uint8_t>(sizeof(usb_desc_header) + (N - 1) * sizeof(char16_t));
    desc.header.bDescriptorType = USB_DESCTYPE_STR;
    for (size_t i = 0; i < N - 1; ++i) {
        desc.unicode_string[i] = static_cast<char16_t>(str[i]);
    }
    return desc;
}

/**
 * @brief Constexpr helper to generate standard Language ID string descriptors.
 */
constexpr usb_desc_LANGID make_language_id_descriptor(uint16_t lang_id = 0x0409U) noexcept {
    return usb_desc_LANGID{
        .header = {
            .bLength = sizeof(usb_desc_LANGID),
            .bDescriptorType = USB_DESCTYPE_STR
        },
        .wLANGID = lang_id
    };
}
