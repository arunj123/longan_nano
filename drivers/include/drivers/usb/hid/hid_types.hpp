#pragma once

#include <cstdint>
#include <span>
#include "drivers/usb/usb_ch9.hpp"

namespace usb::hid {

inline constexpr uint8_t HID_CLASS        = 0x03U;
inline constexpr uint8_t DESC_TYPE_HID    = 0x21U;
inline constexpr uint8_t DESC_TYPE_REPORT = 0x22U;

#pragma pack(push, 1)

/// @brief Standard HID Functional Descriptor (9 bytes)
struct DescHid {
    usb_desc_header header;
    uint16_t bcdHID{0x0111};
    uint8_t  bCountryCode{0};
    uint8_t  bNumDescriptors{1};
    uint8_t  bDescriptorType{DESC_TYPE_REPORT};
    uint16_t wDescriptorLength{0};
};

/// @brief HID-specific control requests (bRequest)
enum class Request : uint8_t {
    GetReport   = 0x01,
    GetIdle     = 0x02,
    GetProtocol = 0x03,
    SetReport   = 0x09,
    SetIdle     = 0x0A,
    SetProtocol = 0x0B
};

/// @brief HID-specific descriptor types
enum class DescriptorType : uint8_t {
    Hid      = 0x21,
    Report   = 0x22,
    Physical = 0x23
};

/// @brief Standard Mouse Report (5 bytes, Report ID 1)
struct MouseReport {
    uint8_t report_id{1};
    uint8_t buttons{0};
    int8_t  x{0};
    int8_t  y{0};
    int8_t  wheel{0};
};
static_assert(sizeof(MouseReport) == 5, "MouseReport must be exactly 5 bytes");

/// @brief Standard 6-Key Rollover Keyboard Report (9 bytes, Report ID 2)
struct KeyboardReport {
    uint8_t report_id{2};
    uint8_t modifier{0};
    uint8_t reserved{0};
    uint8_t keycodes[6]{0};
};
static_assert(sizeof(KeyboardReport) == 9, "KeyboardReport must be exactly 9 bytes");

/// @brief Consumer Control Media Key Report (3 bytes, Report ID 3)
struct ConsumerReport {
    uint8_t  report_id{3};
    uint16_t usage_code{0};
};
static_assert(sizeof(ConsumerReport) == 3, "ConsumerReport must be exactly 3 bytes");

#pragma pack(pop)

/// @brief Consumer media key usage codes (HID Consumer Page 0x0C)
namespace consumer {
    inline constexpr uint16_t None         = 0x0000;
    inline constexpr uint16_t PlayPause    = 0x00CD;
    inline constexpr uint16_t ScanNext     = 0x00B5;
    inline constexpr uint16_t ScanPrevious = 0x00B6;
    inline constexpr uint16_t Stop         = 0x00B7;
    inline constexpr uint16_t VolumeUp     = 0x00E9;
    inline constexpr uint16_t VolumeDown   = 0x00EA;
    inline constexpr uint16_t Mute         = 0x00E2;
}

} // namespace usb::hid
