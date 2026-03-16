#include "usb_device.h"
#include <cstring>
#include "board.h"
#include <cstdio>

extern "C" {
    #include "usbd_transc.h"
    void usbd_isr(usb_core_driver *udev);
    void usb_timer_irq(void);
    void serial_string_get(uint16_t *unicode_str);
}

// Public API
void usb::init() { UsbDevice::getInstance().init(); }
void usb::poll() { UsbDevice::getInstance().poll(); }
bool usb::is_configured() { return UsbDevice::getInstance().is_configured(); }
bool usb::send_report(const uint8_t* buffer, size_t length) { return UsbDevice::getInstance().send_report(buffer, length); }
bool usb::is_transfer_complete() { return UsbDevice::getInstance().is_in_transfer_complete(); }

UsbDevice& UsbDevice::getInstance() {
    static UsbDevice instance;
    return instance;
}

UsbDevice::UsbDevice() : m_in_transfer_complete(true) {
    memset(&m_core_driver, 0, sizeof(usb_core_driver));
    memset(&m_class_core, 0, sizeof(usb_class_core));
    memset(&m_descriptors, 0, sizeof(usb_desc));
    memset(&m_handler, 0, sizeof(usb::hid::CustomHidHandler));

    m_class_core.init = _init_cb;
    m_class_core.deinit = _deinit_cb;
    m_class_core.req_proc = _req_handler_cb;
    m_class_core.data_in = _data_in_cb;
    m_class_core.data_out = _data_out_cb;

    m_descriptors.dev_desc = (uint8_t *)&dev_desc;
    m_descriptors.config_desc = (uint8_t *)&config_desc;
    m_descriptors.strings = usbd_strings;

    serial_string_get((uint16_t*)m_descriptors.strings[usb::STR_IDX_SERIAL]);
}

void UsbDevice::init() {
    eclic_global_interrupt_enable();
    eclic_priority_group_set(ECLIC_PRIGROUP_LEVEL2_PRIO2);
    usb_rcu_config();
    usb_timer_init();
    usb_intr_config();
    usbd_init(&m_core_driver, &m_descriptors, &m_class_core);
}

void UsbDevice::poll() {}
bool UsbDevice::is_configured() { return m_core_driver.dev.cur_status == USBD_CONFIGURED; }

void UsbDevice::isr() { usbd_isr(&m_core_driver); }
void UsbDevice::wakeup_isr() {
    exti_interrupt_flag_clear(EXTI_18);
}
void UsbDevice::timer_isr() { usb_timer_irq(); }

bool UsbDevice::send_report(const uint8_t* buffer, size_t length) {
    if (length > 64) return false;
    if (m_in_transfer_complete) {
        m_in_transfer_complete = false;
        static uint8_t report_buffer[64];
        memset(report_buffer, 0, 64);
        memcpy(report_buffer, buffer, length);
        
        // Re-arm OUT EP
        usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_handler.data, 64U);
        usbd_ep_send(&m_core_driver, CUSTOM_HID_IN_EP, report_buffer, 64);
        return true;
    }
    return false;
}

uint8_t UsbDevice::_init_hid(uint8_t config_index) {
    m_core_driver.dev.class_data[0] = &m_handler;
    usbd_ep_setup(&m_core_driver, &(config_desc.custom_hid_epin));
    usbd_ep_setup(&m_core_driver, &(config_desc.custom_hid_epout));
    usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_handler.data, 64U);
    return USBD_OK;
}

uint8_t UsbDevice::_deinit_hid(uint8_t config_index) {
    usbd_ep_clear(&m_core_driver, CUSTOM_HID_IN_EP);
    usbd_ep_clear(&m_core_driver, CUSTOM_HID_OUT_EP);
    return USBD_OK;
}

uint8_t UsbDevice::_req_handler(usb::UsbRequest *req) {
    usb_transc *transc = &m_core_driver.dev.transc_in[0];
    switch(static_cast<usb::hid::HidReq>(req->bRequest)) {
        case usb::hid::HidReq::GET_IDLE:
            transc->xfer_buf = (uint8_t *)&m_handler.idlestate;
            transc->remain_len = 1U;
            usbd_ctl_send(&m_core_driver);
            break;
        case usb::hid::HidReq::SET_IDLE: 
            m_handler.idlestate = (uint8_t)(req->wValue >> 8); 
            break;
        default:
            if (req->bRequest == static_cast<uint8_t>(usb::StdReq::GET_DESCRIPTOR)) {
                if(usb::hid::DESC_TYPE_REPORT == (req->wValue >> 8)) {
                    transc->remain_len = USB_MIN(CUSTOM_HID_REPORT_DESC_LEN, req->wLength);
                    transc->xfer_buf = (uint8_t *)custom_hid_report_descriptor;
                    usbd_ctl_send(&m_core_driver);
                }
            } else {
                return USBD_FAIL;
            }
            break;
    }
    return USBD_OK;
}

uint8_t UsbDevice::_data_in(uint8_t ep_num) {
    if (ep_num == (CUSTOM_HID_IN_EP & 0x7F)) {
        m_in_transfer_complete = true;
        return USBD_OK;
    }
    return USBD_FAIL;
}

uint8_t UsbDevice::_data_out(uint8_t ep_num) {
    if (ep_num == (CUSTOM_HID_OUT_EP & 0x7F)) {
        // Re-arm OUT EP
        usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_handler.data, 64U);
        return USBD_OK;
    }
    return USBD_FAIL;
}

uint8_t UsbDevice::_init_cb(usb_dev *udev, uint8_t config_index) { return getInstance()._init_hid(config_index); }
uint8_t UsbDevice::_deinit_cb(usb_dev *udev, uint8_t config_index) { return getInstance()._deinit_hid(config_index); }
uint8_t UsbDevice::_req_handler_cb(usb_dev *udev, usb_req *req) { return getInstance()._req_handler(reinterpret_cast<usb::UsbRequest*>(req)); }
uint8_t UsbDevice::_data_in_cb(usb_dev *udev, uint8_t ep_num) { return getInstance()._data_in(ep_num); }
uint8_t UsbDevice::_data_out_cb(usb_dev *udev, uint8_t ep_num) { return getInstance()._data_out(ep_num); }