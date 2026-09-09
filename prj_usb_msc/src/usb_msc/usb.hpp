#ifndef USB_HPP
#define USB_HPP

#include <cstdint>

namespace usb {
    void init();
    void poll();
    bool is_configured();
}

#endif /* USB_HPP */
