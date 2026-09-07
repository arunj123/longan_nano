#include "usb_device.h"
#include <cstring>
#include <cstdio>

UsbDevice& UsbDevice::getInstance() {
    static UsbDevice instance;
    return instance;
}

UsbDevice::UsbDevice() {
    m_descriptors.dev_desc = reinterpret_cast<uint8_t*>(&dev_desc);
    m_descriptors.config_desc = reinterpret_cast<uint8_t*>(&config_desc);
    m_descriptors.strings = usbd_strings;

    auto& custom_hid = m_device.get_driver<0>();
    custom_hid.set_config({
        .ep_in_desc = &config_desc.custom_hid_epin,
        .ep_out_desc = &config_desc.custom_hid_epout,
        .hid_desc = &config_desc.custom_hid_desc,
        .report_desc = {custom_hid_report_descriptor, CUSTOM_HID_REPORT_DESC_LEN},
        .rx_callback = nullptr
    });
}

void UsbDevice::init() {
    m_device.init(&m_descriptors);
}

void UsbDevice::poll() {
    m_device.poll();
}

bool UsbDevice::is_configured() {
    return m_device.is_configured();
}

void UsbDevice::isr() {
    m_device.isr();
}

void UsbDevice::wakeup_isr() {
    m_device.wakeup_isr();
}

void UsbDevice::timer_isr() {
    m_device.timer_isr();
}

bool UsbDevice::is_in_transfer_complete() {
    return m_device.get_driver<0>().is_transfer_complete();
}

bool UsbDevice::send_report(const uint8_t* buffer, size_t length) {
    return m_device.get_driver<0>().send_report(&m_device.core(), buffer, length);
}

namespace usb {
    void init() { UsbDevice::getInstance().init(); }
    void poll() { UsbDevice::getInstance().poll(); }
    bool is_configured() { return UsbDevice::getInstance().is_configured(); }
    bool send_report(const uint8_t* buffer, size_t length) { return UsbDevice::getInstance().send_report(buffer, length); }
    bool is_transfer_complete() { return UsbDevice::getInstance().is_in_transfer_complete(); }
}