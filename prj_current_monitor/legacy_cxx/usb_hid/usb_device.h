#pragma once

#include "drivers/usb/device.hpp"
#include "drivers/usb/hid/custom_hid.hpp"
#include "usbd_descriptors.h"

namespace usb {
    void init();
    void poll();
    bool is_configured();
    bool send_report(const uint8_t* buffer, size_t length);
    bool is_transfer_complete();
}

using CurrentMonitorComposite = usb::CompositeDevice<
    usb::hid::CustomHidDriver<0, CUSTOM_HID_IN_EP, CUSTOM_HID_OUT_EP, CUSTOM_HID_IN_PACKET>
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
    bool send_report(const uint8_t* buffer, size_t length);
    bool is_in_transfer_complete();

private:
    UsbDevice();
    ~UsbDevice() = default;
    UsbDevice(const UsbDevice&) = delete;
    UsbDevice& operator=(const UsbDevice&) = delete;

    usb::GenericUsbDevice<CurrentMonitorComposite> m_device;
    usb_desc m_descriptors{};
};