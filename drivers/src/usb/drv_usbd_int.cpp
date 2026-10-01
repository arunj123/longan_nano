#include "drv_usbd_int.h"
#include "usbd_transc.h"
#include <algorithm>
#include <cstring>

/**
 * @file drv_usbd_int.cpp
 * @brief High-performance native C++23 DWC2 interrupt service routine and FIFO loader.
 */

static uint32_t usbd_int_epout(usb_core_driver* udev);
static uint32_t usbd_int_epin(usb_core_driver* udev);
static uint32_t usbd_int_rxfifo(usb_core_driver* udev);
static uint32_t usbd_int_reset(usb_core_driver* udev);
static uint32_t usbd_int_enumfinish(usb_core_driver* udev);
static uint32_t usbd_int_suspend(usb_core_driver* udev);

void usbd_isr(usb_core_driver* udev) {
    if (HOST_MODE == (udev->regs.gr->GINTF & GINTF_COPM)) {
        return;
    }

    uint32_t intr = udev->regs.gr->GINTF & udev->regs.gr->GINTEN;
    if (!intr) {
        return;
    }

    if (intr & GINTF_OEPIF) {
        usbd_int_epout(udev);
    }

    if (intr & GINTF_IEPIF) {
        usbd_int_epin(udev);
    }

    if (intr & GINTF_SP) {
        usbd_int_suspend(udev);
    }

    if (intr & GINTF_WKUPIF) {
        if (USBD_SUSPENDED == udev->dev.cur_status) {
            udev->dev.cur_status = udev->dev.backup_status;
        }
        udev->regs.gr->GINTF = GINTF_WKUPIF;
    }

    if (intr & GINTF_SOF) {
        if (udev->dev.class_core && udev->dev.class_core->SOF) {
            udev->dev.class_core->SOF(udev);
        }
        udev->regs.gr->GINTF = GINTF_SOF;
    }

    if (intr & GINTF_RXFNEIF) {
        usbd_int_rxfifo(udev);
    }

    if (intr & GINTF_RST) {
        usbd_int_reset(udev);
    }

    if (intr & GINTF_ENUMFIF) {
        usbd_int_enumfinish(udev);
    }
}

static uint32_t usbd_int_epout(usb_core_driver* udev) {
    uint32_t epintr = (udev->regs.dr->DAEPINT >> 16) & udev->regs.dr->DAEPINTEN >> 16;

    for (uint8_t ep_num = 0U; ep_num < 4U; ++ep_num) {
        if (epintr & (1U << ep_num)) {
            uint32_t oepintr = udev->regs.er_out[ep_num]->DOEPINTF & udev->regs.dr->DOEPINTEN;

            if (oepintr & DOEPINTF_TF) {
                udev->regs.er_out[ep_num]->DOEPINTF = DOEPINTF_TF;
                usbd_out_transc(udev, ep_num);
            }

            if (oepintr & DOEPINTF_STPF) {
                udev->regs.er_out[ep_num]->DOEPINTF = DOEPINTF_STPF;
                usbd_setup_transc(udev);
            }

            // Clear any other miscellaneous flags
            uint32_t raw_oep = udev->regs.er_out[ep_num]->DOEPINTF;
            if (raw_oep & ~(DOEPINTF_TF | DOEPINTF_STPF)) {
                udev->regs.er_out[ep_num]->DOEPINTF = raw_oep & ~(DOEPINTF_TF | DOEPINTF_STPF);
            }
        }
    }

    return 1U;
}

static uint32_t usbd_int_epin(usb_core_driver* udev) {
    uint32_t epintr = (udev->regs.dr->DAEPINT & 0x0FU) & (udev->regs.dr->DAEPINTEN & 0x0FU);
    uint32_t diepfeinten = udev->regs.dr->DIEPFEINTEN;

    for (uint8_t ep_num = 0U; ep_num < 4U; ++ep_num) {
        if (epintr & (1U << ep_num)) {
            uint32_t iepintr = udev->regs.er_in[ep_num]->DIEPINTF & udev->regs.dr->DIEPINTEN;

            if (iepintr & DIEPINTF_TF) {
                udev->regs.er_in[ep_num]->DIEPINTF = DIEPINTF_TF;
                if ((udev->regs.er_in[ep_num]->DIEPLEN & DEPLEN_PCNT) == 0U) {
                    usbd_in_transc(udev, ep_num);
                }
            }

            uint32_t raw_iep = udev->regs.er_in[ep_num]->DIEPINTF;
            if (raw_iep & ~(DIEPINTF_TF | DIEPINTF_TXFE)) {
                udev->regs.er_in[ep_num]->DIEPINTF = raw_iep & ~(DIEPINTF_TF | DIEPINTF_TXFE);
            }
        }

        // FIFO empty interrupt
        if (diepfeinten & (1U << ep_num)) {
            if (udev->regs.er_in[ep_num]->DIEPINTF & DIEPINTF_TXFE) {
                usbd_emptytxfifo_write(udev, ep_num);
            }
        }
    }

    return 1U;
}

static uint32_t usbd_int_rxfifo(usb_core_driver* udev) {
    udev->regs.gr->GINTEN &= ~GINTEN_RXFNEIE;
    uint32_t devrxstat = udev->regs.gr->GRSTATP;
    udev->regs.gr->GINTEN |= GINTEN_RXFNEIE;

    uint8_t ep_num = devrxstat & GRSTATRP_EPNUM;
    uint16_t bcount = static_cast<uint16_t>((devrxstat & GRSTATRP_BCOUNT) >> 4);
    uint32_t pktstatus = (devrxstat & GRSTATRP_RPCKST) >> 17;

    if (ep_num >= 4U) return 1U;

    usb_transc* transc = &udev->dev.transc_out[ep_num];

    switch (pktstatus) {
        case RSTAT_SETUP_UPDT:
            usb_rxfifo_read(&udev->regs, reinterpret_cast<uint8_t*>(&udev->dev.control.req), bcount);
            transc->xfer_count += bcount;
            break;

        case RSTAT_DATA_UPDT:
            if (bcount > 0U) {
                if (transc->xfer_buf != nullptr) {
                    usb_rxfifo_read(&udev->regs, transc->xfer_buf, bcount);
                    transc->xfer_buf += bcount;
                } else {
                    uint8_t discard[64];
                    usb_rxfifo_read(&udev->regs, discard, std::min<uint16_t>(bcount, static_cast<uint16_t>(sizeof(discard))));
                }
                transc->xfer_count += bcount;
            }
            break;

        default:
            break;
    }

    return 1U;
}

static uint32_t usbd_int_reset(usb_core_driver* udev) {
    // Clear remote wakeup signaling
    udev->regs.dr->DCTL &= ~DCTL_RWKUP;

    // Flush FIFOs
    usb_txfifo_flush(&udev->regs, 0x10U);
    usb_rxfifo_flush(&udev->regs);

    // Reset endpoints (Invariant: never set SNAK on EP0)
    for (uint8_t i = 0U; i < 4U; ++i) {
        udev->dev.transc_in[i].is_prearmed = 0U;
        if (i > 0U) {
            uint32_t epctl = udev->regs.er_in[i]->DIEPCTL;
            if (epctl & DEPCTL_EPEN) {
                epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID | DEPCTL_CNAK);
                udev->regs.er_in[i]->DIEPCTL = epctl | DEPCTL_EPD | DEPCTL_SNAK;
                uint32_t timeout = 1000;
                while ((udev->regs.er_in[i]->DIEPCTL & DEPCTL_EPEN) && --timeout) {}
            }
            udev->regs.er_in[i]->DIEPCTL = (epctl & ~(DEPCTL_EPEN | DEPCTL_EPD | DEPCTL_CNAK)) | DEPCTL_SD0PID | DEPCTL_SNAK;
        }
        udev->regs.er_in[i]->DIEPLEN = 0U;
        udev->regs.er_in[i]->DIEPINTF = 0xFFU;
        udev->regs.er_out[i]->DOEPLEN = 0U;
        udev->regs.er_out[i]->DOEPINTF = 0xFFU;
    }

    udev->regs.dr->DAEPINT = 0xFFFFFFFFU;
    udev->regs.dr->DAEPINTEN = 1U | (1U << 16);
    udev->regs.dr->DOEPINTEN = DOEPINTEN_STPFEN | DOEPINTEN_TFEN;
    udev->regs.dr->DIEPINTEN = DIEPINTEN_TFEN;
    udev->regs.dr->DCFG &= ~DCFG_DAR;

    // Arm endpoint 0 to receive SETUP packets
    usb_ctlep_startout(udev);

    // Clear reset flag
    udev->regs.gr->GINTF = GINTF_RST;

    // Initialize EP0 transactions
    udev->dev.transc_out[0] = usb_transc{};
    udev->dev.transc_out[0].ep_type = USB_EPTYPE_CTRL;
    udev->dev.transc_out[0].max_len = USB_FS_EP0_MAX_LEN;
    usb_transc0_active(udev, &udev->dev.transc_out[0]);

    udev->dev.transc_in[0] = usb_transc{};
    udev->dev.transc_in[0].ep_addr.dir = 1U;
    udev->dev.transc_in[0].ep_type = USB_EPTYPE_CTRL;
    udev->dev.transc_in[0].max_len = USB_FS_EP0_MAX_LEN;
    usb_transc0_active(udev, &udev->dev.transc_in[0]);

    // Reset status to DEFAULT
    udev->dev.cur_status = USBD_DEFAULT;

    // Notify class init callback
    if (udev->dev.class_core && udev->dev.class_core->init) {
        udev->dev.class_core->init(udev, 0U);
    }

    return 1U;
}

static uint32_t usbd_int_enumfinish(usb_core_driver* udev) {
    udev->regs.dr->DCTL |= DCTL_CGINAK;

    // Set Full-Speed turnaround time (48 MHz clock: UTT = 9)
    udev->regs.gr->GUSBCS = (udev->regs.gr->GUSBCS & ~GUSBCS_UTT) | (0x09U << 10);

    udev->regs.gr->GINTF = GINTF_ENUMFIF;
    return 1U;
}

static uint32_t usbd_int_suspend(usb_core_driver* udev) {
    if (udev->regs.dr->DSTAT & DSTAT_SPST) {
        udev->dev.backup_status = udev->dev.cur_status;
        udev->dev.cur_status = USBD_SUSPENDED;
    }
    udev->regs.gr->GINTF = GINTF_SP;
    return 1U;
}

uint32_t usbd_emptytxfifo_write(usb_core_driver* udev, uint32_t ep_num) {
    if (ep_num >= 4U) return 0U;

    usb_transc* transc = &udev->dev.transc_in[ep_num];

    while (transc->xfer_count < transc->xfer_len) {
        uint32_t len = std::min<uint32_t>(transc->xfer_len - transc->xfer_count, transc->max_len);
        uint32_t word_count = (len + 3U) / 4U;

        uint32_t fifo_space;
        if (0U == ep_num) {
            fifo_space = udev->regs.gr->HNPTFQSTAT & 0xFFFFU;
        } else {
            fifo_space = udev->regs.er_in[ep_num]->DIEPTFSTAT & DIEPTFSTAT_IEPTFS;
        }

        if (fifo_space < word_count) {
            break;
        }

        usb_txfifo_write(&udev->regs, transc->xfer_buf, static_cast<uint8_t>(ep_num), static_cast<uint16_t>(len));

        transc->xfer_buf += len;
        transc->xfer_count += len;

        if (transc->xfer_count == transc->xfer_len) {
            udev->regs.dr->DIEPFEINTEN &= ~(1U << ep_num);
            break;
        }
    }

    return 1U;
}
