#pragma once

#include <cstdint>
#include <cstddef>
#include <span>

namespace usb {

#pragma pack(push, 1)

/// @brief USB standard request setup packet (8 bytes)
struct SetupPacket {
    uint8_t  bmRequestType;
    uint8_t  bRequest;
    uint16_t wValue;
    uint16_t wIndex;
    uint16_t wLength;

    [[nodiscard]] constexpr uint8_t direction() const noexcept {
        return (bmRequestType >> 7) & 0x01;
    }
    [[nodiscard]] constexpr bool is_in() const noexcept {
        return direction() == 1;
    }
    [[nodiscard]] constexpr bool is_out() const noexcept {
        return direction() == 0;
    }
    [[nodiscard]] constexpr uint8_t type() const noexcept {
        return (bmRequestType >> 5) & 0x03;
    }
    [[nodiscard]] constexpr uint8_t recipient() const noexcept {
        return bmRequestType & 0x1F;
    }
};

/// @brief Standard USB Descriptor Header (2 bytes)
struct DescriptorHeader {
    uint8_t bLength;
    uint8_t bDescriptorType;
};

/// @brief Standard USB Device Descriptor (18 bytes)
struct DeviceDescriptor {
    DescriptorHeader header;
    uint16_t         bcdUSB;
    uint8_t          bDeviceClass;
    uint8_t          bDeviceSubClass;
    uint8_t          bDeviceProtocol;
    uint8_t          bMaxPacketSize0;
    uint16_t         idVendor;
    uint16_t         idProduct;
    uint16_t         bcdDevice;
    uint8_t          iManufacturer;
    uint8_t          iProduct;
    uint8_t          iSerialNumber;
    uint8_t          bNumberConfigurations;
};

/// @brief Standard USB Configuration Descriptor (9 bytes)
struct ConfigurationDescriptor {
    DescriptorHeader header;
    uint16_t         wTotalLength;
    uint8_t          bNumInterfaces;
    uint8_t          bConfigurationValue;
    uint8_t          iConfiguration;
    uint8_t          bmAttributes;
    uint8_t          bMaxPower;
};

/// @brief Standard USB Interface Descriptor (9 bytes)
struct InterfaceDescriptor {
    DescriptorHeader header;
    uint8_t          bInterfaceNumber;
    uint8_t          bAlternateSetting;
    uint8_t          bNumEndpoints;
    uint8_t          bInterfaceClass;
    uint8_t          bInterfaceSubClass;
    uint8_t          bInterfaceProtocol;
    uint8_t          iInterface;
};

/// @brief Standard USB Endpoint Descriptor (7 bytes)
struct EndpointDescriptor {
    DescriptorHeader header;
    uint8_t          bEndpointAddress;
    uint8_t          bmAttributes;
    uint16_t         wMaxPacketSize;
    uint8_t          bInterval;
};

/// @brief Standard USB HID Descriptor (9 bytes, strictly packed)
struct HidDescriptor {
    DescriptorHeader header;
    uint16_t         bcdHID;
    uint8_t          bCountryCode;
    uint8_t          bNumDescriptors;
    uint8_t          bDescriptorType;
    uint16_t         wDescriptorLength;
};

static_assert(sizeof(SetupPacket) == 8, "SetupPacket must be 8 bytes");
static_assert(sizeof(DeviceDescriptor) == 18, "DeviceDescriptor must be 18 bytes");
static_assert(sizeof(ConfigurationDescriptor) == 9, "ConfigurationDescriptor must be 9 bytes");
static_assert(sizeof(InterfaceDescriptor) == 9, "InterfaceDescriptor must be 9 bytes");
static_assert(sizeof(EndpointDescriptor) == 7, "EndpointDescriptor must be 7 bytes");
static_assert(sizeof(HidDescriptor) == 9, "HidDescriptor must be exactly 9 bytes");

#pragma pack(pop)

/// @brief Standard USB Requests (bRequest)
enum class StandardRequest : uint8_t {
    GetStatus        = 0x00,
    ClearFeature     = 0x01,
    SetFeature       = 0x03,
    SetAddress       = 0x05,
    GetDescriptor    = 0x06,
    SetDescriptor    = 0x07,
    GetConfiguration = 0x08,
    SetConfiguration = 0x09,
    GetInterface     = 0x0A,
    SetInterface     = 0x0B,
    SynchFrame       = 0x0C
};

/// @brief Descriptor Types
enum class DescriptorType : uint8_t {
    Device           = 0x01,
    Configuration    = 0x02,
    String           = 0x03,
    Interface        = 0x04,
    Endpoint         = 0x05,
    DeviceQualifier  = 0x06,
    OtherSpeedConfig = 0x07,
    InterfacePower   = 0x08,
    Hid              = 0x21,
    Report           = 0x22,
    Physical         = 0x23,
    CsInterface      = 0x24,
    CsEndpoint       = 0x25
};

/// @brief Request status returned by class/endpoint handlers
enum class ReqStatus : uint8_t {
    Success = 0,
    Fail    = 1,
    NotSupported = 2
};

/// @brief Endpoint Transfer Type
enum class EndpointType : uint8_t {
    Control     = 0x00,
    Isochronous = 0x01,
    Bulk        = 0x02,
    Interrupt   = 0x03
};

/// @brief Endpoint Direction
enum class Direction : uint8_t {
    Out = 0,
    In  = 1
};

} // namespace usb
