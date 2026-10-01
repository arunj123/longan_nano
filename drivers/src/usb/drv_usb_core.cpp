#include "drv_usb_core.h"
#include "hal/time.hpp"
#include <cstring>

/**
 * @file drv_usb_core.cpp
 * @brief Native modern C++23 Synopsys DWC2 USB core driver for GD32VF103 (Longan Nano).
 */

static void usb_core_reset(usb_core_regs* usb_regs) {
    usb_regs->gr->GRSTCTL |= drivers::usb::dwc2::GRSTCTL_CSRST;
    uint32_t timeout = 100000U;
    while ((usb_regs->gr->GRSTCTL & drivers::usb::dwc2::GRSTCTL_CSRST) && --timeout) {}
    hal::time::delay_us(3);
}

usb_status usb_basic_init(usb_core_basic* usb_basic, [[maybe_unused]] usb_core_regs* usb_regs) {
    usb_basic->transfer_mode = USB_USE_FIFO;
    usb_basic->core_speed = USB_SPEED_FULL;
    usb_basic->base_reg = drivers::usb::dwc2::USBFS_BASE;
    usb_basic->num_pipe = 8U;
    usb_basic->num_ep = 4U; // GD32VF103 has 4 endpoints
    usb_basic->phy_itf = USB_EMBEDDED_PHY;
    usb_basic->sof_enable = 0U;
    usb_basic->low_power = 0U;
    return USB_OK;
}

usb_status usb_core_init([[maybe_unused]] usb_core_basic usb_basic, usb_core_regs* usb_regs) {
    // Select embedded Full-Speed PHY
    usb_regs->gr->GUSBCS |= drivers::usb::dwc2::GUSBCS_EMBPHY;

    // Reset core
    usb_core_reset(usb_regs);

    // Activate transceiver and power on VBUS comparators (PA9 VBUS is not routed, so ignore VBUS)
    usb_regs->gr->GCCFG |= drivers::usb::dwc2::GCCFG_PWRON |
                           drivers::usb::dwc2::GCCFG_VBUSACEN |
                           drivers::usb::dwc2::GCCFG_VBUSBCEN |
                           drivers::usb::dwc2::GCCFG_SOFOEN |
                           drivers::usb::dwc2::GCCFG_VBUSIG;

    // PHY stabilization delay
    hal::time::delay_ms(20);

    return USB_OK;
}

usb_status usb_txfifo_write([[maybe_unused]] usb_core_regs* usb_regs, const uint8_t* src_buf, uint8_t fifo_num, uint16_t byte_count) {
    uint32_t word_count = (static_cast<uint32_t>(byte_count) + 3U) / 4U;
    volatile uint32_t* fifo = drivers::usb::dwc2::fifo_address(fifo_num);

    while (word_count-- > 0U) {
        uint32_t val;
        std::memcpy(&val, src_buf, 4);
        *fifo = val;
        src_buf += 4;
    }

    return USB_OK;
}

void* usb_rxfifo_read([[maybe_unused]] usb_core_regs* usb_regs, uint8_t* dest_buf, uint16_t byte_count) {
    uint32_t word_count = (static_cast<uint32_t>(byte_count) + 3U) / 4U;
    volatile uint32_t* fifo = drivers::usb::dwc2::fifo_address(0);

    while (word_count-- > 0U) {
        uint32_t val = *fifo;
        std::memcpy(dest_buf, &val, 4);
        dest_buf += 4;
    }

    return dest_buf;
}

usb_status usb_txfifo_flush(usb_core_regs* usb_regs, uint8_t fifo_num) {
    usb_regs->gr->GRSTCTL = (static_cast<uint32_t>(fifo_num) << 6) | drivers::usb::dwc2::GRSTCTL_TXFF;
    uint32_t timeout = 100000U;
    while ((usb_regs->gr->GRSTCTL & drivers::usb::dwc2::GRSTCTL_TXFF) && --timeout) {}
    hal::time::delay_us(3);
    return USB_OK;
}

usb_status usb_rxfifo_flush(usb_core_regs* usb_regs) {
    usb_regs->gr->GRSTCTL = drivers::usb::dwc2::GRSTCTL_RXFF;
    uint32_t timeout = 100000U;
    while ((usb_regs->gr->GRSTCTL & drivers::usb::dwc2::GRSTCTL_RXFF) && --timeout) {}
    hal::time::delay_us(3);
    return USB_OK;
}

void usb_set_txfifo(usb_core_regs* usb_regs, uint8_t fifo, uint16_t size) {
    if (0U == size) {
        if (fifo > 0U) {
            usb_regs->gr->DIEPTFLEN[fifo - 1U] = 0U;
        }
        return;
    }

    uint32_t tx_offset = usb_regs->gr->GRFLEN;

    if (0U == fifo) {
        usb_regs->gr->DIEP0TFLEN_HNPTFLEN = (static_cast<uint32_t>(size) << 16) | tx_offset;
    } else {
        tx_offset += (usb_regs->gr->DIEP0TFLEN_HNPTFLEN) >> 16;
        for (uint8_t i = 0U; i < (fifo - 1U); ++i) {
            tx_offset += (usb_regs->gr->DIEPTFLEN[i] >> 16);
        }
        usb_regs->gr->DIEPTFLEN[fifo - 1U] = (static_cast<uint32_t>(size) << 16) | tx_offset;
    }
}

void usb_curmode_set(usb_core_regs* usb_regs, uint8_t mode) {
    usb_regs->gr->GUSBCS &= ~(drivers::usb::dwc2::GUSBCS_FDM | drivers::usb::dwc2::GUSBCS_FHM);
    if (DEVICE_MODE == mode) {
        usb_regs->gr->GUSBCS |= drivers::usb::dwc2::GUSBCS_FDM;
    } else if (HOST_MODE == mode) {
        usb_regs->gr->GUSBCS |= drivers::usb::dwc2::GUSBCS_FHM;
    }
}
