#include "usbd_core.h"
#include "usbd_enum.h"
#include <cstring>

/**
 * @file usbd_core.cpp
 * @brief Native modern C++23 USB device core lifecycle and endpoint controller.
 */

static constexpr uint32_t ep_type[] = {
    USB_EPTYPE_CTRL,
    USB_EPTYPE_ISOC,
    USB_EPTYPE_BULK,
    USB_EPTYPE_INTR
};

void usbd_init(usb_core_driver* udev, usb_desc* desc, usb_class_core* class_core) {
    udev->dev.desc = desc;
    udev->dev.class_core = class_core;

    serial_string_get(reinterpret_cast<uint16_t*>(udev->dev.desc->strings[STR_IDX_SERIAL]));

    usb_basic_init(&udev->bp, &udev->regs);
    usb_globalint_disable(&udev->regs);

    usb_core_init(udev->bp, &udev->regs);
    usbd_disconnect(udev);

    usb_curmode_set(&udev->regs, DEVICE_MODE);
    usb_devcore_init(udev);

    usb_globalint_enable(&udev->regs);
    usbd_connect(udev);

    udev->dev.cur_status = USBD_DEFAULT;
}

uint32_t usbd_ep_setup(usb_core_driver* udev, const usb_desc_ep* ep_desc) {
    uint8_t ep_addr = ep_desc->bEndpointAddress;
    uint8_t ep_num = EP_ID(ep_addr);
    if (ep_num >= 4U) return 1U;

    usb_transc* transc = EP_DIR(ep_addr) ? &udev->dev.transc_in[ep_num] : &udev->dev.transc_out[ep_num];

    transc->ep_addr.dir = (EP_DIR(ep_addr) != 0U);
    transc->ep_addr.num = static_cast<uint8_t>(ep_num & 0x0FU);
    transc->max_len = ep_desc->wMaxPacketSize & EP_MAX_PACKET_SIZE_MASK;
    transc->ep_type = static_cast<uint8_t>(ep_type[ep_desc->bmAttributes & USB_EPTYPE_MASK]);

    usb_transc_active(udev, transc);
    return 0U;
}

uint32_t usbd_ep_clear(usb_core_driver* udev, uint8_t ep_addr) {
    uint8_t ep_num = EP_ID(ep_addr);
    if (ep_num >= 4U) return 1U;

    usb_transc* transc = EP_DIR(ep_addr) ? &udev->dev.transc_in[ep_num] : &udev->dev.transc_out[ep_num];
    usb_transc_deactive(udev, transc);
    return 0U;
}

uint32_t usbd_ep_recev(usb_core_driver* udev, uint8_t ep_addr, uint8_t* pbuf, uint32_t len) {
    uint8_t ep_num = EP_ID(ep_addr);
    if (ep_num >= 4U) return 1U;

    usb_transc* transc = &udev->dev.transc_out[ep_num];
    transc->xfer_buf = pbuf;
    transc->xfer_len = len;
    transc->xfer_count = 0U;

    usb_transc_outxfer(udev, transc);
    return 0U;
}

uint32_t usbd_ep_send(usb_core_driver* udev, uint8_t ep_addr, uint8_t* pbuf, uint32_t len) {
    uint8_t ep_num = EP_ID(ep_addr);
    if (ep_num >= 4U) return USBD_FAIL;

    usb_transc* transc = &udev->dev.transc_in[ep_num];
    transc->xfer_buf = pbuf;
    transc->xfer_len = len;
    transc->xfer_count = 0U;

    usb_transc_inxfer(udev, transc);
    return USBD_OK;
}

uint32_t usbd_ep_stall(usb_core_driver* udev, uint8_t ep_addr) {
    uint8_t ep_num = EP_ID(ep_addr);
    if (ep_num >= 4U) return 1U;

    usb_transc* transc = EP_DIR(ep_addr) ? &udev->dev.transc_in[ep_num] : &udev->dev.transc_out[ep_num];
    usb_transc_stall(udev, transc);
    return 0U;
}

uint32_t usbd_ep_stall_clear(usb_core_driver* udev, uint8_t ep_addr) {
    uint8_t ep_num = EP_ID(ep_addr);
    if (ep_num >= 4U) return 1U;

    usb_transc* transc = EP_DIR(ep_addr) ? &udev->dev.transc_in[ep_num] : &udev->dev.transc_out[ep_num];
    usb_transc_clrstall(udev, transc);
    return 0U;
}

uint32_t usbd_fifo_flush(usb_core_driver* udev, uint8_t ep_addr) {
    if (EP_DIR(ep_addr)) {
        usb_txfifo_flush(&udev->regs, EP_ID(ep_addr));
    } else {
        usb_rxfifo_flush(&udev->regs);
    }
    return 0U;
}

void usbd_connect(usb_core_driver* udev) {
    usb_dev_connect(udev);
}

void usbd_disconnect(usb_core_driver* udev) {
    usb_dev_disconnect(udev);
}
