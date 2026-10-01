#include "usbd_transc.h"
#include "usbd_enum.h"
#include <cstring>

/**
 * @file usbd_transc.cpp
 * @brief Native modern C++23 USB control and bulk transaction stage sequencer.
 */

UsbTraceEntry g_usb_trace[USB_TRACE_MAX] = {};
volatile uint8_t g_usb_trace_head = 0;
volatile uint8_t g_usb_trace_tail = 0;

void usb_trace_record(uint8_t type, uint8_t ep_num, uint8_t ctl_state, uint8_t status, uint16_t val1, uint16_t val2, const uint8_t* extra) {
    uint8_t next_head = (g_usb_trace_head + 1U) % USB_TRACE_MAX;
    if (next_head != g_usb_trace_tail) {
        UsbTraceEntry& e = g_usb_trace[g_usb_trace_head];
        e.type = type;
        e.ep_num = ep_num;
        e.ctl_state = ctl_state;
        e.status = status;
        e.val1 = val1;
        e.val2 = val2;
        if (extra != nullptr) {
            e.extra[0] = extra[0];
            e.extra[1] = extra[1];
            e.extra[2] = extra[2];
            e.extra[3] = extra[3];
        } else {
            e.extra[0] = e.extra[1] = e.extra[2] = e.extra[3] = 0;
        }
        g_usb_trace_head = next_head;
    }
}

usbd_status usbd_ctl_send(usb_core_driver* udev) {
    usb_transc* transc = &udev->dev.transc_in[0];
    usbd_ep_send(udev, 0U, transc->xfer_buf, transc->remain_len);

    if (transc->remain_len > transc->max_len) {
        udev->dev.control.ctl_state = USB_CTL_DATA_IN;
    } else {
        udev->dev.control.ctl_state = USB_CTL_LAST_DATA_IN;
    }

    return USBD_OK;
}

usbd_status usbd_ctl_recev(usb_core_driver* udev) {
    usb_transc* transc = &udev->dev.transc_out[0];
    usbd_ep_recev(udev, 0U, transc->xfer_buf, transc->remain_len);

    if (transc->remain_len > transc->max_len) {
        udev->dev.control.ctl_state = USB_CTL_DATA_OUT;
    } else {
        udev->dev.control.ctl_state = USB_CTL_LAST_DATA_OUT;
    }

    return USBD_OK;
}

usbd_status usbd_ctl_status_send(usb_core_driver* udev) {
    udev->dev.control.ctl_state = USB_CTL_STATUS_IN;
    usbd_ep_send(udev, 0U, nullptr, 0U);
    return USBD_OK;
}

usbd_status usbd_ctl_status_recev(usb_core_driver* udev) {
    udev->dev.control.ctl_state = USB_CTL_STATUS_OUT;
    usbd_ep_recev(udev, 0U, nullptr, 0U);
    usb_trace_record(4, 0, udev->dev.control.ctl_state, 0, 0, 0);
    return USBD_OK;
}

uint8_t usbd_setup_transc(usb_core_driver* udev) {
    usb_reqsta reqstat = REQ_NOTSUPP;
    udev->dev.control.ctl_zlp = 0U;

    // Arrival of a Setup packet automatically clears any previous protocol STALL (USB 2.0 §8.5.3.4)
    usbd_ep_stall_clear(udev, 0x80U);
    usbd_ep_stall_clear(udev, 0x00U);

    usb_req req = udev->dev.control.req;

    switch (req.bmRequestType & USB_REQTYPE_MASK) {
        case USB_REQTYPE_STRD:
            reqstat = usbd_standard_request(udev, &req);
            break;
        case USB_REQTYPE_CLASS:
            reqstat = usbd_class_request(udev, &req);
            break;
        case USB_REQTYPE_VENDOR:
            reqstat = usbd_vendor_request(udev, &req);
            break;
        default:
            break;
    }

    if (REQ_SUPP == reqstat) {
        if (0U == req.wLength) {
            usbd_ctl_status_send(udev);
        } else {
            if (req.bmRequestType & 0x80U) {
                usbd_ctl_send(udev);
            } else {
                usbd_ctl_recev(udev);
            }
        }
    } else {
        usbd_enum_error(udev, &req);
    }

    uint8_t req_bytes[4] = {
        req.bmRequestType,
        req.bRequest,
        static_cast<uint8_t>(req.wIndex & 0xFF),
        static_cast<uint8_t>(req.wIndex >> 8)
    };
    usb_trace_record(0, 0, udev->dev.control.ctl_state, static_cast<uint8_t>(reqstat), req.wValue, req.wLength, req_bytes);

    return USBD_OK;
}

uint8_t usbd_out_transc(usb_core_driver* udev, uint8_t ep_num) {
    if (USBD_SUSPENDED == udev->dev.cur_status) {
        udev->dev.cur_status = udev->dev.backup_status;
    }

    if (0U == ep_num) {
        usb_trace_record(3, 0, udev->dev.control.ctl_state, 0, 0, 0);
        usb_transc* transc = &udev->dev.transc_out[0];

        switch (udev->dev.control.ctl_state) {
            case USB_CTL_DATA_OUT:
                transc->remain_len -= transc->max_len;
                usbd_ctl_recev(udev);
                break;

            case USB_CTL_LAST_DATA_OUT:
                if (USBD_CONFIGURED == udev->dev.cur_status && udev->dev.class_core && udev->dev.class_core->ctlx_out) {
                    udev->dev.class_core->ctlx_out(udev);
                }
                transc->remain_len = 0U;
                usbd_ctl_status_send(udev);
                break;

            case USB_CTL_STATUS_OUT:
                udev->dev.control.ctl_state = USB_CTL_IDLE;
                usb_ctlep_startout(udev);
                break;

            default:
                break;
        }
    } else if (udev->dev.class_core && udev->dev.class_core->data_out && (USBD_CONFIGURED == udev->dev.cur_status)) {
        udev->dev.class_core->data_out(udev, ep_num);
    }

    return USBD_OK;
}

uint8_t usbd_in_transc(usb_core_driver* udev, uint8_t ep_num) {
    if (USBD_SUSPENDED == udev->dev.cur_status) {
        udev->dev.cur_status = udev->dev.backup_status;
    }

    if (0U == ep_num) {
        usb_transc* transc = &udev->dev.transc_in[0];
        usb_trace_record(1, 0, udev->dev.control.ctl_state, 0, static_cast<uint16_t>(transc->remain_len), static_cast<uint16_t>(transc->xfer_len));

        switch (udev->dev.control.ctl_state) {
            case USB_CTL_DATA_IN:
                transc->remain_len -= transc->max_len;
                usbd_ctl_send(udev);
                break;

            case USB_CTL_LAST_DATA_IN:
                if (udev->dev.control.ctl_zlp) {
                    usbd_ep_send(udev, 0U, nullptr, 0U);
                    udev->dev.control.ctl_zlp = 0U;
                } else {
                    if (USBD_CONFIGURED == udev->dev.cur_status && udev->dev.class_core && udev->dev.class_core->ctlx_in) {
                        udev->dev.class_core->ctlx_in(udev);
                    }
                    transc->remain_len = 0U;
                    usbd_ctl_status_recev(udev);
                }
                break;

            case USB_CTL_STATUS_IN:
                udev->dev.control.ctl_state = USB_CTL_IDLE;
                usb_ctlep_startout(udev);
                break;

            default:
                break;
        }
    } else {
        usb_transc* transc = &udev->dev.transc_in[ep_num];
        if (transc->xfer_count < transc->xfer_len) {
            return USBD_OK;
        }
        if (USBD_CONFIGURED == udev->dev.cur_status && udev->dev.class_core && udev->dev.class_core->data_in) {
            udev->dev.class_core->data_in(udev, ep_num);
        }
    }

    return USBD_OK;
}
