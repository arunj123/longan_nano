#include "usb_device.h"
#include "bsp/board.hpp"
#include <cstring>

static void on_custom_hid_rx(const uint8_t* data, size_t length) {
    if (length < 2) return;
    uint8_t cmd = data[0];
    uint8_t value = data[1];
    switch (cmd) {
        case 0x11:
            bsp::board::LedRed::set(value != 0);
            break;
        case 0x12:
            bsp::board::LedGreen::set(value != 0);
            break;
        case 0x13:
            bsp::board::LedBlue::set(value != 0);
            break;
    }
}

UsbDevice& UsbDevice::getInstance() {
    static UsbDevice instance;
    return instance;
}

UsbDevice::UsbDevice() {
    m_descriptors.dev_desc = reinterpret_cast<uint8_t*>(&composite_dev_desc);
    m_descriptors.config_desc = reinterpret_cast<uint8_t*>(&composite_config_desc);
    m_descriptors.strings = usbd_composite_strings;

    auto& std_hid = m_device.get_driver<0>();
    std_hid.set_config({
        .ep_desc = &composite_config_desc.std_hid_epin,
        .hid_desc = &composite_config_desc.std_hid_desc,
        .report_desc = {std_hid_report_descriptor, STD_HID_REPORT_DESC_LEN}
    });

    auto& custom_hid = m_device.get_driver<1>();
    custom_hid.set_config({
        .ep_in_desc = &composite_config_desc.custom_hid_epin,
        .ep_out_desc = &composite_config_desc.custom_hid_epout,
        .hid_desc = &composite_config_desc.custom_hid_desc,
        .report_desc = {custom_hid_report_descriptor, CUSTOM_HID_REPORT_DESC_LEN},
        .rx_callback = on_custom_hid_rx
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

bool UsbDevice::is_std_hid_transfer_complete() {
    return m_device.get_driver<0>().is_transfer_complete();
}

void UsbDevice::send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons) {
    m_device.get_driver<0>().send_mouse(&m_device.core(), x, y, wheel, buttons);
}

void UsbDevice::send_keyboard_report(uint8_t modifier, uint8_t key) {
    m_device.get_driver<0>().send_keyboard(&m_device.core(), modifier, key);
}

void UsbDevice::send_consumer_report(uint16_t usage_code) {
    m_device.get_driver<0>().send_consumer(&m_device.core(), usage_code);
}

bool UsbDevice::send_custom_hid_report(const uint8_t* buffer, size_t length) {
    return m_device.get_driver<1>().send_report(&m_device.core(), buffer, length);
}

bool UsbDevice::is_in_transfer_complete() {
    return m_device.get_driver<1>().is_transfer_complete();
}

namespace usb {
    void init() { UsbDevice::getInstance().init(); }
    void poll() { UsbDevice::getInstance().poll(); }
    bool is_configured() { return UsbDevice::getInstance().is_configured(); }
    bool is_std_hid_transfer_complete() { return UsbDevice::getInstance().is_std_hid_transfer_complete(); }
    void send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons) {
        UsbDevice::getInstance().send_mouse_report(x, y, wheel, buttons);
    }
    void send_keyboard_report(uint8_t modifier, uint8_t key) {
        UsbDevice::getInstance().send_keyboard_report(modifier, key);
    }
    void send_consumer_report(uint16_t usage_code) {
        UsbDevice::getInstance().send_consumer_report(usage_code);
    }
    bool send_custom_hid_report(const uint8_t* buffer, size_t length) {
        return UsbDevice::getInstance().send_custom_hid_report(buffer, length);
    }
    bool is_in_transfer_complete() {
        return UsbDevice::getInstance().is_in_transfer_complete();
    }
}
