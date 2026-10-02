#include "drv_usb_dev.h"
#include "drv_usbd_int.h"
#include "hal/csr.hpp"
#include "hal/time.hpp"
#include <algorithm>

/**
 * @file drv_usb_dev.cpp
 * @brief Native modern C++23 DWC2 endpoint transaction and flow control engine.
 */

usb_status usb_devint_enable(usb_core_driver* udev) {
    udev->regs.gr->GOTGINTF = 0xFFFFFFFFU;
    udev->regs.gr->GINTF = 0xBFFFFFFFU;
    udev->regs.gr->GINTEN = GINTEN_WKUPIE | GINTEN_SPIE | GINTEN_RXFNEIE |
                            GINTEN_RSTIE | GINTEN_ENUMFIE | GINTEN_IEPIE |
                            GINTEN_OEPIE | GINTEN_SOFIE | GINTEN_ISOONCIE | GINTEN_ISOINCIE;
    return USB_OK;
}

usb_status usb_devcore_init(usb_core_driver* udev) {
    // Restart PHY clock
    volatile uint32_t* pwrclkctl = reinterpret_cast<volatile uint32_t*>(drivers::usb::dwc2::USBFS_BASE + 0x0E00UL);
    *pwrclkctl = 0U;

    // Full speed device
    udev->regs.dr->DCFG &= ~(DCFG_DS | DCFG_EOPFT);
    udev->regs.dr->DCFG |= USB_SPEED_INP_FULL | FRAME_INTERVAL_80;

    // Allocate FIFOs using project configuration
    usb_set_rxfifo(&udev->regs, drivers::usb::RX_FIFO_FS_SIZE);
    usb_set_txfifo(&udev->regs, 0U, drivers::usb::TX0_FIFO_FS_SIZE);
    usb_set_txfifo(&udev->regs, 1U, drivers::usb::TX1_FIFO_FS_SIZE);
    usb_set_txfifo(&udev->regs, 2U, drivers::usb::TX2_FIFO_FS_SIZE);
    usb_set_txfifo(&udev->regs, 3U, drivers::usb::TX3_FIFO_FS_SIZE);

    // Inactive endpoints zeroed (Mandatory Invariant 3)
    for (uint8_t i = 0U; i < 4U; ++i) {
        udev->regs.er_in[i]->DIEPCTL = 0U;
        udev->regs.er_out[i]->DOEPCTL = 0U;
    }

    // Flush FIFOs
    usb_txfifo_flush(&udev->regs, 0x10U);
    usb_rxfifo_flush(&udev->regs);

    // Clear and mask interrupts
    udev->regs.dr->DIEPINTEN = 0U;
    udev->regs.dr->DOEPINTEN = 0U;
    udev->regs.dr->DAEPINT = 0xFFFFFFFFU;
    udev->regs.dr->DAEPINTEN = 0U;

    // Enable device interrupts
    return usb_devint_enable(udev);
}

void usb_dev_connect(usb_core_driver* udev) {
    udev->regs.dr->DCTL &= ~DCTL_SDIS;
    hal::time::delay_ms(3);
}

void usb_dev_disconnect(usb_core_driver* udev) {
    udev->regs.dr->DCTL |= DCTL_SDIS;
    hal::time::delay_ms(1000);
}

void usb_devaddr_set(usb_core_driver* udev, uint8_t dev_addr) {
    udev->regs.dr->DCFG = (udev->regs.dr->DCFG & ~DCFG_DAR) | (static_cast<uint32_t>(dev_addr) << 4);
}

usb_status usb_transc0_active(usb_core_driver* udev, usb_transc* transc) {
    uint8_t ep_num = transc->ep_addr.num;
    if (ep_num != 0U) {
        return USB_FAIL;
    }

    volatile uint32_t* reg_addr = transc->ep_addr.dir ?
        &udev->regs.er_in[0]->DIEPCTL : &udev->regs.er_out[0]->DOEPCTL;

    *reg_addr &= ~(DEPCTL_MPL | DEPCTL_EPTYPE | DEPCTL_TXFNUM);
    *reg_addr |= EP0MPL_64 | (static_cast<uint32_t>(transc->ep_type) << 18) |
                 (static_cast<uint32_t>(ep_num) << 22) | DEPCTL_SD0PID | DEPCTL_EPACT;

    return USB_OK;
}

usb_status usb_transc_active(usb_core_driver* udev, usb_transc* transc) {
    uint8_t ep_num = transc->ep_addr.num;
    if (ep_num >= 4U) {
        return USB_FAIL;
    }

    if (transc->ep_addr.dir) {
        volatile uint32_t* reg = &udev->regs.er_in[ep_num]->DIEPCTL;
        *reg &= ~(DEPCTL_MPL | DEPCTL_EPTYPE | DEPCTL_TXFNUM);
        *reg |= transc->max_len | (static_cast<uint32_t>(transc->ep_type) << 18) |
                (static_cast<uint32_t>(ep_num) << 22) | DEPCTL_SD0PID | DEPCTL_EPACT;
        udev->regs.dr->DAEPINTEN |= (1U << ep_num);
    } else {
        volatile uint32_t* reg = &udev->regs.er_out[ep_num]->DOEPCTL;
        *reg &= ~(DEPCTL_MPL | DEPCTL_EPTYPE);
        *reg |= transc->max_len | (static_cast<uint32_t>(transc->ep_type) << 18) |
                DEPCTL_SD0PID | DEPCTL_EPACT;
        udev->regs.dr->DAEPINTEN |= (1U << (ep_num + 16U));
    }

    return USB_OK;
}

usb_status usb_transc_deactive(usb_core_driver* udev, usb_transc* transc) {
    uint8_t ep_num = transc->ep_addr.num;
    if (ep_num >= 4U) {
        return USB_FAIL;
    }

    if (transc->ep_addr.dir) {
        udev->regs.dr->DAEPINTEN &= ~(1U << ep_num);
        udev->regs.er_in[ep_num]->DIEPCTL &= ~DEPCTL_EPACT;
    } else {
        udev->regs.dr->DAEPINTEN &= ~(1U << (ep_num + 16U));
        udev->regs.er_out[ep_num]->DOEPCTL &= ~DEPCTL_EPACT;
    }

    return USB_OK;
}

usb_status usb_transc_inxfer(usb_core_driver* udev, usb_transc* transc) {
    uint8_t ep_num = transc->ep_addr.num;
    if (ep_num >= 4U) {
        return USB_FAIL;
    }

    uint32_t eplen = 0U;
    if (0U == transc->xfer_len) {
        eplen |= 1U << 19; // 1 packet of 0 bytes
    } else {
        if (0U == ep_num) {
            transc->xfer_len = std::min<uint32_t>(transc->xfer_len, transc->max_len);
            eplen |= 1U << 19;
        } else {
            eplen |= (((transc->xfer_len - 1U) + transc->max_len) / transc->max_len) << 19;
        }
        eplen |= transc->xfer_len;
    }

    // Wait until TX FIFO has enough space if needed
    if (ep_num > 0U && udev->bp.transfer_mode == USB_USE_FIFO) {
        uint32_t needed_words = (transc->xfer_len + 3U) / 4U;
        uint32_t max_fifo = USBFS_TX_FIFO_SIZE[ep_num];
        if (needed_words > max_fifo) needed_words = max_fifo;
        auto deadline = hal::time::Instant::now() + hal::time::Duration::from_ms(10);
        while (((udev->regs.er_in[ep_num]->DIEPTFSTAT & 0xFFFFU) < needed_words) && (hal::time::Instant::now() < deadline)) {}
        if ((udev->regs.er_in[ep_num]->DIEPTFSTAT & 0xFFFFU) < needed_words) {
            return USB_FAIL;
        }
    }

    uint32_t prev_mstatus = hal::core::disable_interrupts();

    bool was_prearmed = (transc->is_prearmed != 0U);
    transc->is_prearmed = 0U;

    uint32_t epctl = udev->regs.er_in[ep_num]->DIEPCTL;

    if (0U == ep_num) {
        // Control EP0: DWC2 EP0 slave mode requires EPEN before writing FIFO
        udev->regs.er_in[0]->DIEPINTF = DIEPINTF_TF | DIEPINTF_EPDIS | DIEPINTF_TXFUD;
        udev->regs.er_in[0]->DIEPLEN = eplen;

        epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_EPD | DEPCTL_SNAK);
        udev->regs.er_in[0]->DIEPCTL = epctl | DEPCTL_EPEN | DEPCTL_CNAK;

        if (udev->bp.transfer_mode == USB_USE_FIFO) {
            usbd_emptytxfifo_write(udev, 0U);
        }
    } else {
        // Bulk / Interrupt IN endpoint: two-phase arming
        if (!was_prearmed) {
            epctl = udev->regs.er_in[ep_num]->DIEPCTL;
            if (!(epctl & DEPCTL_NAKS)) {
                epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_EPD | DEPCTL_CNAK);
                udev->regs.er_in[ep_num]->DIEPCTL = epctl | DEPCTL_SNAK;
                uint32_t timeout = 1000;
                while (!(udev->regs.er_in[ep_num]->DIEPCTL & DEPCTL_NAKS) && --timeout) {}
            }

            udev->regs.er_in[ep_num]->DIEPINTF = DIEPINTF_TF | DIEPINTF_EPDIS | DIEPINTF_TXFUD;
            udev->regs.er_in[ep_num]->DIEPLEN = eplen;

            epctl = udev->regs.er_in[ep_num]->DIEPCTL;
            epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_EPD | DEPCTL_CNAK | DEPCTL_SNAK);
            udev->regs.er_in[ep_num]->DIEPCTL = epctl | DEPCTL_EPEN;
        }

        // Push data to FIFO
        if (udev->bp.transfer_mode == USB_USE_FIFO) {
            if (transc->xfer_len > 0U) {
                usbd_emptytxfifo_write(udev, ep_num);
                if (transc->xfer_count < transc->xfer_len) {
                    udev->regs.dr->DIEPFEINTEN |= (1U << ep_num);
                }
            }
        }

        // Clear NAK to allow packet transmission
        uint32_t cnak_ctl = udev->regs.er_in[ep_num]->DIEPCTL;
        cnak_ctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_EPD | DEPCTL_SNAK);
        cnak_ctl |= DEPCTL_EPEN | DEPCTL_CNAK;
        udev->regs.er_in[ep_num]->DIEPCTL = cnak_ctl;
    }

    hal::core::restore_interrupts(prev_mstatus);
    return USB_OK;
}

usb_status usb_transc_outxfer(usb_core_driver* udev, usb_transc* transc) {
    uint8_t ep_num = transc->ep_addr.num;
    if (ep_num >= 4U) {
        return USB_FAIL;
    }

    uint32_t eplen = 0U;
    if (0U == transc->xfer_len) {
        eplen |= (1U << 19);
    } else {
        uint32_t packet_count = (transc->xfer_len + transc->max_len - 1U) / transc->max_len;
        eplen |= (packet_count << 19) | transc->xfer_len;
    }

    udev->regs.er_out[ep_num]->DOEPLEN = eplen;

    uint32_t epctl = udev->regs.er_out[ep_num]->DOEPCTL;
    epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_EPD | DEPCTL_SNAK);
    udev->regs.er_out[ep_num]->DOEPCTL = epctl | DEPCTL_EPEN | DEPCTL_CNAK;

    return USB_OK;
}

usb_status usb_transc_stall(usb_core_driver* udev, usb_transc* transc) {
    uint8_t ep_num = transc->ep_addr.num;
    if (ep_num >= 4U) {
        return USB_FAIL;
    }

    if (transc->ep_addr.dir) {
        udev->regs.er_in[ep_num]->DIEPCTL |= DEPCTL_STALL;
    } else {
        udev->regs.er_out[ep_num]->DOEPCTL |= DEPCTL_STALL;
    }

    return USB_OK;
}

usb_status usb_transc_clrstall(usb_core_driver* udev, usb_transc* transc) {
    uint8_t ep_num = transc->ep_addr.num;
    if (ep_num >= 4U) {
        return USB_FAIL;
    }

    if (transc->ep_addr.dir) {
        uint32_t epctl = udev->regs.er_in[ep_num]->DIEPCTL;
        epctl &= ~DEPCTL_STALL;
        if (transc->ep_type == USB_EPTYPE_INTR || transc->ep_type == USB_EPTYPE_BULK) {
            epctl |= DEPCTL_SD0PID;
        }
        udev->regs.er_in[ep_num]->DIEPCTL = epctl;
    } else {
        uint32_t epctl = udev->regs.er_out[ep_num]->DOEPCTL;
        epctl &= ~DEPCTL_STALL;
        if (transc->ep_type == USB_EPTYPE_INTR || transc->ep_type == USB_EPTYPE_BULK) {
            epctl |= DEPCTL_SD0PID;
        }
        udev->regs.er_out[ep_num]->DOEPCTL = epctl;
    }

    return USB_OK;
}

void usbd_ep_nak_arm(usb_core_driver* udev, uint8_t ep_addr, uint32_t len) {
    uint8_t ep_num = ep_id(ep_addr);
    if (ep_num >= 4U) return;

    uint32_t prev_mstatus = hal::core::disable_interrupts();

    uint32_t max_len = udev->dev.transc_in[ep_num].max_len;
    if (max_len == 0U) max_len = 64U;
    uint32_t packet_count = (len + max_len - 1U) / max_len;
    if (packet_count == 0U) packet_count = 1U;
    uint32_t target_dieplen = (packet_count << 19) | len;

    uint32_t live_ctl = udev->regs.er_in[ep_num]->DIEPCTL;
    uint32_t live_len = udev->regs.er_in[ep_num]->DIEPLEN;

    // Idempotent check
    if ((live_ctl & DEPCTL_EPEN) && (live_ctl & DEPCTL_NAKS) && (live_len == target_dieplen)) {
        udev->dev.transc_in[ep_num].is_prearmed = 1U;
        hal::core::restore_interrupts(prev_mstatus);
        return;
    }

    // Step 1: Assert SNAK and poll NAKS=1
    uint32_t epctl = live_ctl;
    epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_EPD | DEPCTL_CNAK | DEPCTL_SNAK);
    udev->regs.er_in[ep_num]->DIEPCTL = epctl | DEPCTL_SNAK;

    uint32_t timeout = 1000;
    while (!(udev->regs.er_in[ep_num]->DIEPCTL & DEPCTL_NAKS) && --timeout) {}

    // Step 2: Clear stale flags
    udev->regs.er_in[ep_num]->DIEPINTF = DIEPINTF_TF | DIEPINTF_EPDIS | DIEPINTF_TXFUD;

    // Step 3: Program DIEPLEN
    udev->regs.er_in[ep_num]->DIEPLEN = target_dieplen;

    // Step 4: Assert EPEN with NAK confirmed
    epctl = udev->regs.er_in[ep_num]->DIEPCTL;
    epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_EPD | DEPCTL_CNAK | DEPCTL_SNAK);
    udev->regs.er_in[ep_num]->DIEPCTL = epctl | DEPCTL_EPEN;

    udev->dev.transc_in[ep_num].is_prearmed = 1U;
    hal::core::restore_interrupts(prev_mstatus);
}

void usb_ctlep_startout(usb_core_driver* udev) {
    udev->regs.er_out[0]->DOEPLEN = (24U << 0) | (1U << 19) | (3U << 29);
}
