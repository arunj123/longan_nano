#ifndef USB_HPP
#define USB_HPP

#include <cstdint>
#include <cstddef>

namespace usb {
    void init();
    void poll();
    bool is_configured();
    
    // Sends a high-speed data report (up to 64 bytes)
    bool send_report(const uint8_t* buffer, size_t length);
    
    // Check if the previous transfer is complete
    bool is_transfer_complete();
} // namespace usb

#endif // USB_HPP