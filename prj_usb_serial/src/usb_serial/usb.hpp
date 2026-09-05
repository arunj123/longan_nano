#ifndef USB_HPP
#define USB_HPP

#include "drivers/usb/cdc_acm.hpp"

// The USB driver instance needs to be globally accessible for the C-based ISRs.
// We declare it here as 'extern' to make it visible across the project.
extern usb_core_driver cdc_acm;

namespace usb {

/*!
    \brief      Initializes all USB-related clocks, interrupts, and peripherals.
*/
void init();

/*!
    \brief      Handles periodic USB tasks like sending and receiving data.
                This should be called repeatedly in the main loop.
*/
void poll();

/*!
    \brief      Checks if the USB device has been configured by the host.
    \retval     true if configured, false otherwise.
*/
bool is_configured();

} // namespace usb

#endif // USB_HPP
