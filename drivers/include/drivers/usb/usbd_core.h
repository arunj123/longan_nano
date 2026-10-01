#pragma once

#include <cstdint>
#include "drv_usb_core.h"
#include "drv_usb_dev.h"

/**
 * @file usbd_core.h
 * @brief High-level USB device core interface.
 */

enum usbd_status : uint8_t {
    USBD_OK   = 0U,
    USBD_BUSY = 1U,
    USBD_FAIL = 2U
};

enum _usbd_status {
    USBD_DEFAULT    = 1U,
    USBD_ADDRESSED  = 2U,
    USBD_CONFIGURED = 3U,
    USBD_SUSPENDED  = 4U
};

inline void usbd_addr_set(usb_core_driver* udev, uint8_t addr) noexcept {
    usb_devaddr_set(udev, addr);
}

inline uint16_t usbd_rxcount_get(const usb_core_driver* udev, uint8_t ep_num) noexcept {
    return static_cast<uint16_t>(udev->dev.transc_out[ep_num].xfer_count);
}

void usbd_init(usb_core_driver* udev, usb_desc* desc, usb_class_core* class_core);
uint32_t usbd_ep_setup(usb_core_driver* udev, const usb_desc_ep* ep_desc);
uint32_t usbd_ep_clear(usb_core_driver* udev, uint8_t ep_addr);
uint32_t usbd_ep_recev(usb_core_driver* udev, uint8_t ep_addr, uint8_t* pbuf, uint32_t len);
uint32_t usbd_ep_send(usb_core_driver* udev, uint8_t ep_addr, uint8_t* pbuf, uint32_t len);
uint32_t usbd_ep_stall(usb_core_driver* udev, uint8_t ep_addr);
uint32_t usbd_ep_stall_clear(usb_core_driver* udev, uint8_t ep_addr);
uint32_t usbd_fifo_flush(usb_core_driver* udev, uint8_t ep_addr);
void usbd_connect(usb_core_driver* udev);
void usbd_disconnect(usb_core_driver* udev);
