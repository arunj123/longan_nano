/*!
    \file    usb_device.h
    \brief   Header for the C++ USB device abstraction layer

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef USB_DEVICE_H
#define USB_DEVICE_H

// C-style headers for the core driver are still needed
extern "C" {
    #include "drv_usb_hw.h"
    #include "usbd_core.h"
}

// Include our self-contained type definitions
#include "usb_types.h"
#include "usbd_descriptors.h"

// Forward declaration
class UsbDevice;

namespace usb {
    void init(bool);
    void poll();
    bool is_configured();

    // Public API for sending reports
    bool is_std_hid_transfer_complete();
    void send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons);
    void send_keyboard_report(uint8_t modifier, uint8_t key);
    void send_consumer_report(uint16_t usage_code);
    void send_custom_hid_report(uint8_t report_id, uint8_t data);
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
    void send_custom_hid_report(uint8_t report_id, uint8_t data);

private:
    UsbDevice();
    ~UsbDevice() = default;
    UsbDevice(const UsbDevice&) = delete;
    void operator=(const UsbDevice&) = delete;

    void init(bool enable_msc);
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

    // --- MSC & BBB & SCSI Implementation ---
    void _msc_init();
    void _msc_deinit();
    uint8_t _msc_req_handler(usb::UsbRequest *req);
    void _msc_data_in();
    void _msc_data_out();
    void _msc_bbb_init();
    void _msc_bbb_reset();
    void _msc_bbb_deinit();
    void _msc_bbb_data_in();
    void _msc_bbb_data_out();
    void _msc_bbb_cbw_decode();
    void _msc_bbb_csw_send(usb::msc::CswStatus csw_status);
    void _msc_bbb_clrfeature(uint8_t ep_num);
    void _msc_bbb_data_send(uint8_t *pbuf, uint32_t len);
    void _msc_bbb_abort();
    int8_t _scsi_process_cmd(uint8_t lun, uint8_t *params);
    void _scsi_sense_code(uint8_t lun, usb::msc::scsi::SenseKey skey, usb::msc::scsi::Asc asc);
    int8_t _scsi_test_unit_ready(uint8_t lun);
    int8_t _scsi_inquiry(uint8_t lun, uint8_t *params);
    int8_t _scsi_read_capacity10(uint8_t lun);
    int8_t _scsi_read_format_capacity(uint8_t lun);
    int8_t _scsi_mode_sense6(uint8_t lun);
    int8_t _scsi_mode_sense10(uint8_t lun);
    int8_t _scsi_request_sense(uint8_t lun, uint8_t *params);
    int8_t _scsi_read10(uint8_t lun, uint8_t *params);
    int8_t _scsi_write10(uint8_t lun, uint8_t *params);
    int8_t _scsi_verify10(uint8_t lun);
    int8_t _scsi_process_read(uint8_t lun);
    int8_t _scsi_process_write(uint8_t lun);
    int8_t _scsi_check_address_range(uint8_t lun, uint32_t blk_offset, uint16_t blk_nbr);

    // Static C-style callbacks that bridge to the C++ class
    static uint8_t _init_cb(usb_dev *udev, uint8_t config_index);
    static uint8_t _deinit_cb(usb_dev *udev, uint8_t config_index);
    static uint8_t _req_handler_cb(usb_dev *udev, usb_req *req);
    static uint8_t _data_in_cb(usb_dev *udev, uint8_t ep_num);
    static uint8_t _data_out_cb(usb_dev *udev, uint8_t ep_num);

    static UsbDevice* s_instance;

    usb_core_driver m_core_driver;
    usb_class_core  m_class_core;
    usb_desc        m_descriptors;
    bool            m_msc_enabled;

    // State handlers for each class using correct C++ types
    usb::hid::StandardHidHandler m_std_hid_handler;
    usb::hid::CustomHidHandler   m_custom_hid_handler;
    usb::msc::MscHandler         m_msc_handler;

    // --- Friend Declarations ---
    // Befriend the public namespace functions so they can call the private methods.
    friend void usb::init(bool enable_msc);
    friend void usb::poll();
    friend bool usb::is_configured();
    friend void usb::send_mouse_report(int8_t, int8_t, int8_t, uint8_t);
    friend void usb::send_keyboard_report(uint8_t, uint8_t);
    friend void usb::send_consumer_report(uint16_t);
    friend void usb::send_custom_hid_report(uint8_t, uint8_t);
};

#endif // USB_DEVICE_H