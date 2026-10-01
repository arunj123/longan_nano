#pragma once

#include <cstdint>
#include "drv_usb_core.h"

/**
 * @file drv_usb_dev.h
 * @brief USB device-mode endpoint and flow control declarations.
 */

inline constexpr uint32_t USBFS_TX_FIFO_SIZE[4] = {64U, 64U, 32U, 32U};

usb_status usb_devint_enable(usb_core_driver* udev);
usb_status usb_devcore_init(usb_core_driver* udev);
void usb_dev_connect(usb_core_driver* udev);
void usb_dev_disconnect(usb_core_driver* udev);
void usb_devaddr_set(usb_core_driver* udev, uint8_t dev_addr);
usb_status usb_transc0_active(usb_core_driver* udev, usb_transc* transc);
usb_status usb_transc_active(usb_core_driver* udev, usb_transc* transc);
usb_status usb_transc_deactive(usb_core_driver* udev, usb_transc* transc);
usb_status usb_transc_inxfer(usb_core_driver* udev, usb_transc* transc);
usb_status usb_transc_outxfer(usb_core_driver* udev, usb_transc* transc);
usb_status usb_transc_stall(usb_core_driver* udev, usb_transc* transc);
usb_status usb_transc_clrstall(usb_core_driver* udev, usb_transc* transc);
void usbd_ep_nak_arm(usb_core_driver* udev, uint8_t ep_addr, uint32_t len);
void usb_ctlep_startout(usb_core_driver* udev);
