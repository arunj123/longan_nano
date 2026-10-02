#pragma once

#include <cstdint>
#include <cstddef>
#include <algorithm>
#include "dwc2_regs.hpp"
#include "drv_usb_regs.h"
#include "usb_ch9.hpp"

/**
 * @file drv_usb_core.h
 * @brief Zero-overhead USB Core Driver for GD32VF103 (Longan Nano).
 *
 * Implements compile-time MMIO address calculation with zero pointer indirection overhead
 * and minimized SRAM footprint.
 */

inline constexpr uint8_t  USB_FS_EP0_MAX_LEN      = 64U;
inline constexpr uint32_t HC_MAX_PACKET_COUNT     = 140U;
inline constexpr uint32_t EP_MAX_PACKET_SIZE_MASK = 0x07FFU;

constexpr uint8_t ep_id(uint8_t x) noexcept {
    return static_cast<uint8_t>(x & 0x7FU);
}
constexpr uint8_t ep_dir(uint8_t x) noexcept {
    return static_cast<uint8_t>(x >> 7);
}
constexpr uint8_t ep_in(uint8_t x) noexcept {
    return static_cast<uint8_t>(0x80U | (x & 0x7FU));
}
constexpr uint8_t ep_out(uint8_t x) noexcept {
    return static_cast<uint8_t>(x & 0x7FU);
}

enum _usb_mode {
    DEVICE_MODE = 0U,
    HOST_MODE   = 1U,
    OTG_MODE    = 2U
};

enum _usb_eptype {
    USB_EPTYPE_CTRL = 0U,
    USB_EPTYPE_ISOC = 1U,
    USB_EPTYPE_BULK = 2U,
    USB_EPTYPE_INTR = 3U,
    USB_EPTYPE_MASK = 3U
};

enum usb_status : uint8_t {
    USB_OK = 0U,
    USB_FAIL = 1U
};

enum usb_transfer_mode : uint8_t {
    USB_USE_FIFO = 0U
};

struct usb_core_basic {
    uint8_t  core_speed{2U}; // USB_SPEED_FULL
    uint8_t  num_pipe{8U};
    uint8_t  num_ep{4U};     // GD32VF103 has 4 endpoints (0..3)
    uint8_t  transfer_mode{0U};
    uint8_t  phy_itf{1U};    // Embedded PHY
    uint8_t  sof_enable{0U};
    uint8_t  low_power{0U};
    uint8_t  lpm_enable{0U};
    uint8_t  vbus_sensing_enable{0U};
    uint8_t  use_dedicated_ep1{0U};
    uint8_t  use_external_vbus{0U};
    uint32_t base_reg{drivers::usb::dwc2::USBFS_BASE};
};

// Zero-overhead compile-time MMIO register container (no RAM pointers needed)
struct usb_core_regs {
    [[no_unique_address]] drivers::usb::dwc2::GlobalRegsAccessor gr{};
    [[no_unique_address]] drivers::usb::dwc2::DeviceRegsAccessor dr{};
    [[no_unique_address]] drivers::usb::dwc2::InEndpointsAccessor er_in{};
    [[no_unique_address]] drivers::usb::dwc2::OutEndpointsAccessor er_out{};
    [[no_unique_address]] drivers::usb::dwc2::PwrclkctlAccessor PWRCLKCTL{};
};

struct usb_desc {
    uint8_t* dev_desc{nullptr};
    uint8_t* config_desc{nullptr};
    uint8_t* bos_desc{nullptr};
    void* const* strings{nullptr};
};

struct usb_pm {
    uint8_t power_mode{0};
    uint8_t power_low{0};
    uint8_t dev_remote_wakeup{0};
    uint8_t remote_wakeup_on{0};
};

struct usb_control {
    usb_req req{};
    uint8_t ctl_state{0};
    uint8_t ctl_zlp{0};
};

struct usb_transc {
    struct {
        uint8_t num : 4;
        uint8_t pad : 3;
        uint8_t dir : 1;
    } ep_addr;

    uint8_t   ep_type{0};
    uint16_t  max_len{64};
    uint32_t  xfer_len{0};
    uint32_t  xfer_count{0};
    uint8_t*  xfer_buf{nullptr};
    uint32_t  remain_len{0};
    uint8_t   is_prearmed{0};
};

struct _usb_core_driver;

struct usb_class_core {
    uint8_t command{0};
    uint8_t alter_set{0};

    uint8_t (*init)(_usb_core_driver* udev, uint8_t config_index){nullptr};
    uint8_t (*deinit)(_usb_core_driver* udev, uint8_t config_index){nullptr};
    uint8_t (*req_proc)(_usb_core_driver* udev, usb_req* req){nullptr};
    uint8_t (*set_intf)(_usb_core_driver* udev, usb_req* req){nullptr};
    uint8_t (*ctlx_in)(_usb_core_driver* udev){nullptr};
    uint8_t (*ctlx_out)(_usb_core_driver* udev){nullptr};
    uint8_t (*data_in)(_usb_core_driver* udev, uint8_t ep_num){nullptr};
    uint8_t (*data_out)(_usb_core_driver* udev, uint8_t ep_num){nullptr};
    uint8_t (*SOF)(_usb_core_driver* udev){nullptr};
    uint8_t (*incomplete_isoc_in)(_usb_core_driver* udev){nullptr};
    uint8_t (*incomplete_isoc_out)(_usb_core_driver* udev){nullptr};
};

struct usb_perp_dev {
    uint8_t         config{0};
    uint8_t         dev_addr{0};
    volatile uint8_t cur_status{0};
    volatile uint8_t backup_status{0};

    usb_transc      transc_in[4]{};   // Strictly 4 physical endpoints on GD32VF103
    usb_transc      transc_out[4]{};  // Strictly 4 physical endpoints on GD32VF103

    usb_pm          pm{};
    usb_control     control{};
    usb_desc*       desc{nullptr};
    usb_class_core* class_core{nullptr};
    void*           class_data[4]{nullptr, nullptr, nullptr, nullptr};
    void*           user_data{nullptr};
    void*           pdata{nullptr};
};

struct _usb_core_driver {
    usb_core_basic  bp{};
    usb_core_regs   regs{};
    usb_perp_dev    dev{};
};

using usb_core_driver = _usb_core_driver;
using usb_dev = _usb_core_driver;

// Inline register access utilities
[[nodiscard]] inline uint32_t usb_coreintr_get(usb_core_regs* usb_regs) noexcept {
    return usb_regs->gr->GINTEN & usb_regs->gr->GINTF;
}

inline void usb_set_rxfifo(usb_core_regs* usb_regs, uint16_t size) noexcept {
    usb_regs->gr->GRFLEN = size;
}

inline void usb_globalint_enable(usb_core_regs* usb_regs) noexcept {
    usb_regs->gr->GAHBCS |= drivers::usb::dwc2::GAHBCS_GINTEN;
}

inline void usb_globalint_disable(usb_core_regs* usb_regs) noexcept {
    usb_regs->gr->GAHBCS &= ~drivers::usb::dwc2::GAHBCS_GINTEN;
}

inline void usb_clock_active(usb_core_driver* udev) noexcept {
    *udev->regs.PWRCLKCTL &= ~(drivers::usb::dwc2::PWRCLKCTL_SHCLK | drivers::usb::dwc2::PWRCLKCTL_SUCLK);
}

// Function declarations
usb_status usb_basic_init(usb_core_basic* usb_basic, usb_core_regs* usb_regs);
usb_status usb_core_init(usb_core_basic usb_basic, usb_core_regs* usb_regs);
usb_status usb_txfifo_write(usb_core_regs* usb_regs, const uint8_t* src_buf, uint8_t fifo_num, uint16_t byte_count);
void* usb_rxfifo_read(usb_core_regs* usb_regs, uint8_t* dest_buf, uint16_t byte_count);
usb_status usb_txfifo_flush(usb_core_regs* usb_regs, uint8_t fifo_num);
usb_status usb_rxfifo_flush(usb_core_regs* usb_regs);
void usb_set_txfifo(usb_core_regs* usb_regs, uint8_t fifo, uint16_t size);
void usb_curmode_set(usb_core_regs* usb_regs, uint8_t mode);
