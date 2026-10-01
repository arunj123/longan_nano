/*!
    \file    usb_device.h
    \brief   Header for the modern C++ USB device composite HID abstraction layer
*/

#pragma once

#include "drivers/usb/usb_core.hpp"
#include "bsp/usb_hw.hpp"
#include "usb_types.h"
#include "usbd_descriptors.h"

// Forward declaration
class UsbDevice;

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

class UsbDevice {
public:
    static UsbDevice& getInstance();

    // Public methods for C-style ISRs
    void isr();
    void wakeup_isr();
    void timer_isr();

    // Public methods for the application
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
    void operator=(const UsbDevice&) = delete;

    void init();
    void poll();
    bool is_configured();

    // --- Composite Dispatcher Methods ---
    uint8_t _init_composite(uint8_t config_index);
    uint8_t _deinit_composite(uint8_t config_index);
    uint8_t _req_handler(usb::UsbRequest *req);
    uint8_t _data_in(uint8_t ep_num);
    uint8_t _data_out(uint8_t ep_num);

    // --- Standard HID Implementation ---
    void _std_hid_init();
    void _std_hid_deinit();
    uint8_t _std_hid_req_handler(usb::UsbRequest *req);
    void _std_hid_data_in();

    // --- Custom HID Implementation ---
    void _custom_hid_init();
    void _custom_hid_deinit();
    uint8_t _custom_hid_req_handler(usb::UsbRequest *req);
    void _custom_hid_data_in();
    void _custom_hid_data_out();

    // Static C-style callbacks that bridge to the C++ class
    static uint8_t _init_cb(usb_dev *udev, uint8_t config_index);
    static uint8_t _deinit_cb(usb_dev *udev, uint8_t config_index);
    static uint8_t _req_handler_cb(usb_dev *udev, usb_req *req);
    static uint8_t _data_in_cb(usb_dev *udev, uint8_t ep_num);
    static uint8_t _data_out_cb(usb_dev *udev, uint8_t ep_num);

    static UsbDevice* s_instance;

    // A single, global flag to prevent concurrent IN transfers
    volatile bool m_in_transfer_complete;

    usb_core_driver m_core_driver;
    usb_class_core  m_class_core;
    usb_desc        m_descriptors;

    // State handlers for each class
    usb::hid::StandardHidHandler m_std_hid_handler;
    usb::hid::CustomHidHandler   m_custom_hid_handler;

    // --- Friend Declarations ---
    friend void usb::init();
    friend void usb::poll();
    friend bool usb::is_configured();
    friend void usb::send_mouse_report(int8_t, int8_t, int8_t, uint8_t);
    friend void usb::send_keyboard_report(uint8_t, uint8_t);
    friend void usb::send_consumer_report(uint16_t);
    friend bool usb::send_custom_hid_report(const uint8_t*, size_t);
    friend bool usb::is_std_hid_transfer_complete();
};