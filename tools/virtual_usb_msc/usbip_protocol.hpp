#pragma once

#include <cstdint>
#include <cstddef>
#include <bit>
#include <concepts>
#include <cstring>

namespace usbip {

// USB/IP Protocol Constants
inline constexpr uint16_t USBIP_VERSION      = 0x0111;
inline constexpr uint16_t OP_REQUEST         = 0x8000;
inline constexpr uint16_t OP_REPLY           = 0x0000;

inline constexpr uint16_t OP_REQ_DEVLIST     = 0x8005;
inline constexpr uint16_t OP_REP_DEVLIST     = 0x0005;
inline constexpr uint16_t OP_REQ_IMPORT      = 0x8003;
inline constexpr uint16_t OP_REP_IMPORT      = 0x0003;

inline constexpr uint32_t USBIP_CMD_SUBMIT   = 0x00000001;
inline constexpr uint32_t USBIP_RET_SUBMIT   = 0x00000003;
inline constexpr uint32_t USBIP_CMD_UNLINK   = 0x00000002;
inline constexpr uint32_t USBIP_RET_UNLINK   = 0x00000004;

inline constexpr uint16_t DEFAULT_PORT       = 3240;
inline constexpr size_t   SYSFS_PATH_MAX     = 256;
inline constexpr size_t   SYSFS_BUS_ID_SIZE  = 32;

// USB Transfer Direction
inline constexpr uint32_t DIR_OUT            = 0;
inline constexpr uint32_t DIR_IN             = 1;

// Endianness helpers (Network byte order is big-endian)
template <std::integral T>
[[nodiscard]] constexpr T to_be(T val) noexcept {
    if constexpr (std::endian::native == std::endian::little) {
        return std::byteswap(val);
    }
    return val;
}

template <std::integral T>
[[nodiscard]] constexpr T from_be(T val) noexcept {
    return to_be(val);
}

#pragma pack(push, 1)

/// @brief Header for OP_REQ_* and OP_REP_* discovery packets
struct OpHeader {
    uint16_t version;
    uint16_t code;
    uint32_t status;
};

/// @brief OP_REP_DEVLIST packet header
struct OpDevlistReplyHeader {
    OpHeader base;
    uint32_t ndev;
};

/// @brief USB device metadata exported over USB/IP
struct UsbDeviceDesc {
    char     path[SYSFS_PATH_MAX];
    char     busid[SYSFS_BUS_ID_SIZE];
    uint32_t busnum;
    uint32_t devnum;
    uint32_t speed; // 2 = High Speed, 3 = Full Speed
    uint16_t idVendor;
    uint16_t idProduct;
    uint16_t bcdDevice;
    uint8_t  bDeviceClass;
    uint8_t  bDeviceSubClass;
    uint8_t  bDeviceProtocol;
    uint8_t  bConfigurationValue;
    uint8_t  bNumConfigurations;
    uint8_t  bNumInterfaces;
};

/// @brief USB interface metadata exported over USB/IP
struct UsbInterfaceDesc {
    uint8_t bInterfaceClass;
    uint8_t bInterfaceSubClass;
    uint8_t bInterfaceProtocol;
    uint8_t padding;
};

/// @brief OP_REQ_IMPORT packet payload
struct OpImportRequest {
    OpHeader base;
    char     busid[SYSFS_BUS_ID_SIZE];
};

/// @brief OP_REP_IMPORT packet payload
struct OpImportReply {
    OpHeader      base;
    UsbDeviceDesc udev;
};

/// @brief Common 20-byte header for all USBIP URB commands
struct UsbipHeaderBasic {
    uint32_t command;
    uint32_t seqnum;
    uint32_t devid;
    uint32_t direction; // 0 = OUT, 1 = IN
    uint32_t ep;        // Endpoint number
};

/// @brief 28-byte payload for USBIP_CMD_SUBMIT
struct UsbipCmdSubmit {
    uint32_t transfer_flags;
    int32_t  transfer_buffer_length;
    int32_t  start_frame;
    int32_t  number_of_packets;
    int32_t  interval;
    uint8_t  setup[8];
};

/// @brief 28-byte payload for USBIP_RET_SUBMIT
struct UsbipRetSubmit {
    int32_t  status;           // 0 = Success, -EPIPE = STALL
    int32_t  actual_length;    // Bytes transferred
    int32_t  start_frame;      // 0
    int32_t  number_of_packets;// 0
    int32_t  error_count;      // 0
    uint8_t  padding[8];       // Pads to 28 bytes so total header is 48 bytes
};

/// @brief 28-byte payload for USBIP_CMD_UNLINK
struct UsbipCmdUnlink {
    uint32_t unlink_seqnum;
    uint8_t  padding[24];
};

/// @brief 28-byte payload for USBIP_RET_UNLINK
struct UsbipRetUnlink {
    int32_t  status;
    uint8_t  padding[24];
};

/// @brief Full 48-byte USBIP packet header
struct UsbipPacketHeader {
    UsbipHeaderBasic base;
    union {
        UsbipCmdSubmit cmd_submit;
        UsbipRetSubmit ret_submit;
        UsbipCmdUnlink cmd_unlink;
        UsbipRetUnlink ret_unlink;
        uint8_t        raw[28];
    } u;

    void swap_to_network() noexcept {
        base.command   = to_be(base.command);
        base.seqnum    = to_be(base.seqnum);
        base.devid     = to_be(base.devid);
        base.direction = to_be(base.direction);
        base.ep        = to_be(base.ep);
    }

    void swap_from_network() noexcept {
        base.command   = from_be(base.command);
        base.seqnum    = from_be(base.seqnum);
        base.devid     = from_be(base.devid);
        base.direction = from_be(base.direction);
        base.ep        = from_be(base.ep);
    }
};
static_assert(sizeof(UsbipPacketHeader) == 48, "UsbipPacketHeader must be exactly 48 bytes");

#pragma pack(pop)

} // namespace usbip
