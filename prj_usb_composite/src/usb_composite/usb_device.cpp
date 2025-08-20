/*!
    \file    usb_device.cpp
    \brief   Implementation of the C++ USB device abstraction layer

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

#include "usb_device.h"
#include "usbd_msc_mem.h"
#include <cstring>
#include "board.h"

// Forward declare C functions from the library that we will call
extern "C" {
    void usbd_isr(usb_core_driver *udev);
    void usb_timer_irq(void);
    void serial_string_get(uint16_t *unicode_str);
}

// Define the static instance for the singleton
UsbDevice* UsbDevice::s_instance = nullptr;

// ===================================================================
// Public Namespace Functions (The User's API)
// ===================================================================

void usb::init(bool enable_msc) { UsbDevice::getInstance().init(enable_msc); }
void usb::poll() { UsbDevice::getInstance().poll(); }
bool usb::is_configured() { return UsbDevice::getInstance().is_configured(); }
void usb::send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons) { UsbDevice::getInstance().send_mouse_report(x, y, wheel, buttons); }
void usb::send_keyboard_report(uint8_t modifier, uint8_t key) { UsbDevice::getInstance().send_keyboard_report(modifier, key); }
void usb::send_consumer_report(uint16_t usage_code) { UsbDevice::getInstance().send_consumer_report(usage_code); }
void usb::send_custom_hid_report(uint8_t report_id, uint8_t data) { UsbDevice::getInstance().send_custom_hid_report(report_id, data); }

// ===================================================================
// UsbDevice Class Implementation
// ===================================================================

UsbDevice& UsbDevice::getInstance() {
    if (!s_instance) {
        s_instance = new UsbDevice();
    }
    return *s_instance;
}

UsbDevice::UsbDevice() : m_msc_enabled(false) {
    memset(&m_core_driver, 0, sizeof(usb_core_driver));
    memset(&m_class_core, 0, sizeof(usb_class_core));
    memset(&m_descriptors, 0, sizeof(usb_desc));
    memset(&m_std_hid_handler, 0, sizeof(usb::hid::StandardHidHandler));
    memset(&m_custom_hid_handler, 0, sizeof(usb::hid::CustomHidHandler));
    memset(&m_msc_handler, 0, sizeof(usb::msc::MscHandler));

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

void UsbDevice::init(bool enable_msc) {
    m_msc_enabled = enable_msc;

    // --- Dynamic Descriptor Modification ---
    if (!m_msc_enabled) {
        // If MSC is disabled, modify the descriptor before initializing the core.
        // The host will never see the MSC interface.
        composite_config_desc.config.bNumInterfaces = 2; // Only HID interfaces
        composite_config_desc.config.wTotalLength = HID_ONLY_CONFIG_DESC_SIZE;
    }
    
    eclic_global_interrupt_enable();
    eclic_priority_group_set(ECLIC_PRIGROUP_LEVEL2_PRIO2);
    usb_rcu_config();
    usb_timer_init();
    usb_intr_config();
    usbd_init(&m_core_driver, &m_descriptors, &m_class_core);
}

void UsbDevice::poll() { /* For non-interrupt driven logic */ }
bool UsbDevice::is_configured() { return m_core_driver.dev.cur_status == USBD_CONFIGURED; }

// --- ISR Handlers ---
void UsbDevice::isr() { usbd_isr(&m_core_driver); }
void UsbDevice::wakeup_isr() {
    if (m_core_driver.bp.low_power) { /* Resume MCU clock logic here if needed */ }
    exti_interrupt_flag_clear(EXTI_18);
}
void UsbDevice::timer_isr() { usb_timer_irq(); }

// --- Report Sending Methods ---
void UsbDevice::send_mouse_report(int8_t x, int8_t y, int8_t wheel, uint8_t buttons) {
    uint8_t report[5] = {REPORT_ID_MOUSE, buttons, (uint8_t)x, (uint8_t)y, (uint8_t)wheel};
    if (m_std_hid_handler.prev_transfer_complete) {
        m_std_hid_handler.prev_transfer_complete = 0; // Mark endpoint as busy
        usbd_ep_send(&m_core_driver, STD_HID_IN_EP, report, 5);
    }
}

void UsbDevice::send_keyboard_report(uint8_t modifier, uint8_t key) {
    uint8_t report[9] = {REPORT_ID_KEYBOARD, modifier, 0, key, 0, 0, 0, 0, 0};
    if (m_std_hid_handler.prev_transfer_complete) {
        m_std_hid_handler.prev_transfer_complete = 0;
        usbd_ep_send(&m_core_driver, STD_HID_IN_EP, report, 9);
    }
}

void UsbDevice::send_consumer_report(uint16_t usage_code) {
    uint8_t report[3] = {REPORT_ID_CONSUMER, (uint8_t)(usage_code & 0xFF), (uint8_t)(usage_code >> 8)};
    if (m_std_hid_handler.prev_transfer_complete) {
        m_std_hid_handler.prev_transfer_complete = 0;
        usbd_ep_send(&m_core_driver, STD_HID_IN_EP, report, 3);
    }
}

void UsbDevice::send_custom_hid_report(uint8_t report_id, uint8_t data) {
    if (m_custom_hid_handler.prev_transfer_complete) {
        m_custom_hid_handler.prev_transfer_complete = 0; // Mark as busy
        uint8_t report[2] = {report_id, data};
        usbd_ep_send(&m_core_driver, CUSTOM_HID_IN_EP, report, 2);
    }
}

// --- Composite Dispatcher Methods ---
uint8_t UsbDevice::_init_composite(uint8_t config_index) {
    (void)config_index;
    m_core_driver.dev.class_data[STD_HID_INTERFACE]    = &m_std_hid_handler;
    m_core_driver.dev.class_data[CUSTOM_HID_INTERFACE] = &m_custom_hid_handler;
    
    _std_hid_init();
    _custom_hid_init();

    if (m_msc_enabled) {
        m_core_driver.dev.class_data[MSC_INTERFACE] = &m_msc_handler;
        _msc_init();
    }
    
    return USBD_OK;
}

uint8_t UsbDevice::_deinit_composite(uint8_t config_index) {
    (void)config_index;
    _std_hid_deinit();
    _custom_hid_deinit();

    if (m_msc_enabled) {
        _msc_deinit();
    }

    return USBD_OK;
}

uint8_t UsbDevice::_req_handler(usb::UsbRequest *req) {
    /*
     * This is the main dispatcher for class-specific and vendor-specific requests.
     * The core USB library handles all standard requests (like GET_DESCRIPTOR,
     * SET_CONFIGURATION, etc.) *before* calling this function. We only need to
     * route the request to the correct interface handler based on wIndex.
     */
    uint8_t interface = static_cast<uint8_t>(req->wIndex & 0xFF);

    switch (interface) {
        case STD_HID_INTERFACE:    return _std_hid_req_handler(req);
        case CUSTOM_HID_INTERFACE: return _custom_hid_req_handler(req);
        case MSC_INTERFACE:
            if (m_msc_enabled) {
                return _msc_req_handler(req);
            }
            break; // Fall through to fail if MSC is not enabled
        default:
            break;
    }
    return USBD_FAIL;
}

uint8_t UsbDevice::_data_in(uint8_t ep_num) {
    if (ep_num == (STD_HID_IN_EP & 0x7F))    { _std_hid_data_in(); return USBD_OK; }
    if (ep_num == (CUSTOM_HID_IN_EP & 0x7F)) { _custom_hid_data_in(); return USBD_OK; }
    if (m_msc_enabled && (ep_num == (MSC_IN_EP & 0x7F))) {
        _msc_data_in();
        return USBD_OK;
    }
    return USBD_FAIL;
}

uint8_t UsbDevice::_data_out(uint8_t ep_num) {
    if (ep_num == (CUSTOM_HID_OUT_EP & 0x7F)) { _custom_hid_data_out(); return USBD_OK; }
    if (m_msc_enabled && (ep_num == (MSC_OUT_EP & 0x7F))) {
        _msc_data_out();
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
    m_std_hid_handler.prev_transfer_complete = 1U;
}

void UsbDevice::_std_hid_deinit() {
    usbd_ep_clear(&m_core_driver, STD_HID_IN_EP);
}

uint8_t UsbDevice::_std_hid_req_handler(usb::UsbRequest *req) {
    usb_transc *transc = &m_core_driver.dev.transc_in[0];
    switch(static_cast<usb::hid::HidReq>(req->bRequest)) {
        case usb::hid::HidReq::GET_REPORT: break;
        case usb::hid::HidReq::GET_IDLE:
            transc->xfer_buf = (uint8_t *)&m_std_hid_handler.idle_state;
            transc->remain_len = 1U;
            break;
        case usb::hid::HidReq::GET_PROTOCOL:
            transc->xfer_buf = (uint8_t *)&m_std_hid_handler.protocol;
            transc->remain_len = 1U;
            break;
        case usb::hid::HidReq::SET_REPORT: break;
        case usb::hid::HidReq::SET_IDLE: m_std_hid_handler.idle_state = (uint8_t)(req->wValue >> 8); break;
        case usb::hid::HidReq::SET_PROTOCOL: m_std_hid_handler.protocol = (uint8_t)(req->wValue); break;
        default:
            // Handle standard requests like GET_DESCRIPTOR for REPORT
            if (req->bRequest == static_cast<uint8_t>(usb::StdReq::GET_DESCRIPTOR)) {
                if(usb::hid::DESC_TYPE_REPORT == (req->wValue >> 8)) {
                    transc->remain_len = USB_MIN(STD_HID_REPORT_DESC_LEN, req->wLength);
                    transc->xfer_buf = (uint8_t *)std_hid_report_descriptor;
                    return static_cast<uint8_t>(usb::ReqStatus::REQ_SUPP);
                }
            }
            break;
    }
    return USBD_OK;
}

void UsbDevice::_std_hid_data_in() {
    m_std_hid_handler.prev_transfer_complete = 1U;
}

// --- Custom HID Implementation ---
void UsbDevice::_custom_hid_init() {
    usbd_ep_setup(&m_core_driver, &(composite_config_desc.custom_hid_epin));
    usbd_ep_setup(&m_core_driver, &(composite_config_desc.custom_hid_epout));
    usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 2U);
    m_custom_hid_handler.prev_transfer_complete = 1U;
}

void UsbDevice::_custom_hid_deinit() {
    usbd_ep_clear(&m_core_driver, CUSTOM_HID_IN_EP);
    usbd_ep_clear(&m_core_driver, CUSTOM_HID_OUT_EP);
}

uint8_t UsbDevice::_custom_hid_req_handler(usb::UsbRequest *req) {
    usb_transc *transc = &m_core_driver.dev.transc_in[0];
    switch(static_cast<usb::hid::HidReq>(req->bRequest)) {
        case usb::hid::HidReq::GET_REPORT: break;
        case usb::hid::HidReq::GET_IDLE:
            transc->xfer_buf = (uint8_t *)&m_custom_hid_handler.idlestate;
            transc->remain_len = 1U;
            break;
        case usb::hid::HidReq::GET_PROTOCOL:
            transc->xfer_buf = (uint8_t *)&m_custom_hid_handler.protocol;
            transc->remain_len = 1U;
            break;
        case usb::hid::HidReq::SET_REPORT: m_custom_hid_handler.reportID = (uint8_t)(req->wValue); break;
        case usb::hid::HidReq::SET_IDLE: m_custom_hid_handler.idlestate = (uint8_t)(req->wValue >> 8); break;
        case usb::hid::HidReq::SET_PROTOCOL: m_custom_hid_handler.protocol = (uint8_t)(req->wValue); break;
        default:
            if (req->bRequest == static_cast<uint8_t>(usb::StdReq::GET_DESCRIPTOR)) {
                if(usb::hid::DESC_TYPE_REPORT == (req->wValue >> 8)) {
                    transc->remain_len = USB_MIN(CUSTOM_HID_REPORT_DESC_LEN, req->wLength);
                    transc->xfer_buf = (uint8_t *)custom_hid_report_descriptor;
                    return static_cast<uint8_t>(usb::ReqStatus::REQ_SUPP);
                }
            }
            return USBD_FAIL;
    }
    return USBD_OK;
}

void UsbDevice::_custom_hid_data_in() {
    m_custom_hid_handler.prev_transfer_complete = 1U; // Mark as ready for next transfer
}

void UsbDevice::_custom_hid_data_out() {
    // The host has sent us data. The received data is in m_custom_hid_handler.data.
    // The format is [Report ID, Value].

    // Get the report ID and value
    uint8_t report_id = m_custom_hid_handler.data[0];
    uint8_t value = m_custom_hid_handler.data[1];

    // Control the RGB LEDs based on the report ID
    switch (report_id) {
        case 0x11: // Report ID for Red LED
            if (value) {
                gpio_bit_reset(LED_R_GPIO_PORT, LED_R_PIN); // Turn ON (active low)
            } else {
                gpio_bit_set(LED_R_GPIO_PORT, LED_R_PIN); // Turn OFF
            }
            break;

        case 0x12: // Report ID for Green LED
            if (value) {
                gpio_bit_reset(LED_G_GPIO_PORT, LED_G_PIN); // Turn ON (active low)
            } else {
                gpio_bit_set(LED_G_GPIO_PORT, LED_G_PIN); // Turn OFF
            }
            break;

        case 0x13: // Report ID for Blue LED
             if (value) {
                gpio_bit_reset(LED_B_GPIO_PORT, LED_B_PIN); // Turn ON (active low)
            } else {
                gpio_bit_set(LED_B_GPIO_PORT, LED_B_PIN); // Turn OFF
            }
            break;
    }

    // Re-arm the OUT endpoint to receive the next report from the host.
    usbd_ep_recev(&m_core_driver, CUSTOM_HID_OUT_EP, m_custom_hid_handler.data, 2U);
}

// --- MSC Implementation ---
void UsbDevice::_msc_init() {
    usbd_ep_setup(&m_core_driver, &(composite_config_desc.msc_epin));
    usbd_ep_setup(&m_core_driver, &(composite_config_desc.msc_epout));
    _msc_bbb_init();
}

void UsbDevice::_msc_deinit() {
    usbd_ep_clear(&m_core_driver, MSC_IN_EP);
    usbd_ep_clear(&m_core_driver, MSC_OUT_EP);
    _msc_bbb_deinit();
}

uint8_t UsbDevice::_msc_req_handler(usb::UsbRequest *req) {
    usb_transc *transc = &m_core_driver.dev.transc_in[0];

    // Handle class-specific requests targeted to the MSC interface
    if ((req->bmRequestType & USB_RECPTYPE_MASK) == USB_RECPTYPE_ITF) {
        switch (req->bRequest) {
            case usb::msc::REQ_GET_MAX_LUN:
                m_msc_handler.max_lun = (uint8_t)get_msc_mem_fops().mem_maxlun();
                transc->xfer_buf = &m_msc_handler.max_lun;
                transc->remain_len = 1U;
                return USBD_OK; // Success: Data is prepared and ready to send.

            case usb::msc::REQ_BBB_RESET:
                _msc_bbb_reset();
                return USBD_OK; // Success: Reset was performed.

            default:
                // Stall any other unknown class-specific interface requests.
                return USBD_FAIL;
        }
    }

    // Handle standard requests targeted to an MSC endpoint (like clearing a stall)
    if ((req->bmRequestType & USB_RECPTYPE_MASK) == USB_RECPTYPE_EP) {
        if (req->bRequest == static_cast<uint8_t>(usb::StdReq::CLEAR_FEATURE)) {
            _msc_bbb_clrfeature(static_cast<uint8_t>(req->wIndex));
            return USBD_OK; // Success: Stall was cleared.
        }
    }

    // If the request type doesn't match interface or endpoint, it's not for us.
    return USBD_FAIL;
}

void UsbDevice::_msc_data_in() { _msc_bbb_data_in(); }
void UsbDevice::_msc_data_out() { _msc_bbb_data_out(); }

// --- BBB Protocol Implementation ---
void UsbDevice::_msc_bbb_init() {
    m_msc_handler.bbb_state = usb::msc::BbbState::BBB_IDLE;
    m_msc_handler.bbb_status = usb::msc::BbbStatus::BBB_STATUS_NORMAL;
    for(uint8_t lun_num = 0U; lun_num < MEM_LUN_NUM; lun_num++) {
        get_msc_mem_fops().mem_init(lun_num);
    }
    usbd_fifo_flush(&m_core_driver, MSC_OUT_EP);
    usbd_fifo_flush(&m_core_driver, MSC_IN_EP);
    usbd_ep_recev(&m_core_driver, MSC_OUT_EP, (uint8_t *)&m_msc_handler.bbb_cbw, usb::msc::BBB_CBW_LENGTH);
}

void UsbDevice::_msc_bbb_reset() {
    m_msc_handler.bbb_state = usb::msc::BbbState::BBB_IDLE;
    m_msc_handler.bbb_status = usb::msc::BbbStatus::BBB_STATUS_RECOVERY;
    usbd_ep_recev(&m_core_driver, MSC_OUT_EP, (uint8_t *)&m_msc_handler.bbb_cbw, usb::msc::BBB_CBW_LENGTH);
}

void UsbDevice::_msc_bbb_deinit() {
    m_msc_handler.bbb_state = usb::msc::BbbState::BBB_IDLE;
}

void UsbDevice::_msc_bbb_data_in() {
    switch(m_msc_handler.bbb_state) {
        case usb::msc::BbbState::BBB_DATA_IN:
            if(_scsi_process_cmd(m_msc_handler.bbb_cbw.bCBWLUN, &m_msc_handler.bbb_cbw.CBWCB[0]) < 0) {
                _msc_bbb_csw_send(usb::msc::CswStatus::CMD_FAILED);
            }
            break;
        case usb::msc::BbbState::BBB_SEND_DATA:
        case usb::msc::BbbState::BBB_LAST_DATA_IN:
            _msc_bbb_csw_send(usb::msc::CswStatus::CMD_PASSED);
            break;
        default:
            break;
    }
}

void UsbDevice::_msc_bbb_data_out() {
    switch(m_msc_handler.bbb_state) {
        case usb::msc::BbbState::BBB_IDLE:
            _msc_bbb_cbw_decode();
            break;
        case usb::msc::BbbState::BBB_DATA_OUT:
            if(_scsi_process_cmd(m_msc_handler.bbb_cbw.bCBWLUN, &m_msc_handler.bbb_cbw.CBWCB[0]) < 0) {
                _msc_bbb_csw_send(usb::msc::CswStatus::CMD_FAILED);
            }
            break;
        default:
            break;
    }
}

void UsbDevice::_msc_bbb_cbw_decode() {
    m_msc_handler.bbb_csw.dCSWTag = m_msc_handler.bbb_cbw.dCBWTag;
    m_msc_handler.bbb_csw.dCSWDataResidue = m_msc_handler.bbb_cbw.dCBWDataTransferLength;

    if((usb::msc::BBB_CBW_LENGTH != usbd_rxcount_get(&m_core_driver, MSC_OUT_EP)) || 
       (usb::msc::BBB_CBW_SIGNATURE != m_msc_handler.bbb_cbw.dCBWSignature) || 
       (m_msc_handler.bbb_cbw.bCBWLUN > 1U) || 
       (m_msc_handler.bbb_cbw.bCBWCBLength < 1U) || 
       (m_msc_handler.bbb_cbw.bCBWCBLength > 16U)) {
        _scsi_sense_code(m_msc_handler.bbb_cbw.bCBWLUN, usb::msc::scsi::SenseKey::ILLEGAL_REQUEST, usb::msc::scsi::Asc::INVALID_CDB);
        m_msc_handler.bbb_status = usb::msc::BbbStatus::BBB_STATUS_ERROR;
        _msc_bbb_abort();
    } else {
        if(_scsi_process_cmd(m_msc_handler.bbb_cbw.bCBWLUN, &m_msc_handler.bbb_cbw.CBWCB[0]) < 0) {
            _msc_bbb_abort();
        } else if((usb::msc::BbbState::BBB_DATA_IN != m_msc_handler.bbb_state) && 
                  (usb::msc::BbbState::BBB_DATA_OUT != m_msc_handler.bbb_state) && 
                  (usb::msc::BbbState::BBB_LAST_DATA_IN != m_msc_handler.bbb_state)) {
            if(m_msc_handler.bbb_datalen > 0U) {
                _msc_bbb_data_send(m_msc_handler.bbb_data, m_msc_handler.bbb_datalen);
            } else {
                _msc_bbb_csw_send(usb::msc::CswStatus::CMD_PASSED);
            }
        }
    }
}

void UsbDevice::_msc_bbb_csw_send(usb::msc::CswStatus csw_status) {
    m_msc_handler.bbb_csw.dCSWSignature = usb::msc::BBB_CSW_SIGNATURE;
    m_msc_handler.bbb_csw.bCSWStatus = static_cast<uint8_t>(csw_status);
    m_msc_handler.bbb_state = usb::msc::BbbState::BBB_IDLE;

    usbd_ep_send(&m_core_driver, MSC_IN_EP, (uint8_t *)&m_msc_handler.bbb_csw, usb::msc::BBB_CSW_LENGTH);
    usbd_ep_recev(&m_core_driver, MSC_OUT_EP, (uint8_t *)&m_msc_handler.bbb_cbw, usb::msc::BBB_CBW_LENGTH);
}

void UsbDevice::_msc_bbb_clrfeature(uint8_t ep_num) {
    if(usb::msc::BbbStatus::BBB_STATUS_ERROR == m_msc_handler.bbb_status) {
        usbd_ep_stall(&m_core_driver, MSC_IN_EP);
        m_msc_handler.bbb_status = usb::msc::BbbStatus::BBB_STATUS_NORMAL;
    } else if((0x80U == (ep_num & 0x80U)) && (usb::msc::BbbStatus::BBB_STATUS_RECOVERY != m_msc_handler.bbb_status)) {
        _msc_bbb_csw_send(usb::msc::CswStatus::CMD_FAILED);
    }
}

void UsbDevice::_msc_bbb_data_send(uint8_t *pbuf, uint32_t len) {
    len = USB_MIN(m_msc_handler.bbb_cbw.dCBWDataTransferLength, len);
    m_msc_handler.bbb_csw.dCSWDataResidue -= len;
    m_msc_handler.bbb_csw.bCSWStatus = static_cast<uint8_t>(usb::msc::CswStatus::CMD_PASSED);
    m_msc_handler.bbb_state = usb::msc::BbbState::BBB_SEND_DATA;
    usbd_ep_send(&m_core_driver, MSC_IN_EP, pbuf, len);
}

void UsbDevice::_msc_bbb_abort() {
    if((0U == m_msc_handler.bbb_cbw.bmCBWFlags) && 
       (0U != m_msc_handler.bbb_cbw.dCBWDataTransferLength) && 
       (usb::msc::BbbStatus::BBB_STATUS_NORMAL == m_msc_handler.bbb_status)) {
        usbd_ep_stall(&m_core_driver, MSC_OUT_EP);
    }
    usbd_ep_stall(&m_core_driver, MSC_IN_EP);
    if(usb::msc::BbbStatus::BBB_STATUS_ERROR == m_msc_handler.bbb_status) {
        usbd_ep_recev(&m_core_driver, MSC_OUT_EP, (uint8_t *)&m_msc_handler.bbb_cbw, usb::msc::BBB_CBW_LENGTH);
    }
}

// --- SCSI Command Implementation ---
int8_t UsbDevice::_scsi_process_cmd(uint8_t lun, uint8_t *params) {
    using namespace usb::msc::scsi;
    switch(static_cast<Command>(params[0])) {
        case Command::TEST_UNIT_READY:        return _scsi_test_unit_ready(lun);
        case Command::REQUEST_SENSE:          return _scsi_request_sense(lun, params);
        case Command::INQUIRY:                return _scsi_inquiry(lun, params);
        case Command::MODE_SENSE_6:           return _scsi_mode_sense6(lun);
        case Command::MODE_SENSE_10:          return _scsi_mode_sense10(lun);
        case Command::READ_FORMAT_CAPACITIES: return _scsi_read_format_capacity(lun);
        case Command::READ_CAPACITY_10:       return _scsi_read_capacity10(lun);
        case Command::READ_10:                return _scsi_read10(lun, params);
        case Command::WRITE_10:               return _scsi_write10(lun, params);
        case Command::VERIFY_10:              return _scsi_verify10(lun);
        case Command::START_STOP_UNIT:
        case Command::ALLOW_MEDIUM_REMOVAL:
            m_msc_handler.bbb_datalen = 0U;
            return 0;
        default:
            _scsi_sense_code(lun, SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
            return -1;
    }
}

void UsbDevice::_scsi_sense_code(uint8_t lun, usb::msc::scsi::SenseKey skey, usb::msc::scsi::Asc asc) {
    (void)lun; // Unused parameter
    m_msc_handler.scsi_sense[m_msc_handler.scsi_sense_tail].key = skey;
    m_msc_handler.scsi_sense[m_msc_handler.scsi_sense_tail].ASC = static_cast<uint8_t>(asc);
    m_msc_handler.scsi_sense_tail++;
    if(usb::msc::scsi::SENSE_LIST_DEEPTH == m_msc_handler.scsi_sense_tail) {
        m_msc_handler.scsi_sense_tail = 0U;
    }
}

int8_t UsbDevice::_scsi_test_unit_ready(uint8_t lun) {
    if(0U != m_msc_handler.bbb_cbw.dCBWDataTransferLength) {
        _scsi_sense_code(m_msc_handler.bbb_cbw.bCBWLUN, usb::msc::scsi::SenseKey::ILLEGAL_REQUEST, usb::msc::scsi::Asc::INVALID_CDB);
        return -1;
    }
    if(0 != get_msc_mem_fops().mem_ready(lun)) {
        _scsi_sense_code(lun, usb::msc::scsi::SenseKey::NOT_READY, usb::msc::scsi::Asc::MEDIUM_NOT_PRESENT);
        return -1;
    }
    m_msc_handler.bbb_datalen = 0U;
    return 0;
}

int8_t UsbDevice::_scsi_inquiry(uint8_t lun, uint8_t *params) {
    uint8_t* page = (uint8_t *)get_msc_mem_fops().mem_inquiry_data[lun];
    uint16_t len = (uint16_t)(page[4] + 5U);
    if(params[4] <= len) {
        len = params[4];
    }
    m_msc_handler.bbb_datalen = len;
    memcpy(m_msc_handler.bbb_data, page, len);
    return 0;
}

int8_t UsbDevice::_scsi_read_capacity10(uint8_t lun) {
    uint32_t blk_num = get_msc_mem_fops().mem_block_len[lun] - 1U;
    m_msc_handler.scsi_blk_nbr[lun] = get_msc_mem_fops().mem_block_len[lun];
    m_msc_handler.scsi_blk_size[lun] = get_msc_mem_fops().mem_block_size[lun];
    m_msc_handler.bbb_data[0] = (uint8_t)(blk_num >> 24);
    m_msc_handler.bbb_data[1] = (uint8_t)(blk_num >> 16);
    m_msc_handler.bbb_data[2] = (uint8_t)(blk_num >> 8);
    m_msc_handler.bbb_data[3] = (uint8_t)(blk_num);
    m_msc_handler.bbb_data[4] = (uint8_t)(m_msc_handler.scsi_blk_size[lun] >> 24);
    m_msc_handler.bbb_data[5] = (uint8_t)(m_msc_handler.scsi_blk_size[lun] >> 16);
    m_msc_handler.bbb_data[6] = (uint8_t)(m_msc_handler.scsi_blk_size[lun] >> 8);
    m_msc_handler.bbb_data[7] = (uint8_t)(m_msc_handler.scsi_blk_size[lun]);
    m_msc_handler.bbb_datalen = 8U;
    return 0;
}

int8_t UsbDevice::_scsi_read10(uint8_t lun, uint8_t *params) {
    if(usb::msc::BbbState::BBB_IDLE == m_msc_handler.bbb_state) {
        if(0x80U != (m_msc_handler.bbb_cbw.bmCBWFlags & 0x80U)) {
            _scsi_sense_code(m_msc_handler.bbb_cbw.bCBWLUN, usb::msc::scsi::SenseKey::ILLEGAL_REQUEST, usb::msc::scsi::Asc::INVALID_CDB);
            return -1;
        }
        if(0 != get_msc_mem_fops().mem_ready(lun)) {
            _scsi_sense_code(lun, usb::msc::scsi::SenseKey::NOT_READY, usb::msc::scsi::Asc::MEDIUM_NOT_PRESENT);
            return -1;
        }
        m_msc_handler.scsi_blk_addr = (params[2] << 24) | (params[3] << 16) | (params[4] << 8) | params[5];
        m_msc_handler.scsi_blk_len = (params[7] << 8) | params[8];
        if(_scsi_check_address_range(lun, m_msc_handler.scsi_blk_addr, (uint16_t)m_msc_handler.scsi_blk_len) < 0) {
            return -1;
        }
        m_msc_handler.bbb_state = usb::msc::BbbState::BBB_DATA_IN;
        m_msc_handler.scsi_blk_addr *= m_msc_handler.scsi_blk_size[lun];
        m_msc_handler.scsi_blk_len  *= m_msc_handler.scsi_blk_size[lun];
        if(m_msc_handler.bbb_cbw.dCBWDataTransferLength != m_msc_handler.scsi_blk_len) {
            _scsi_sense_code(m_msc_handler.bbb_cbw.bCBWLUN, usb::msc::scsi::SenseKey::ILLEGAL_REQUEST, usb::msc::scsi::Asc::INVALID_CDB);
            return -1;
        }
    }
    m_msc_handler.bbb_datalen = MSC_MEDIA_PACKET_SIZE;
    return _scsi_process_read(lun);
}

int8_t UsbDevice::_scsi_write10(uint8_t lun, uint8_t *params) {
    if(usb::msc::BbbState::BBB_IDLE == m_msc_handler.bbb_state) {
        if(0x80U == (m_msc_handler.bbb_cbw.bmCBWFlags & 0x80U)) {
            _scsi_sense_code(m_msc_handler.bbb_cbw.bCBWLUN, usb::msc::scsi::SenseKey::ILLEGAL_REQUEST, usb::msc::scsi::Asc::INVALID_CDB);
            return -1;
        }
        if(0 != get_msc_mem_fops().mem_ready(lun)) {
            _scsi_sense_code(lun, usb::msc::scsi::SenseKey::NOT_READY, usb::msc::scsi::Asc::MEDIUM_NOT_PRESENT);
            return -1;
        }
        if(0 != get_msc_mem_fops().mem_protected(lun)) {
            _scsi_sense_code(lun, usb::msc::scsi::SenseKey::NOT_READY, usb::msc::scsi::Asc::WRITE_PROTECTED);
            return -1;
        }
        m_msc_handler.scsi_blk_addr = (params[2] << 24) | (params[3] << 16) | (params[4] << 8) | params[5];
        m_msc_handler.scsi_blk_len = (params[7] << 8) | params[8];
        if(_scsi_check_address_range(lun, m_msc_handler.scsi_blk_addr, (uint16_t)m_msc_handler.scsi_blk_len) < 0) {
            return -1;
        }
        m_msc_handler.scsi_blk_addr *= m_msc_handler.scsi_blk_size[lun];
        m_msc_handler.scsi_blk_len  *= m_msc_handler.scsi_blk_size[lun];
        if(m_msc_handler.bbb_cbw.dCBWDataTransferLength != m_msc_handler.scsi_blk_len) {
            _scsi_sense_code(m_msc_handler.bbb_cbw.bCBWLUN, usb::msc::scsi::SenseKey::ILLEGAL_REQUEST, usb::msc::scsi::Asc::INVALID_CDB);
            return -1;
        }
        m_msc_handler.bbb_state = usb::msc::BbbState::BBB_DATA_OUT;
        usbd_ep_recev(&m_core_driver, MSC_OUT_EP, m_msc_handler.bbb_data, USB_MIN(m_msc_handler.scsi_blk_len, MSC_MEDIA_PACKET_SIZE));
    } else {
        return _scsi_process_write(lun);
    }
    return 0;
}

int8_t UsbDevice::_scsi_process_read(uint8_t lun) {
    uint32_t len = USB_MIN(m_msc_handler.scsi_blk_len, MSC_MEDIA_PACKET_SIZE);
    if(get_msc_mem_fops().mem_read(lun, m_msc_handler.bbb_data, m_msc_handler.scsi_blk_addr, (uint16_t)(len / m_msc_handler.scsi_blk_size[lun])) < 0) {
        _scsi_sense_code(lun, usb::msc::scsi::SenseKey::HARDWARE_ERROR, usb::msc::scsi::Asc::UNRECOVERED_READ_ERROR);
        return -1;
    }
    usbd_ep_send(&m_core_driver, MSC_IN_EP, m_msc_handler.bbb_data, len);
    m_msc_handler.scsi_blk_addr += len;
    m_msc_handler.scsi_blk_len  -= len;
    m_msc_handler.bbb_csw.dCSWDataResidue -= len;
    if(0U == m_msc_handler.scsi_blk_len) {
        m_msc_handler.bbb_state = usb::msc::BbbState::BBB_LAST_DATA_IN;
    }
    return 0;
}

int8_t UsbDevice::_scsi_process_write(uint8_t lun) {
    uint32_t len = USB_MIN(m_msc_handler.scsi_blk_len, MSC_MEDIA_PACKET_SIZE);
    if(get_msc_mem_fops().mem_write(lun, m_msc_handler.bbb_data, m_msc_handler.scsi_blk_addr, (uint16_t)(len / m_msc_handler.scsi_blk_size[lun])) < 0) {
        _scsi_sense_code(lun, usb::msc::scsi::SenseKey::HARDWARE_ERROR, usb::msc::scsi::Asc::WRITE_FAULT);
        return -1;
    }
    m_msc_handler.scsi_blk_addr += len;
    m_msc_handler.scsi_blk_len  -= len;
    m_msc_handler.bbb_csw.dCSWDataResidue -= len;
    if(0U == m_msc_handler.scsi_blk_len) {
        _msc_bbb_csw_send(usb::msc::CswStatus::CMD_PASSED);
    } else {
        usbd_ep_recev(&m_core_driver, MSC_OUT_EP, m_msc_handler.bbb_data, USB_MIN(m_msc_handler.scsi_blk_len, MSC_MEDIA_PACKET_SIZE));
    }
    return 0;
}

int8_t UsbDevice::_scsi_check_address_range(uint8_t lun, uint32_t blk_offset, uint16_t blk_nbr) {
    if((blk_offset + blk_nbr) > m_msc_handler.scsi_blk_nbr[lun]) {
        _scsi_sense_code(lun, usb::msc::scsi::SenseKey::ILLEGAL_REQUEST, usb::msc::scsi::Asc::ADDRESS_OUT_OF_RANGE);
        return -1;
    }
    return 0;
}

// Dummy implementations for other SCSI commands to satisfy the switch case
int8_t UsbDevice::_scsi_read_format_capacity(uint8_t lun) { (void)lun; m_msc_handler.bbb_datalen = 0U; return 0; }
int8_t UsbDevice::_scsi_mode_sense6(uint8_t lun) { (void)lun; m_msc_handler.bbb_datalen = 0U; return 0; }
int8_t UsbDevice::_scsi_mode_sense10(uint8_t lun) { (void)lun; m_msc_handler.bbb_datalen = 0U; return 0; }
int8_t UsbDevice::_scsi_request_sense(uint8_t lun, uint8_t *params) { (void)lun; (void)params; m_msc_handler.bbb_datalen = 0U; return 0; }
int8_t UsbDevice::_scsi_verify10(uint8_t lun) { (void)lun; m_msc_handler.bbb_datalen = 0U; return 0; }