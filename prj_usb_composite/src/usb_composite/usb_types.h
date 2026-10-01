#pragma once

#include <cstdint>
#include "usbd_conf.h"
#include "drivers/usb/usb_ch9.hpp"
#include "drivers/usb/hid/hid_types.hpp"

namespace usb {

using UsbRequest = usb_req;
using DescHeader = usb_desc_header;

namespace hid {

// HID Class-Specific Request Codes (bRequest)
enum class HidReq : uint8_t {
    GET_REPORT   = 0x01,
    GET_IDLE     = 0x02,
    GET_PROTOCOL = 0x03,
    SET_REPORT   = 0x09,
    SET_IDLE     = 0x0A,
    SET_PROTOCOL = 0x0B
};

struct StandardHidHandler {
    uint32_t protocol;
    uint32_t idle_state;
};

struct CustomHidHandler {
    uint8_t data[64];
    uint8_t reportID;
    uint8_t idlestate;
    uint8_t protocol;
};

} // namespace hid
} // namespace usb