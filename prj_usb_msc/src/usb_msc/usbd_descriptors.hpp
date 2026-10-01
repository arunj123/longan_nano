#ifndef USBD_DESCRIPTORS_HPP
#define USBD_DESCRIPTORS_HPP

#include <cstdint>
#include "drivers/usb/usb_core.hpp"
#include "drivers/usb/usbd_core.h"
#include "usbd_conf.h"

#define USBD_VID                     0x28E9
#define USBD_PID                     0xABA0 // Fresh PID for Build 00A0 (LNMSC00000A0)

#define MSC_CONFIG_DESC_SIZE         32U

#pragma pack(push, 1)
struct UsbMscConfigDescSet {
    usb_desc_config config;
    usb_desc_itf    msc_itf;
    usb_desc_ep     msc_epout; // Bulk-OUT (0x01)
    usb_desc_ep     msc_epin;  // Bulk-IN (0x81)
};
#pragma pack(pop)

extern usb_desc_dev msc_dev_desc;
extern UsbMscConfigDescSet msc_config_desc;
extern void *const usbd_msc_strings[];

void set_custom_serial_string(const char *ascii_str);

#endif /* USBD_DESCRIPTORS_HPP */
