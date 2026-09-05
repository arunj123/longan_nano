#ifndef USB_DEVICE_H
#define USB_DEVICE_H

#include "drivers/usb/usb_core.hpp"

#include "bsp/usb_hw.hpp"

#include "usb_types.h"
#include "usbd_descriptors.h"

class UsbDevice;

namespace usb {
    void init();
    void poll();
    bool is_configured();
    bool send_report(const uint8_t* buffer, size_t length);
    bool is_transfer_complete();
}

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

    uint8_t _init_hid(uint8_t config_index);
    uint8_t _deinit_hid(uint8_t config_index);
    uint8_t _req_handler(usb::UsbRequest *req);
    uint8_t _data_in(uint8_t ep_num);
    uint8_t _data_out(uint8_t ep_num);

    static uint8_t _init_cb(usb_dev *udev, uint8_t config_index);
    static uint8_t _deinit_cb(usb_dev *udev, uint8_t config_index);
    static uint8_t _req_handler_cb(usb_dev *udev, usb_req *req);
    static uint8_t _data_in_cb(usb_dev *udev, uint8_t ep_num);
    static uint8_t _data_out_cb(usb_dev *udev, uint8_t ep_num);

    usb_core_driver m_core_driver;
    usb_class_core  m_class_core;
    usb_desc        m_descriptors;
    usb::hid::CustomHidHandler m_handler;
    volatile bool m_in_transfer_complete;
};

#endif // USB_DEVICE_H