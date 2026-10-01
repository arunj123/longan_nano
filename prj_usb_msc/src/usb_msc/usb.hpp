#pragma once

#include <cstdint>

namespace usb {
    void init();
    void poll();
    bool is_configured();
}
