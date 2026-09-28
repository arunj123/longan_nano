#include <cstring>
/*!
    \file    drv_usbd_int.c
    \brief   USB device mode interrupt routines

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

/*
    Copyright (c) 2025, GigaDevice Semiconductor Inc.

    Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

    1. Redistributions of source code must retain the above copyright notice, this
       list of conditions and the following disclaimer.
    2. Redistributions in binary form must reproduce the above copyright notice,
       this list of conditions and the following disclaimer in the documentation
       and/or other materials provided with the distribution.
    3. Neither the name of the copyright holder nor the names of its contributors
       may be used to endorse or promote products derived from this software without
       specific prior written permission.

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT,
INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY
OF SUCH DAMAGE.
*/

#include "drv_usbd_int.h"
#include "usbd_transc.h"

static const uint8_t USB_SPEED[4] = {
    (uint8_t)USB_SPEED_HIGH,
    (uint8_t)USB_SPEED_FULL,
    (uint8_t)USB_SPEED_FULL,
    (uint8_t)USB_SPEED_LOW
};

__IO uint8_t setupc_flag = 0U;

struct Ep1DebugEntry {
    uint32_t step;       // 1 = inxfer, 2 = epin_entry, 4 = in_transc_called, 5 = csw_send
    uint32_t val1;       // iepintr or epctl
    uint32_t dieplen;    // DIEPLEN
    uint32_t dieptfstat; // DIEPTFSTAT
    uint32_t xfer_count; // transc->xfer_count
    uint32_t xfer_len;   // transc->xfer_len
    uint32_t epctl;      // DIEPCTL
    uint32_t reserved;
};

#define EP1_DEBUG_MAX 256
__attribute__((used)) Ep1DebugEntry g_ep1_debug[EP1_DEBUG_MAX] = {};
__attribute__((used)) uint32_t g_ep1_debug_idx = 0;
__attribute__((used)) volatile bool g_ep1_freeze = false;

void ep1_debug_record(uint32_t step, uint32_t val1, uint32_t dieplen, uint32_t dieptfstat, uint32_t xfer_count, uint32_t xfer_len, uint32_t epctl)
{
    if (g_ep1_freeze) return;
    uint32_t idx = g_ep1_debug_idx % EP1_DEBUG_MAX;
    uint32_t t = *(volatile const uint32_t*)0xD1000000;
    g_ep1_debug[idx] = {step, val1, dieplen, dieptfstat, xfer_count, xfer_len, epctl, t};
    g_ep1_debug_idx = g_ep1_debug_idx + 1U;
}

/* local function prototypes ('static') */
static uint32_t usbd_int_epout(usb_core_driver *udev);
static uint32_t usbd_int_epin(usb_core_driver *udev);
static uint32_t usbd_int_rxfifo(usb_core_driver *udev);
static uint32_t usbd_int_reset(usb_core_driver *udev);
static uint32_t usbd_int_enumfinish(usb_core_driver *udev);
static uint32_t usbd_int_suspend(usb_core_driver *udev);
uint32_t usbd_emptytxfifo_write(usb_core_driver *udev, uint32_t ep_num);

/*!
    \brief      USB device-mode interrupts global service routine handler
    \param[in]  udev: pointer to USB device instance
    \param[out] none
    \retval     none
*/
void usbd_isr(usb_core_driver *udev)
{
    if(HOST_MODE != (udev->regs.gr->GINTF & GINTF_COPM)) {
        uint32_t intr = udev->regs.gr->GINTF;
        intr &= udev->regs.gr->GINTEN;

        /* there are no interrupts, avoid spurious interrupt */
        if(!intr) {
            return;
        }

        /* OUT endpoints interrupts */
        if(intr & GINTF_OEPIF) {
            (void)usbd_int_epout(udev);
        }

        /* IN endpoints interrupts */
        if(intr & GINTF_IEPIF) {
            (void)usbd_int_epin(udev);
        }

        /* suspend interrupt */
        if(intr & GINTF_SP) {
            (void)usbd_int_suspend(udev);
        }

        /* wakeup interrupt */
        if(intr & GINTF_WKUPIF) {
            if(USBD_SUSPENDED == udev->dev.cur_status) {
                /* inform upper layer by the resume event */
                udev->dev.cur_status = udev->dev.backup_status;
            }

            /* clear interrupt */
            udev->regs.gr->GINTF = GINTF_WKUPIF;
        }

        /* start of frame interrupt */
        if(intr & GINTF_SOF) {
            if(udev->dev.class_core->SOF) {
                (void)udev->dev.class_core->SOF(udev);
            }

            if(0U != setupc_flag) {
                setupc_flag = setupc_flag + 1;

                if(setupc_flag >= 3U) {
                    usbd_setup_transc(udev);

                    setupc_flag = 0U;
                }
            }

            /* clear interrupt */
            udev->regs.gr->GINTF = GINTF_SOF;
        }

        /* receive FIFO not empty interrupt */
        if(intr & GINTF_RXFNEIF) {
            (void)usbd_int_rxfifo(udev);
        }

        /* USB reset interrupt */
        if(intr & GINTF_RST) {
            (void)usbd_int_reset(udev);
        }

        /* enumeration has been done interrupt */
        if(intr & GINTF_ENUMFIF) {
            (void)usbd_int_enumfinish(udev);
        }

        /* incomplete synchronization IN transfer interrupt*/
        if(intr & GINTF_ISOINCIF) {
            if(NULL != udev->dev.class_core->incomplete_isoc_in) {
                (void)udev->dev.class_core->incomplete_isoc_in(udev);
            }

            /* clear interrupt */
            udev->regs.gr->GINTF = GINTF_ISOINCIF;
        }

        /* incomplete synchronization OUT transfer interrupt*/
        if(intr & GINTF_ISOONCIF) {
            if(NULL != udev->dev.class_core->incomplete_isoc_out) {
                (void)udev->dev.class_core->incomplete_isoc_out(udev);
            }

            /* clear interrupt */
            udev->regs.gr->GINTF = GINTF_ISOONCIF;
        }

#ifdef VBUS_SENSING_ENABLED

        /* session request interrupt */
        if(intr & GINTF_SESIF) {
            udev->regs.gr->GINTF = GINTF_SESIF;
        }

        /* OTG mode interrupt */
        if(intr & GINTF_OTGIF) {
            if(udev->regs.gr->GOTGINTF & GOTGINTF_SESEND) {

            }

            /* clear OTG interrupt */
            udev->regs.gr->GINTF = GINTF_OTGIF;
        }
#endif /* VBUS_SENSING_ENABLED */
    }
}

/*!
    \brief      indicates that an OUT endpoint has a pending interrupt
    \param[in]  udev: pointer to USB device instance
    \param[out] none
    \retval     operation status
*/
static uint32_t usbd_int_epout(usb_core_driver *udev)
{
    uint32_t epintnum = 0U;
    uint8_t ep_num = 0U;

    for(epintnum = usb_oepintnum_read(udev); epintnum; epintnum >>= 1, ep_num++) {
        if(epintnum & 0x01U) {
            __IO uint32_t oepintr = usb_oepintr_read(udev, ep_num);

            /* transfer complete interrupt */
            if(oepintr & DOEPINTF_TF) {
                /* clear the bit in DOEPINTF for this interrupt */
                udev->regs.er_out[ep_num]->DOEPINTF = DOEPINTF_TF;

                /* inform upper layer: data ready */
                (void)usbd_out_transc(udev, ep_num);
            }

            /* SETUP phase finished interrupt (control endpoints) */
            if(oepintr & DOEPINTF_STPF) {
                if((0U == ep_num) && (0U != setupc_flag)) {
                    /* inform the upper layer that a SETUP packet is available */
                    (void)usbd_setup_transc(udev);

                    udev->regs.er_out[ep_num]->DOEPINTF = DOEPINTF_STPF;

                    setupc_flag = 0U;
                }
            }

            /* clear unhandled OUT endpoint interrupt flags */
            uint32_t raw_oep = udev->regs.er_out[ep_num]->DOEPINTF;
            if (raw_oep & ~(DOEPINTF_TF | DOEPINTF_STPF)) {
                udev->regs.er_out[ep_num]->DOEPINTF = raw_oep & ~(DOEPINTF_TF | DOEPINTF_STPF);
            }
        }
    }

    return 1U;
}

/*!
    \brief      indicates that an IN endpoint has a pending interrupt
    \param[in]  udev: pointer to USB device instance
    \param[out] none
    \retval     operation status
*/
static uint32_t usbd_int_epin(usb_core_driver *udev)
{
    uint32_t epintnum = 0U;
    uint8_t ep_num = 0U;

    for(epintnum = usb_iepintnum_read(udev); epintnum; epintnum >>= 1, ep_num++) {
        if(epintnum & 0x1U) {
            __IO uint32_t iepintr = usb_iepintr_read(udev, ep_num);

            if(ep_num == 1) {
                ep1_debug_record(2, iepintr, udev->regs.er_in[1]->DIEPLEN, udev->regs.er_in[1]->DIEPTFSTAT, udev->dev.transc_in[1].xfer_count, udev->dev.transc_in[1].xfer_len, udev->regs.er_in[1]->DIEPCTL);
            }

            /* 1. Process TX FIFO empty FIRST to feed remaining data packets */
            if(iepintr & DIEPINTF_TXFE) {
                usbd_emptytxfifo_write(udev, (uint32_t)ep_num);
                // Note: DIEPINTF_TXFE is a read-only status bit in DWC2; do not write 1 to it.
            }

            /* 2. Process Transfer Finished AFTER FIFO processing */
            if(iepintr & DIEPINTF_TF) {
                udev->regs.er_in[ep_num]->DIEPINTF = DIEPINTF_TF;

                if(ep_num == 1) {
                    ep1_debug_record(4, iepintr, udev->regs.er_in[1]->DIEPLEN, udev->regs.er_in[1]->DIEPTFSTAT, udev->dev.transc_in[1].xfer_count, udev->dev.transc_in[1].xfer_len, udev->regs.er_in[1]->DIEPCTL);
                }

                /* Only signal transfer completion if hardware packet counter reached 0 (all packets ACKed by host) */
                if((udev->regs.er_in[ep_num]->DIEPLEN & DEPLEN_PCNT) == 0U) {
                    /* data transmission is completed */
                    (void)usbd_in_transc(udev, ep_num);
                }
            }

            /* Clear unhandled rc_w1 IN endpoint interrupt flags (excluding read-only TXFE) */
            uint32_t raw_iep = udev->regs.er_in[ep_num]->DIEPINTF;
            if(raw_iep & ~(DIEPINTF_TF | DIEPINTF_TXFE)) {
                udev->regs.er_in[ep_num]->DIEPINTF = raw_iep & ~(DIEPINTF_TF | DIEPINTF_TXFE);
            }
        }
    }

    return 1U;
}

/*!
    \brief      handle the RX status queue level interrupt
    \param[in]  udev: pointer to USB device instance
    \param[out] none
    \retval     operation status
*/
static uint32_t usbd_int_rxfifo(usb_core_driver *udev)
{
    usb_transc *transc = NULL;

    uint8_t data_PID = 0U;
    uint32_t bcount = 0U;

    __IO uint32_t devrxstat = 0U;

    /* disable the RX status queue non-empty interrupt */
    udev->regs.gr->GINTEN &= ~GINTEN_RXFNEIE;

    /* get the status from the top of the FIFO */
    devrxstat = udev->regs.gr->GRSTATP;

    uint8_t ep_num = (uint8_t)(devrxstat & GRSTATRP_EPNUM);

    transc = &udev->dev.transc_out[ep_num];

    bcount = (devrxstat & GRSTATRP_BCOUNT) >> 4;
    data_PID = (uint8_t)((devrxstat & GRSTATRP_DPID) >> 15);

    switch((devrxstat & GRSTATRP_RPCKST) >> 17) {
    case RSTAT_GOUT_NAK:
        break;

    case RSTAT_DATA_UPDT:
        if(bcount > 0U) {
            (void)usb_rxfifo_read(&udev->regs, transc->xfer_buf, (uint16_t)bcount);

            transc->xfer_buf += bcount;
            transc->xfer_count += bcount;
        }
        break;

    case RSTAT_XFER_COMP:
        /* trigger the OUT endpoint interrupt */
        break;

    case RSTAT_SETUP_COMP:
        /* trigger the OUT endpoint interrupt */
        break;

    case RSTAT_SETUP_UPDT:
        if((0U == transc->ep_addr.num) && (8U == bcount) && (DPID_DATA0 == data_PID)) {
            /* copy the SETUP packet received in FIFO into the setup buffer in RAM */
            (void)usb_rxfifo_read(&udev->regs, (uint8_t *)&udev->dev.control.req, (uint16_t)bcount);

            transc->xfer_count += bcount;

            /* set the flag */
            setupc_flag = 1;
        }
        break;

    default:
        break;
    }

    /* enable the RX status queue level interrupt */
    udev->regs.gr->GINTEN |= GINTEN_RXFNEIE;

    return 1U;
}

/*!
    \brief      handle USB reset interrupt
    \param[in]  udev: pointer to USB device instance
    \param[out] none
    \retval     status
*/
static uint32_t usbd_int_reset(usb_core_driver *udev)
{
    uint32_t i;

    /* Freeze debug recording on USB reset if activity occurred */
    if (g_ep1_debug_idx > 20) {
        g_ep1_freeze = true;
    }

    /* clear the remote wakeup signaling */
    udev->regs.dr->DCTL &= ~DCTL_RWKUP;

    /* flush all TX FIFOs and RX FIFO */
    (void)usb_txfifo_flush(&udev->regs, 0x10U);
    (void)usb_rxfifo_flush(&udev->regs);

    for(i = 0U; i < udev->bp.num_ep; i++) {
        udev->regs.er_in[i]->DIEPLEN = 0U;
        udev->regs.er_in[i]->DIEPINTF = 0xFFU;
        udev->regs.er_out[i]->DOEPLEN = 0U;
        udev->regs.er_out[i]->DOEPINTF = 0xFFU;
        if(i > 0U) {
            udev->regs.er_in[i]->DIEPCTL = DEPCTL_SD0PID | DEPCTL_SNAK;
            udev->regs.er_out[i]->DOEPCTL = DEPCTL_SD0PID | DEPCTL_SNAK;
        } else {
            udev->regs.er_in[0]->DIEPCTL = DEPCTL_SNAK;
            udev->regs.er_out[0]->DOEPCTL = DEPCTL_SNAK;
        }
    }

    /* clear all pending device endpoint interrupts */
    udev->regs.dr->DAEPINT = 0xFFFFFFFFU;

    /* enable endpoint 0 interrupts */
    udev->regs.dr->DAEPINTEN = 1U | (1U << 16);

    /* enable OUT endpoint interrupts */
    udev->regs.dr->DOEPINTEN = DOEPINTEN_STPFEN | DOEPINTEN_TFEN;

    /* enable IN endpoint interrupts */
    udev->regs.dr->DIEPINTEN = DIEPINTEN_TFEN;

    /* reset device address */
    udev->regs.dr->DCFG &= ~DCFG_DAR;

    /* configure endpoint 0 to receive SETUP packets */
    usb_ctlep_startout(udev);

    /* clear USB reset interrupt */
    udev->regs.gr->GINTF = GINTF_RST;

    std::memset(&udev->dev.transc_out[0], 0, sizeof(usb_transc));
    udev->dev.transc_out[0].ep_type = USB_EPTYPE_CTRL;
    udev->dev.transc_out[0].max_len = USB_FS_EP0_MAX_LEN;
    (void)usb_transc_active(udev, &udev->dev.transc_out[0]);

    std::memset(&udev->dev.transc_in[0], 0, sizeof(usb_transc));
    udev->dev.transc_in[0].ep_addr.dir = 1U;
    udev->dev.transc_in[0].ep_type = USB_EPTYPE_CTRL;
    udev->dev.transc_in[0].max_len = USB_FS_EP0_MAX_LEN;

    (void)usb_transc_active(udev, &udev->dev.transc_in[0]);

    /* upon reset call user call back */
    udev->dev.cur_status = (uint8_t)USBD_DEFAULT;

    return 1U;
}

/*!
    \brief      handle USB speed enumeration finish interrupt
    \param[in]  udev: pointer to USB device instance
    \param[out] none
    \retval     status
*/
static uint32_t usbd_int_enumfinish(usb_core_driver *udev)
{
    uint8_t enum_speed = (uint8_t)((udev->regs.dr->DSTAT & DSTAT_ES) >> 1);

    udev->regs.dr->DCTL &= ~DCTL_CGINAK;
    udev->regs.dr->DCTL |= DCTL_CGINAK;

    udev->regs.gr->GUSBCS &= ~GUSBCS_UTT;

    /* set USB turn-around time based on device speed and PHY interface */
    if((uint8_t)USB_SPEED_HIGH == USB_SPEED[enum_speed]) {
        udev->bp.core_speed = (uint8_t)USB_SPEED_HIGH;

        udev->regs.gr->GUSBCS |= 0x09U << 10;
    } else {
        udev->bp.core_speed = (uint8_t)USB_SPEED_FULL;

        udev->regs.gr->GUSBCS |= 0x05U << 10;
    }

    /* clear interrupt */
    udev->regs.gr->GINTF = GINTF_ENUMFIF;

    return 1U;
}

/*!
    \brief      USB suspend interrupt handler
    \param[in]  udev: pointer to USB device instance
    \param[out] none
    \retval     operation status
*/
static uint32_t usbd_int_suspend(usb_core_driver *udev)
{
    __IO uint8_t low_power = udev->bp.low_power;
    __IO uint8_t suspend = (uint8_t)(udev->regs.dr->DSTAT & DSTAT_SPST);
    __IO uint8_t is_configured = ((uint8_t)USBD_CONFIGURED == udev->dev.cur_status) ? 1U : 0U;

    udev->dev.backup_status = udev->dev.cur_status;
    udev->dev.cur_status = (uint8_t)USBD_SUSPENDED;

    if(low_power && suspend && is_configured) {
        /* switch-off the USB clocks */
        *udev->regs.PWRCLKCTL |= PWRCLKCTL_SUCLK | PWRCLKCTL_SHCLK;

        /* enter DEEP_SLEEP mode with LDO in low power mode */
        // deepsleep mode
    }

    /* clear interrupt */
    udev->regs.gr->GINTF = GINTF_SP;

    return 1U;
}

/*!
    \brief      check FIFO for the next packet to be loaded
    \param[in]  udev: pointer to USB device instance
    \param[in]  ep_num: endpoint identifier which is in (0..3)
    \param[out] none
    \retval     status
*/
uint32_t usbd_emptytxfifo_write(usb_core_driver *udev, uint32_t ep_num)
{
    uint32_t len;
    uint32_t word_count;

    usb_transc *transc = &udev->dev.transc_in[ep_num];

    while(transc->xfer_count < transc->xfer_len) {
        len = transc->xfer_len - transc->xfer_count;

        if(len > transc->max_len) {
            len = transc->max_len;
        }

        /* write FIFO in word(4bytes) */
        word_count = (len + 3U) / 4U;

        uint32_t fifo_space;
        if(0U == ep_num) {
            fifo_space = udev->regs.gr->HNPTFQSTAT & 0xFFFFU;
        } else {
            fifo_space = udev->regs.er_in[ep_num]->DIEPTFSTAT & DIEPTFSTAT_IEPTFS;
        }

        if(fifo_space < word_count) {
            if (ep_num == 1) {
                ep1_debug_record(7, word_count, udev->regs.er_in[1]->DIEPLEN, udev->regs.er_in[1]->DIEPTFSTAT, transc->xfer_count, transc->xfer_len, udev->regs.er_in[1]->DIEPCTL);
            }
            break;
        }

        /* write the FIFO */
        (void)usb_txfifo_write(&udev->regs, transc->xfer_buf, (uint8_t)ep_num, (uint16_t)len);

        transc->xfer_buf += len;
        transc->xfer_count += len;

        if(transc->xfer_count == transc->xfer_len) {
            /* disable the device endpoint FIFO empty interrupt */
            udev->regs.dr->DIEPFEINTEN &= ~(0x01U << ep_num);
            break;
        }
    }

    return 1U;
}
