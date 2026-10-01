#pragma once

#include <cstdint>
#include "usbd_core.h"

/**
 * @file usbd_transc.h
 * @brief USB control and bulk transaction stage sequencer.
 */

enum _usb_ctl_status {
    USB_CTL_IDLE          = 0U,
    USB_CTL_DATA_IN       = 1U,
    USB_CTL_LAST_DATA_IN  = 2U,
    USB_CTL_DATA_OUT      = 3U,
    USB_CTL_LAST_DATA_OUT = 4U,
    USB_CTL_STATUS_IN     = 5U,
    USB_CTL_STATUS_OUT    = 6U
};

usbd_status usbd_ctl_send(usb_core_driver* udev);
usbd_status usbd_ctl_recev(usb_core_driver* udev);
usbd_status usbd_ctl_status_send(usb_core_driver* udev);
usbd_status usbd_ctl_status_recev(usb_core_driver* udev);
uint8_t usbd_setup_transc(usb_core_driver* udev);
uint8_t usbd_out_transc(usb_core_driver* udev, uint8_t ep_num);
uint8_t usbd_in_transc(usb_core_driver* udev, uint8_t ep_num);

struct UsbTraceEntry {
    uint8_t  type; // 0=SETUP, 1=IN_TF, 2=TXFE, 3=OUT_TF, 4=STATUS_RECV
    uint8_t  ep_num;
    uint8_t  ctl_state;
    uint8_t  status;
    uint16_t val1;
    uint16_t val2;
    uint8_t  extra[4];
};

#define USB_TRACE_MAX 64
extern UsbTraceEntry g_usb_trace[USB_TRACE_MAX];
extern volatile uint8_t g_usb_trace_head;
extern volatile uint8_t g_usb_trace_tail;

void usb_trace_record(uint8_t type, uint8_t ep_num, uint8_t ctl_state, uint8_t status, uint16_t val1, uint16_t val2, const uint8_t* extra = nullptr);
