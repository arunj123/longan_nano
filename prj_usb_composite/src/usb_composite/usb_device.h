#pragma once

#include "drivers/usb/device.hpp"
#include "drivers/usb/hid/standard_hid.hpp"
#include "drivers/usb/hid/custom_hid.hpp"
#include "usbd_descriptors.h"

namespace usb {
    void init();
    void poll();
    bool is_configured();

    // Public API for sending reports
    bool is_std_hid_transfer_complete();
    void send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons);
    void send_keyboard_report(uint8_t modifier, uint8_t key);
    void send_consumer_report(uint16_t usage_code);
    bool send_custom_hid_report(const uint8_t* buffer, size_t length);
    bool is_in_transfer_complete(); 
}

using CompositeHidDevice = usb::CompositeDevice<
    usb::hid::StandardHidDriver<0, STD_HID_IN_EP>,
    usb::hid::CustomHidDriver<1, CUSTOM_HID_IN_EP, CUSTOM_HID_OUT_EP, CUSTOM_HID_IN_PACKET>
>;

class UsbDevice {
public:
    static UsbDevice& getInstance();

    void isr();
    void wakeup_isr();
    void timer_isr();

    void init();
    void poll();
    bool is_configured();

    bool is_std_hid_transfer_complete();
    void send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons);
    void send_keyboard_report(uint8_t modifier, uint8_t key);
    void send_consumer_report(uint16_t usage_code);
    bool send_custom_hid_report(const uint8_t* buffer, size_t length);
    bool is_in_transfer_complete();

private:
    UsbDevice();
    ~UsbDevice() = default;
    UsbDevice(const UsbDevice&) = delete;
    UsbDevice& operator=(const UsbDevice&) = delete;

    usb::GenericUsbDevice<CompositeHidDevice> m_device;
    usb_desc m_descriptors{};
};