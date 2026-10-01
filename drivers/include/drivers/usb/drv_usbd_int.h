#pragma once

#include <cstdint>
#include "drv_usb_core.h"
#include "drv_usb_dev.h"

/**
 * @file drv_usbd_int.h
 * @brief USB Device interrupt router and FIFO feeder.
 */

void usbd_isr(usb_core_driver* udev);
uint32_t usbd_emptytxfifo_write(usb_core_driver* udev, uint32_t ep_num);

// Zero-overhead debug trace stub (saves 8192 bytes of SRAM)
inline void ep1_debug_record(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t) noexcept {}
