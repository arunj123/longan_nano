#include "usbd_enum.h"
#include "usbd_transc.h"
#include "hal/time.hpp"
#include <algorithm>
#include <cstring>

/**
 * @file usbd_enum.cpp
 * @brief Native modern C++23 USB Chapter 9 standard enumeration engine.
 */

inline constexpr uint32_t  USB_STRING_COUNT = 4U;

inline constexpr uintptr_t DEVICE_ID1 = 0x1FFFF7E8U;
inline constexpr uintptr_t DEVICE_ID2 = 0x1FFFF7ECU;
inline constexpr uintptr_t DEVICE_ID3 = 0x1FFFF7F0U;
inline constexpr uintptr_t DEVICE_ID  = 0x40022100U;

static uint16_t g_status_val = 0U;

void usbd_enum_error(usb_core_driver* udev, usb_req* req) {
    udev->dev.control.ctl_state = 0U; // USB_CTL_IDLE

    if (req != nullptr && (req->bmRequestType & 0x80U)) {
        usbd_ep_stall(udev, 0x80U);
    } else {
        usbd_ep_stall(udev, 0x00U);
    }

    usb_ctlep_startout(udev);
}

void int_to_unicode(uint32_t value, uint8_t* pbuf, uint8_t len) {
    for (uint8_t index = 0U; index < len; ++index) {
        if ((value >> 28) < 0x0AU) {
            pbuf[2 * index] = static_cast<uint8_t>((value >> 28) + '0');
        } else {
            pbuf[2 * index] = static_cast<uint8_t>((value >> 28) + 'A' - 10U);
        }
        value <<= 4;
        pbuf[2U * index + 1U] = 0U;
    }
}

void serial_string_get(uint16_t* unicode_str) {
    if (unicode_str[1] != 0) {
        return; // Custom serial number already set by application
    }

    if (6U != (unicode_str[0] & 0x00FFU)) {
        uint32_t d0 = *reinterpret_cast<const volatile uint32_t*>(DEVICE_ID1);
        uint32_t d1 = *reinterpret_cast<const volatile uint32_t*>(DEVICE_ID2);
        uint32_t d2 = *reinterpret_cast<const volatile uint32_t*>(DEVICE_ID3);
        d0 += d2;

        if (d0 != 0U) {
            int_to_unicode(d0, reinterpret_cast<uint8_t*>(&unicode_str[1]), 8U);
            int_to_unicode(d1, reinterpret_cast<uint8_t*>(&unicode_str[9]), 4U);
        }
    } else {
        uint32_t d = *reinterpret_cast<const volatile uint32_t*>(DEVICE_ID);
        if (d != 0U) {
            unicode_str[1] = static_cast<uint16_t>(d & 0xFFFFU);
            unicode_str[2] = static_cast<uint16_t>((d >> 16) & 0xFFFFU);
        }
    }
}

static usb_reqsta handle_get_status(usb_core_driver* udev, usb_req* req) {
    g_status_val = 0U;

    switch (req->bmRequestType & USB_RECPTYPE_MASK) {
        case USB_RECPTYPE_DEV:
            if (udev->dev.cur_status == USBD_ADDRESSED || udev->dev.cur_status == USBD_CONFIGURED) {
                if (udev->dev.pm.dev_remote_wakeup) {
                    g_status_val |= USB_STATUS_REMOTE_WAKEUP;
                }
                g_status_val |= USB_STATUS_SELF_POWERED;
            }
            break;

        case USB_RECPTYPE_ITF:
            if (udev->dev.cur_status != USBD_CONFIGURED) {
                return REQ_NOTSUPP;
            }
            break;

        case USB_RECPTYPE_EP: {
            uint8_t ep_addr = static_cast<uint8_t>(req->wIndex);
            uint8_t ep_num = ep_id(ep_addr);
            if (ep_num >= 4U) return REQ_NOTSUPP;

            if (ep_dir(ep_addr)) {
                if (udev->regs.er_in[ep_num]->DIEPCTL & DEPCTL_STALL) {
                    g_status_val = 1U;
                }
            } else {
                if (udev->regs.er_out[ep_num]->DOEPCTL & DEPCTL_STALL) {
                    g_status_val = 1U;
                }
            }
            break;
        }

        default:
            return REQ_NOTSUPP;
    }

    usb_transc* transc = &udev->dev.transc_in[0];
    transc->xfer_buf = reinterpret_cast<uint8_t*>(&g_status_val);
    transc->remain_len = 2U;
    return REQ_SUPP;
}

static usb_reqsta handle_clear_feature(usb_core_driver* udev, usb_req* req) {
    switch (req->bmRequestType & USB_RECPTYPE_MASK) {
        case USB_RECPTYPE_DEV:
            if (req->wValue == FEATURE_SELECTOR_REMOTEWAKEUP) {
                udev->dev.pm.dev_remote_wakeup = 0U;
                return REQ_SUPP;
            }
            break;

        case USB_RECPTYPE_EP:
            if (req->wValue == FEATURE_SELECTOR_EP) {
                uint8_t ep_addr = static_cast<uint8_t>(req->wIndex);
                uint8_t ep_num = ep_id(ep_addr);
                if (ep_num >= 4U) return REQ_NOTSUPP;

                usbd_ep_stall_clear(udev, ep_addr);
                return REQ_SUPP;
            }
            break;

        default:
            break;
    }

    return REQ_NOTSUPP;
}

static usb_reqsta handle_set_feature(usb_core_driver* udev, usb_req* req) {
    switch (req->bmRequestType & USB_RECPTYPE_MASK) {
        case USB_RECPTYPE_DEV:
            if (req->wValue == FEATURE_SELECTOR_REMOTEWAKEUP) {
                udev->dev.pm.dev_remote_wakeup = 1U;
                return REQ_SUPP;
            }
            break;

        case USB_RECPTYPE_EP:
            if (req->wValue == FEATURE_SELECTOR_EP) {
                uint8_t ep_addr = static_cast<uint8_t>(req->wIndex);
                uint8_t ep_num = ep_id(ep_addr);
                if (ep_num >= 4U) return REQ_NOTSUPP;

                usbd_ep_stall(udev, ep_addr);
                return REQ_SUPP;
            }
            break;

        default:
            break;
    }

    return REQ_NOTSUPP;
}

static usb_reqsta handle_set_address(usb_core_driver* udev, usb_req* req) {
    if (0U == req->wIndex && 0U == req->wLength) {
        udev->dev.dev_addr = static_cast<uint8_t>(req->wValue & 0x7FU);
        if (udev->dev.cur_status != USBD_CONFIGURED) {
            usbd_addr_set(udev, udev->dev.dev_addr);
            udev->dev.cur_status = (udev->dev.dev_addr != 0U) ? USBD_ADDRESSED : USBD_DEFAULT;
            return REQ_SUPP;
        }
    }
    return REQ_NOTSUPP;
}

static usb_reqsta handle_get_descriptor(usb_core_driver* udev, usb_req* req) {
    uint8_t desc_type = byte_high(req->wValue);
    uint8_t desc_index = byte_low(req->wValue);

    usb_transc* transc = &udev->dev.transc_in[0];
    transc->remain_len = 0U;
    transc->xfer_buf = nullptr;
    udev->dev.control.ctl_zlp = 0U;

    switch (req->bmRequestType & USB_RECPTYPE_MASK) {
        case USB_RECPTYPE_DEV:
            switch (desc_type) {
                case USB_DESCTYPE_DEV:
                    transc->xfer_buf = udev->dev.desc->dev_desc;
                    transc->remain_len = udev->dev.desc->dev_desc[0];

                    // Windows xHCI Initial 8-Byte Probe Invariant
                    if (64U == req->wLength) {
                        transc->remain_len = 8U;
                    }
                    break;

                case USB_DESCTYPE_CONFIG:
                    transc->xfer_buf = udev->dev.desc->config_desc;
                    transc->remain_len = static_cast<uint16_t>(udev->dev.desc->config_desc[2] | (udev->dev.desc->config_desc[3] << 8));
                    break;

                case USB_DESCTYPE_STR:
                    if (desc_index < USB_STRING_COUNT && udev->dev.desc->strings[desc_index] != nullptr) {
                        transc->xfer_buf = reinterpret_cast<uint8_t*>(const_cast<void*>(udev->dev.desc->strings[desc_index]));
                        transc->remain_len = transc->xfer_buf[0];
                    } else {
                        return REQ_NOTSUPP;
                    }
                    break;

                case USB_DESCTYPE_BOS:
                    if (udev->dev.desc->bos_desc != nullptr) {
                        transc->xfer_buf = udev->dev.desc->bos_desc;
                        transc->remain_len = udev->dev.desc->bos_desc[2];
                    } else {
                        return REQ_NOTSUPP;
                    }
                    break;

                default:
                    return REQ_NOTSUPP;
            }
            break;

        case USB_RECPTYPE_ITF:
            if (udev->dev.class_core && udev->dev.class_core->req_proc) {
                auto status = udev->dev.class_core->req_proc(udev, req);
                if (status != 0U || 0U == transc->remain_len || transc->xfer_buf == nullptr) {
                    transc->remain_len = 0U;
                    transc->xfer_buf = nullptr;
                    return REQ_NOTSUPP;
                }
            } else {
                return REQ_NOTSUPP;
            }
            break;

        default:
            return REQ_NOTSUPP;
    }

    if (transc->xfer_buf != nullptr && transc->remain_len != 0U && req->wLength != 0U) {
        if (transc->remain_len < req->wLength) {
            if (transc->remain_len >= transc->max_len && (transc->remain_len % transc->max_len == 0U)) {
                udev->dev.control.ctl_zlp = 1U;
            }
        } else {
            transc->remain_len = req->wLength;
        }
        return REQ_SUPP;
    }

    return REQ_NOTSUPP;
}

static uint8_t g_config_val = 0U;

static usb_reqsta handle_get_configuration(usb_core_driver* udev, [[maybe_unused]] usb_req* req) {
    g_config_val = udev->dev.config;
    usb_transc* transc = &udev->dev.transc_in[0];
    transc->xfer_buf = &g_config_val;
    transc->remain_len = 1U;
    return REQ_SUPP;
}

static usb_reqsta handle_set_configuration(usb_core_driver* udev, usb_req* req) {
    uint8_t cfg_idx = static_cast<uint8_t>(req->wValue);

    if (cfg_idx > 1U) {
        return REQ_NOTSUPP;
    }

    switch (udev->dev.cur_status) {
        case USBD_ADDRESSED:
            if (cfg_idx) {
                udev->dev.config = cfg_idx;
                udev->dev.cur_status = USBD_CONFIGURED;
                if (udev->dev.class_core && udev->dev.class_core->init) {
                    udev->dev.class_core->init(udev, cfg_idx);
                }
            }
            return REQ_SUPP;

        case USBD_CONFIGURED:
            if (0U == cfg_idx) {
                udev->dev.config = cfg_idx;
                udev->dev.cur_status = USBD_ADDRESSED;
                if (udev->dev.class_core && udev->dev.class_core->deinit) {
                    udev->dev.class_core->deinit(udev, cfg_idx);
                }
            }
            return REQ_SUPP;

        default:
            break;
    }

    return REQ_NOTSUPP;
}

static uint8_t g_alt_setting = 0U;

static usb_reqsta handle_get_interface(usb_core_driver* udev, [[maybe_unused]] usb_req* req) {
    if (udev->dev.cur_status != USBD_CONFIGURED) {
        return REQ_NOTSUPP;
    }
    g_alt_setting = 0U;
    usb_transc* transc = &udev->dev.transc_in[0];
    transc->xfer_buf = &g_alt_setting;
    transc->remain_len = 1U;
    return REQ_SUPP;
}

static usb_reqsta handle_set_interface(usb_core_driver* udev, usb_req* req) {
    if (udev->dev.cur_status == USBD_CONFIGURED) {
        if (udev->dev.class_core && udev->dev.class_core->set_intf) {
            udev->dev.class_core->set_intf(udev, req);
        }
        return REQ_SUPP;
    }
    return REQ_NOTSUPP;
}

usb_reqsta usbd_standard_request(usb_core_driver* udev, usb_req* req) {
    switch (req->bRequest) {
        case USB_GET_STATUS:        return handle_get_status(udev, req);
        case USB_CLEAR_FEATURE:     return handle_clear_feature(udev, req);
        case USB_SET_FEATURE:       return handle_set_feature(udev, req);
        case USB_SET_ADDRESS:       return handle_set_address(udev, req);
        case USB_GET_DESCRIPTOR:    return handle_get_descriptor(udev, req);
        case USB_GET_CONFIGURATION: return handle_get_configuration(udev, req);
        case USB_SET_CONFIGURATION: return handle_set_configuration(udev, req);
        case USB_GET_INTERFACE:     return handle_get_interface(udev, req);
        case USB_SET_INTERFACE:     return handle_set_interface(udev, req);
        case USB_SYNCH_FRAME:       return REQ_SUPP;
        default:                    return REQ_NOTSUPP;
    }
}

usb_reqsta usbd_class_request(usb_core_driver* udev, usb_req* req) {
    if (USBD_CONFIGURED == udev->dev.cur_status && udev->dev.class_core && udev->dev.class_core->req_proc) {
        return static_cast<usb_reqsta>(udev->dev.class_core->req_proc(udev, req));
    }
    return REQ_NOTSUPP;
}

usb_reqsta usbd_vendor_request([[maybe_unused]] usb_core_driver* udev, [[maybe_unused]] usb_req* req) {
    return REQ_NOTSUPP;
}
