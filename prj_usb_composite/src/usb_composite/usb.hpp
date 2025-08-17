/*!
    \file    usb.hpp
    \brief   Public C++ API for the composite USB device

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef USB_HPP
#define USB_HPP

#include <cstdint>

namespace usb {
    /*!
        \brief      Initializes all USB-related clocks, interrupts, and peripherals.
        \param[in]  enable_msc: Set to true to include the Mass Storage interface, false otherwise.
    */
    void init(bool enable_msc = true); // Default to true for backward compatibility

    /*!
        \brief      Handles periodic USB tasks. Call this in the main loop if you have
                    any non-interrupt-driven USB logic (currently not required for this implementation).
    */
    void poll();

    /*!
        \brief      Checks if the USB device has been fully enumerated and configured by the host.
        \retval     true if configured and ready for data transfer, false otherwise.
    */
    bool is_configured();

    /*!
        \brief      Sends a standard HID mouse report.
        \param[in]  x: Relative movement on the X-axis (-127 to 127).
        \param[in]  y: Relative movement on the Y-axis (-127 to 127).
        \param[in]  wheel: Relative scroll wheel movement (-127 to 127).
        \param[in]  buttons: Bitmask for mouse buttons (Bit 0: Left, Bit 1: Right, Bit 2: Middle).
    */
    void send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons);

    /*!
        \brief      Sends a standard HID keyboard report.
        \param[in]  modifier: Bitmask for modifier keys (e.g., L-CTRL, L-SHIFT).
        \param[in]  key: The keycode of the key to press.
    */
    void send_keyboard_report(uint8_t modifier, uint8_t key);

    /*!
        \brief      Sends a standard HID consumer control report (e.g., for media keys).
        \param[in]  usage_code: The consumer usage code (e.g., 0x00E9 for Volume Up).
    */
    void send_consumer_report(uint16_t usage_code);

    /*!
        \brief      Sends a custom HID report to the host application.
        \param[in]  report_id: The ID of the report, as defined in the report descriptor.
        \param[in]  data: The single byte of data for the report.
    */
    void send_custom_hid_report(uint8_t report_id, uint8_t data);

} // namespace usb

#endif // USB_HPP