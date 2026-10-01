/*!
    \file    usb_device.cpp
    \brief   Implementation of the C++ USB device abstraction layer

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

#include "usb_device.h"
#include <cstring>
#include "board.h"
#include <cstdio>
#include "lcd.h"
#include "shared_defs.h"
#include "display_manager.h"
#include "hal/eclic.hpp"
#include "hal/exti.hpp"
#include "bsp/board.hpp"

// ===================================================================
// Public Namespace Functions (The User's API)
// ===================================================================

void usb::init() { UsbDevice::getInstance().init(); }
void usb::poll() { UsbDevice::getInstance().poll(); }
bool usb::is_configured() { return UsbDevice::getInstance().is_configured(); }
void usb::send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons) { UsbDevice::getInstance().send_mouse_report(x, y, wheel, buttons); }
void usb::send_keyboard_report(uint8_t modifier, uint8_t key) { UsbDevice::getInstance().send_keyboard_report(modifier, key); }
void usb::send_consumer_report(uint16_t usage_code) { UsbDevice::getInstance().send_consumer_report(usage_code); }
bool usb::send_custom_hid_report(const uint8_t* buffer, size_t length) { return UsbDevice::getInstance().send_custom_hid_report(buffer, length); }
bool usb::is_std_hid_transfer_complete() { return UsbDevice::getInstance().is_in_transfer_complete(); }
// ===================================================================
// UsbDevice Class Implementation
// ===================================================================

UsbDevice& UsbDevice::getInstance() {
    static UsbDevice instance; // Guaranteed to be destroyed, instantiated on first use
    return instance;
}

UsbDevice::UsbDevice() : m_in_transfer_complete(true) {
    m_core_driver = usb_core_driver{};
    m_class_core = usb_class_core{};
    m_descriptors = usb_desc{};
    memset(&m_std_hid_handler, 0, sizeof(usb::hid::StandardHidHandler));
    memset(&m_custom_hid_handler, 0, sizeof(usb::hid::CustomHidHandler));

    m_class_core.init = _init_cb;
    m_class_core.deinit = _deinit_cb;
    m_class_core.req_proc = _req_handler_cb;
    m_class_core.data_in = _data_in_cb;
    m_class_core.data_out = _data_out_cb;

    m_descriptors.dev_desc = (uint8_t *)&composite_dev_desc;
    m_descriptors.config_desc = (uint8_t *)&composite_config_desc;
    m_descriptors.strings = usbd_composite_strings;

    serial_string_get((uint16_t*)m_descriptors.strings[usb::STR_IDX_SERIAL]);
}

bool UsbDevice::is_in_transfer_complete() {
    return m_in_transfer_complete;
}

void UsbDevice::init() {
    hal::eclic::Eclic::set_priority_group(hal::eclic::PriorityGroup::Level2Prio2);
    hal::eclic::Eclic::enable_global_interrupts();
    bsp::usb::rcu_config();
    bsp::usb::timer_init();
    bsp::usb::intr_config();
    usbd_init(&m_core_driver, &m_descriptors, &m_class_core);
}

void UsbDevice::poll() { /* For non-interrupt driven logic */ }
bool UsbDevice::is_configured() { return m_core_driver.dev.cur_status == USBD_CONFIGURED; }

// --- ISR Handlers ---
void UsbDevice::isr() { usbd_isr(&m_core_driver); }
void UsbDevice::wakeup_isr() {
    if (m_core_driver.bp.low_power) { /* Resume MCU clock logic here if needed */ }
    hal::exti::Exti::clear_pending(18);
}
void UsbDevice::timer_isr() { bsp::usb::timer_irq(); }

// --- Report Sending Methods ---
void UsbDevice::send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons) {
    uint8_t report[5] = {REPORT_ID_MOUSE, buttons, (uint8_t)x, (uint8_t)y, (uint8_t)wheel};
    if (m_in_transfer_complete) {
        m_in_transfer_complete = false;
        
        // Flush and re-arm the Custom HID OUT endpoint to kick the driver
        // out of its unstable idle state before starting a new IN transfer.
        usbd_fifo_flush(&m_core_driver, CUSTOM_HID_OUT_EP);
        usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 64U);

        usbd_ep_send(&m_core_driver, STD_HID_IN_EP, report, 5);
    }
}

void UsbDevice::send_keyboard_report(uint8_t modifier, uint8_t key) {
    uint8_t report[9] = {REPORT_ID_KEYBOARD, modifier, 0, key, 0, 0, 0, 0, 0};
    if (m_in_transfer_complete) {
        m_in_transfer_complete = false;

        // Flush and re-arm the Custom HID OUT endpoint to kick the driver
        // out of its unstable idle state before starting a new IN transfer.
        usbd_fifo_flush(&m_core_driver, CUSTOM_HID_OUT_EP);
        usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 64U);
        
        usbd_ep_send(&m_core_driver, STD_HID_IN_EP, report, 9);
    }
}

void UsbDevice::send_consumer_report(uint16_t usage_code) {
    uint8_t report[3] = {REPORT_ID_CONSUMER, (uint8_t)(usage_code & 0xFF), (uint8_t)(usage_code >> 8)};
    if (m_in_transfer_complete) {
        m_in_transfer_complete = false;

        // Flush and re-arm the Custom HID OUT endpoint to kick the driver
        // out of its unstable idle state before starting a new IN transfer.
        usbd_fifo_flush(&m_core_driver, CUSTOM_HID_OUT_EP);
        usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 64U);
        
        usbd_ep_send(&m_core_driver, STD_HID_IN_EP, report, 3);
    }
}

bool UsbDevice::send_custom_hid_report(const uint8_t* buffer, size_t length) {
    // 1. Prevent sending a packet that is too large
    if (length > CUSTOM_HID_IN_PACKET) {
        return false;
    }

    // 2. Check if the USB endpoint is ready for another transfer
    if (m_in_transfer_complete) {
        m_in_transfer_complete = false; // Mark transfer busy

        // 3. Prepare the full 64-byte report buffer
        // The low-level driver expects a buffer of the exact endpoint packet size.
        static uint8_t report_buffer[CUSTOM_HID_IN_PACKET] = {0};
        
        // Copy the user's data into the buffer. Unused bytes will be zero.
        memcpy(report_buffer, buffer, length);
        
        // Flush and re-arm the Custom HID OUT endpoint to kick the driver
        // out of its unstable idle state before starting a new IN transfer.
        usbd_fifo_flush(&m_core_driver, CUSTOM_HID_OUT_EP);
        usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 64U);

        // 4. Send the entire 64-byte buffer.
        usbd_ep_send(&m_core_driver, CUSTOM_HID_IN_EP, report_buffer, CUSTOM_HID_IN_PACKET);
        return true; // Send initiated successfully
    }

    return false; // Endpoint was not ready
}

bool UsbDevice::is_std_hid_transfer_complete() {
    return m_in_transfer_complete;
}


// --- Composite Dispatcher Methods ---
uint8_t UsbDevice::_init_composite(uint8_t config_index) {
    (void)config_index;
    m_core_driver.dev.class_data[STD_HID_INTERFACE]    = &m_std_hid_handler;
    m_core_driver.dev.class_data[CUSTOM_HID_INTERFACE] = &m_custom_hid_handler;
    
    _std_hid_init();
    _custom_hid_init();
    
    return USBD_OK;
}

uint8_t UsbDevice::_deinit_composite(uint8_t config_index) {
    (void)config_index;
    _std_hid_deinit();
    _custom_hid_deinit();

    return USBD_OK;
}

uint8_t UsbDevice::_req_handler(usb::UsbRequest *req) {
    uint8_t interface = static_cast<uint8_t>(req->wIndex & 0xFF);

    switch (interface) {
        case STD_HID_INTERFACE:    return _std_hid_req_handler(req);
        case CUSTOM_HID_INTERFACE: return _custom_hid_req_handler(req);
        default:                   break;
    }
    return USBD_FAIL;
}

uint8_t UsbDevice::_data_in(uint8_t ep_num) {
    if (ep_num == (STD_HID_IN_EP & 0x7F) || ep_num == (CUSTOM_HID_IN_EP & 0x7F)) {
        m_in_transfer_complete = true;
        return USBD_OK;
    }
    return USBD_FAIL;
}

uint8_t UsbDevice::_data_out(uint8_t ep_num) {
    if (ep_num == (CUSTOM_HID_OUT_EP & 0x7F)) {
        _custom_hid_data_out();
        return USBD_OK;
    }
    return USBD_FAIL;
}

// --- Static C-style Callbacks ---
uint8_t UsbDevice::_init_cb(usb_dev *udev, uint8_t config_index) { (void)udev; return getInstance()._init_composite(config_index); }
uint8_t UsbDevice::_deinit_cb(usb_dev *udev, uint8_t config_index) { (void)udev; return getInstance()._deinit_composite(config_index); }
uint8_t UsbDevice::_req_handler_cb(usb_dev *udev, usb_req *req) { (void)udev; return getInstance()._req_handler(reinterpret_cast<usb::UsbRequest*>(req)); }
uint8_t UsbDevice::_data_in_cb(usb_dev *udev, uint8_t ep_num) { (void)udev; return getInstance()._data_in(ep_num); }
uint8_t UsbDevice::_data_out_cb(usb_dev *udev, uint8_t ep_num) { (void)udev; return getInstance()._data_out(ep_num); }

// ===================================================================
// Localized Class Implementations
// ===================================================================

// --- Standard HID Implementation ---
void UsbDevice::_std_hid_init() {
    usbd_ep_setup(&m_core_driver, &(composite_config_desc.std_hid_epin));
}

void UsbDevice::_std_hid_deinit() {
    usbd_ep_clear(&m_core_driver, STD_HID_IN_EP);
}

uint8_t UsbDevice::_std_hid_req_handler(usb::UsbRequest *req) {
    // Get the pointer to the control IN transaction state
    usb_transc *transc = &m_core_driver.dev.transc_in[0];

    switch(static_cast<usb::hid::HidReq>(req->bRequest)) {
        case usb::hid::HidReq::GET_REPORT: 
            return USBD_FAIL;

        case usb::hid::HidReq::GET_IDLE:
            transc->xfer_buf = (uint8_t *)&m_std_hid_handler.idle_state;
            transc->remain_len = 1U;
            return USBD_OK;

        case usb::hid::HidReq::GET_PROTOCOL:
            transc->xfer_buf = (uint8_t *)&m_std_hid_handler.protocol;
            transc->remain_len = 1U;
            return USBD_OK;

        case usb::hid::HidReq::SET_REPORT: 
            return USBD_FAIL;

        case usb::hid::HidReq::SET_IDLE: 
            m_std_hid_handler.idle_state = (uint8_t)(req->wValue >> 8); 
            return USBD_OK;
            
        case usb::hid::HidReq::SET_PROTOCOL: 
            m_std_hid_handler.protocol = (uint8_t)(req->wValue); 
            return USBD_OK;

        default:
            if (req->bRequest == USB_GET_DESCRIPTOR) {
                if (usb::hid::DESC_TYPE_REPORT == (req->wValue >> 8)) {
                    transc->remain_len = USB_MIN(STD_HID_REPORT_DESC_LEN, req->wLength);
                    transc->xfer_buf = const_cast<uint8_t*>(std_hid_report_descriptor);
                    return USBD_OK;
                } else if (usb::hid::DESC_TYPE_HID == (req->wValue >> 8)) {
                    transc->remain_len = USB_MIN(sizeof(usb::hid::DescHid), req->wLength);
                    transc->xfer_buf = reinterpret_cast<uint8_t*>(&(composite_config_desc.std_hid_desc));
                    return USBD_OK;
                }
            }
            return USBD_FAIL;
    }
}

// --- Custom HID Implementation ---
void UsbDevice::_custom_hid_init() {
    usbd_ep_setup(&m_core_driver, &(composite_config_desc.custom_hid_epin));
    usbd_ep_setup(&m_core_driver, &(composite_config_desc.custom_hid_epout));
    usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 64U);
}

void UsbDevice::_custom_hid_deinit() {
    usbd_ep_clear(&m_core_driver, CUSTOM_HID_IN_EP);
    usbd_ep_clear(&m_core_driver, CUSTOM_HID_OUT_EP);
}

uint8_t UsbDevice::_custom_hid_req_handler(usb::UsbRequest *req) {
    // Get the pointer to the control IN transaction state
    usb_transc *transc = &m_core_driver.dev.transc_in[0];

    switch(static_cast<usb::hid::HidReq>(req->bRequest)) {
        case usb::hid::HidReq::GET_REPORT: 
            return USBD_FAIL;

        case usb::hid::HidReq::GET_IDLE:
            transc->xfer_buf = (uint8_t *)&m_custom_hid_handler.idlestate;
            transc->remain_len = 1U;
            return USBD_OK;

        case usb::hid::HidReq::GET_PROTOCOL:
            transc->xfer_buf = (uint8_t *)&m_custom_hid_handler.protocol;
            transc->remain_len = 1U;
            return USBD_OK;

        case usb::hid::HidReq::SET_REPORT: 
            return USBD_FAIL;

        case usb::hid::HidReq::SET_IDLE: 
            m_custom_hid_handler.idlestate = (uint8_t)(req->wValue >> 8); 
            return USBD_OK;

        case usb::hid::HidReq::SET_PROTOCOL: 
            m_custom_hid_handler.protocol = (uint8_t)(req->wValue); 
            return USBD_OK;

        default:
            if (req->bRequest == USB_GET_DESCRIPTOR) {
                if (usb::hid::DESC_TYPE_REPORT == (req->wValue >> 8)) {
                    transc->remain_len = USB_MIN(CUSTOM_HID_REPORT_DESC_LEN, req->wLength);
                    transc->xfer_buf = const_cast<uint8_t*>(custom_hid_report_descriptor);
                    return USBD_OK;
                } else if (usb::hid::DESC_TYPE_HID == (req->wValue >> 8)) {
                    transc->remain_len = USB_MIN(sizeof(usb::hid::DescHid), req->wLength);
                    transc->xfer_buf = reinterpret_cast<uint8_t*>(&(composite_config_desc.custom_hid_desc));
                    return USBD_OK;
                }
            }
            return USBD_FAIL;
    }
}

void UsbDevice::_custom_hid_data_out() {
    uint32_t received_count = usbd_rxcount_get(&m_core_driver, CUSTOM_HID_OUT_EP);

    // This debug print is okay for now, but in production, even this should be removed from the ISR.
    // printf("DEBUG RX: size=%lu, data[0]=0x%02X, data[1]=0x%02X\n", ...);

    if (received_count == 0) {
        usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 64U);
        return;
    }

    // We pass the raw data directly to the DisplayManager.
    display::DisplayManager::getInstance().handleUsbPacket(m_custom_hid_handler.data, received_count);

    uint8_t command = m_custom_hid_handler.data[0];
    uint8_t value   = m_custom_hid_handler.data[1];

    switch (command) {
        // LED control logic uses 'value' which is data[1]
        case 0x11: {
            bsp::board::LedRed::set(value != 0);
            break;
        }
        case 0x12: {
            bsp::board::LedGreen::set(value != 0);
            break;
        }
        case 0x13: {
            bsp::board::LedBlue::set(value != 0);
            break;
        }
    }

    // Re-arm the OUT endpoint to receive the next report from the host.
    usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 64U);
}
