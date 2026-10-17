use embedded_hal::delay::DelayNs;
use gd32vf103_pac::rcu::{ahben, Rcu};
use gd32vf103_pac::usbfs::{
    dcfg, dctl, depctl, diepinten, diepintf, doepinten, doepintf, gahbcs, gccfg, gintf, ginten,
    grstat, grstctl, gusbcs, Usbfs,
};
use crate::delay::Delay;
use super::{ControlState, SetupPacket, STR_LANG_ID, STR_MANUFACTURER, STR_SERIAL};

// ============================================================================
// USB Composite Device Driver (Standard HID + Custom HID)
// ============================================================================

const COMPOSITE_DEV_DESC: [u8; 18] = [
    18,         // bLength
    0x01,       // bDescriptorType = Device
    0x00, 0x02, // bcdUSB = 2.00
    0x00,       // bDeviceClass = 0 (Defined at interface level)
    0x00,       // bDeviceSubClass = 0
    0x00,       // bDeviceProtocol = 0
    64,         // bMaxPacketSize0 = 64
    0xE9, 0x28, // idVendor = 0x28E9
    0xE8, 0xAB, // idProduct = 0xABE8 (Fresh composite PID)
    0x00, 0x01, // bcdDevice = 1.00
    1,          // iManufacturer = String 1
    2,          // iProduct = String 2
    3,          // iSerialNumber = String 3
    1,          // bNumConfigurations = 1
];

const COMPOSITE_CONFIG_DESC: [u8; 66] = [
    // Configuration Descriptor (9 bytes)
    9,          // bLength
    0x02,       // bDescriptorType = Config
    66, 0,      // wTotalLength = 66
    2,          // bNumInterfaces = 2
    1,          // bConfigurationValue = 1
    0,          // iConfiguration = 0
    0x80,       // bmAttributes = Bus-powered
    50,         // bMaxPower = 100mA (50 * 2mA)

    // Interface 0: Standard HID (Keyboard, Mouse, Consumer) (9 bytes)
    9,          // bLength
    0x04,       // bDescriptorType = Interface
    0,          // bInterfaceNumber = 0
    0,          // bAlternateSetting = 0
    1,          // bNumEndpoints = 1
    0x03,       // bInterfaceClass = HID
    0x00,       // bInterfaceSubClass = 0
    0x00,       // bInterfaceProtocol = 0
    0,          // iInterface = 0

    // HID Descriptor 0 (9 bytes)
    9,          // bLength
    0x21,       // bDescriptorType = HID
    0x11, 0x01, // bcdHID = 1.11
    0x00,       // bCountryCode = 0
    1,          // bNumDescriptors = 1
    0x22,       // bDescriptorType = Report
    124, 0,     // wDescriptorLength = 124

    // Endpoint 1 IN: Interrupt IN (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x81,       // bEndpointAddress = EP1 IN
    0x03,       // bmAttributes = Interrupt
    16, 0,      // wMaxPacketSize = 16 (strictly >= 9 for keyboard reports)
    10,         // bInterval = 10ms

    // Interface 1: Custom HID (Image Data / Dynamic Display) (9 bytes)
    9,          // bLength
    0x04,       // bDescriptorType = Interface
    1,          // bInterfaceNumber = 1
    0,          // bAlternateSetting = 0
    2,          // bNumEndpoints = 2
    0x03,       // bInterfaceClass = HID
    0x00,       // bInterfaceSubClass = 0
    0x00,       // bInterfaceProtocol = 0
    0,          // iInterface = 0

    // HID Descriptor 1 (9 bytes)
    9,          // bLength
    0x21,       // bDescriptorType = HID
    0x11, 0x01, // bcdHID = 1.11
    0x00,       // bCountryCode = 0
    1,          // bNumDescriptors = 1
    0x22,       // bDescriptorType = Report
    27, 0,      // wDescriptorLength = 27

    // Endpoint 2 IN: Interrupt IN (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x82,       // bEndpointAddress = EP2 IN
    0x03,       // bmAttributes = Interrupt
    64, 0,      // wMaxPacketSize = 64
    1,          // bInterval = 1ms

    // Endpoint 2 OUT: Interrupt OUT (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x02,       // bEndpointAddress = EP2 OUT
    0x03,       // bmAttributes = Interrupt
    64, 0,      // wMaxPacketSize = 64
    1,          // bInterval = 1ms
];

const STD_HID_REPORT_DESC: [u8; 124] = [
    // Mouse (52 bytes)
    0x05, 0x01, 0x09, 0x02, 0xA1, 0x01, 0x85, 0x01, 0x09, 0x01, 0xA1, 0x00, 0x05, 0x09,
    0x19, 0x01, 0x29, 0x03, 0x15, 0x00, 0x25, 0x01, 0x95, 0x03, 0x75, 0x01, 0x81, 0x02,
    0x95, 0x01, 0x75, 0x05, 0x81, 0x01, 0x05, 0x01, 0x09, 0x30, 0x09, 0x31, 0x15, 0x81,
    0x25, 0x7F, 0x75, 0x08, 0x95, 0x02, 0x81, 0x06, 0xC0, 0xC0,
    // Keyboard (47 bytes)
    0x05, 0x01, 0x09, 0x06, 0xA1, 0x01, 0x85, 0x02, 0x05, 0x07, 0x19, 0xE0, 0x29, 0xE7,
    0x15, 0x00, 0x25, 0x01, 0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x95, 0x01, 0x75, 0x08,
    0x81, 0x01, 0x95, 0x06, 0x75, 0x08, 0x15, 0x00, 0x25, 0x65, 0x05, 0x07, 0x19, 0x00,
    0x29, 0x65, 0x81, 0x00, 0xC0,
    // Consumer (25 bytes)
    0x05, 0x0C, 0x09, 0x01, 0xA1, 0x01, 0x85, 0x03, 0x19, 0x00, 0x2A, 0x3C, 0x02, 0x15,
    0x00, 0x26, 0x3C, 0x02, 0x95, 0x01, 0x75, 0x10, 0x81, 0x00, 0xC0,
];

const COMPOSITE_CUSTOM_HID_REPORT_DESC: [u8; 27] = [
    0x06, 0x00, 0xFF, 0x09, 0x01, 0xA1, 0x01, 0x09, 0x02, 0x15, 0x00, 0x26, 0xFF, 0x00,
    0x75, 0x08, 0x95, 0x40, 0x91, 0x02, 0x09, 0x03, 0x95, 0x40, 0x81, 0x02, 0xC0,
];

const STR_COMPOSITE_PRODUCT: [u8; 58] = [
    58, 0x03,
    b'L', 0, b'o', 0, b'n', 0, b'g', 0, b'a', 0, b'n', 0,
    b' ', 0, b'N', 0, b'a', 0, b'n', 0, b'o', 0,
    b' ', 0, b'C', 0, b'o', 0, b'm', 0, b'p', 0, b'o', 0, b's', 0, b'i', 0, b't', 0, b'e', 0,
    b' ', 0, b'D', 0, b'e', 0, b'v', 0, b'i', 0, b'c', 0, b'e', 0,
];

pub struct UsbComposite {
    usbfs: Usbfs,
    ctl_state: ControlState,
    setup_pkt: SetupPacket,
    ep0_in_data: [u8; 128],
    ep0_in_len: usize,
    ep0_in_offset: usize,
    ep0_out_buf: [u8; 64],
    pending_addr: u8,
    configured: bool,
    ep1_busy: bool,
    ep2_busy: bool,
    idle_rate: u8,

    // Custom HID RX buffer
    custom_rx_buf: [u8; 64],
    custom_rx_len: usize,
    custom_rx_ready: bool,
}

impl UsbComposite {
    pub fn new(usbfs: Usbfs, rcu: &Rcu, delay: &mut Delay) -> Self {
        rcu.regs().ahben.set_bits(ahben::USBFSEN);
        usbfs.global().gusbcs.set_bits(gusbcs::EMBPHY | gusbcs::FDM);

        usbfs.global().grstctl.set_bits(grstctl::CSRST);
        let mut timeout = 100_000u32;
        while (usbfs.global().grstctl.read() & grstctl::CSRST) != 0 && timeout > 0 {
            timeout -= 1;
        }
        delay.delay_us(3);

        usbfs.global().gccfg.set_bits(
            gccfg::PWRON | gccfg::VBUSACEN | gccfg::VBUSBCEN | gccfg::SOFOEN | gccfg::VBUSIG,
        );

        delay.delay_ms(20);

        usbfs.device().dctl.set_bits(dctl::SDIS);
        delay.delay_ms(100);

        usbfs.pwrclkctl().write(0);

        usbfs.device().dcfg.modify(|val| {
            (val & !(dcfg::DS_MASK | dcfg::EOPFT_MASK)) | dcfg::DS_FULL | dcfg::EOPFT_80
        });

        // Allocate FIFOs: RX=128 words, TX0=32 words, TX1=16 words, TX2=64 words, TX3=16 words
        usbfs.global().grflen.write(128);
        usbfs.global().diep0tflen_hnptflen.write((32 << 16) | 128);
        usbfs.global().dieptflen[0].write((16 << 16) | (128 + 32));
        usbfs.global().dieptflen[1].write((64 << 16) | (128 + 32 + 16));
        usbfs.global().dieptflen[2].write((16 << 16) | (128 + 32 + 16 + 64));

        for i in 0..4 {
            usbfs.in_ep(i).diepctl.write(0);
            usbfs.out_ep(i).doepctl.write(0);
        }

        usbfs.global().grstctl.write((0x10 << 6) | grstctl::TXFF);
        timeout = 100_000;
        while (usbfs.global().grstctl.read() & grstctl::TXFF) != 0 && timeout > 0 {
            timeout -= 1;
        }
        delay.delay_us(3);

        usbfs.global().grstctl.write(grstctl::RXFF);
        timeout = 100_000;
        while (usbfs.global().grstctl.read() & grstctl::RXFF) != 0 && timeout > 0 {
            timeout -= 1;
        }
        delay.delay_us(3);

        usbfs.device().diepinten.write(0);
        usbfs.device().doepinten.write(0);
        usbfs.device().daepint.write(0xFFFF_FFFF);
        usbfs.device().daepinten.write(0);

        usbfs.global().gotgintf.write(0xFFFF_FFFF);
        usbfs.global().gintf.write(0xBFFF_FFFF);
        usbfs.global().ginten.write(
            ginten::WKUPIE
                | ginten::SPIE
                | ginten::RXFNEIE
                | ginten::RSTIE
                | ginten::ENUMFIE
                | ginten::IEPIE
                | ginten::OEPIE
                | ginten::SOFIE,
        );

        usbfs.global().gahbcs.set_bits(gahbcs::GINTEN);

        let mut instance = Self {
            usbfs,
            ctl_state: ControlState::Idle,
            setup_pkt: SetupPacket::default(),
            ep0_in_data: [0u8; 128],
            ep0_in_len: 0,
            ep0_in_offset: 0,
            ep0_out_buf: [0u8; 64],
            pending_addr: 0,
            configured: false,
            ep1_busy: false,
            ep2_busy: false,
            idle_rate: 0,
            custom_rx_buf: [0u8; 64],
            custom_rx_len: 0,
            custom_rx_ready: false,
        };

        instance.handle_reset();
        instance.usbfs.device().dctl.clear_bits(dctl::SDIS);
        delay.delay_ms(3);

        instance
    }

    /// Consumes the USB Composite driver, asserts a soft disconnect on the USB bus (DCTL.SDIS = 1),
    /// masks global AHB interrupts, and returns the underlying `Usbfs` peripheral.
    pub fn into_inner(self) -> Usbfs {
        self.usbfs.device().dctl.set_bits(dctl::SDIS);
        self.usbfs.global().gahbcs.clear_bits(gahbcs::GINTEN);
        self.usbfs
    }

    #[inline(always)]
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    #[inline(always)]
    pub fn is_std_hid_busy(&self) -> bool {
        self.ep1_busy
    }

    #[inline(always)]
    pub fn is_custom_hid_busy(&self) -> bool {
        self.ep2_busy
    }

    #[inline(always)]
    pub fn has_custom_rx(&self) -> bool {
        self.custom_rx_ready
    }

    pub fn read_custom_rx(&mut self, buf: &mut [u8]) -> usize {
        if !self.custom_rx_ready {
            return 0;
        }
        let len = core::cmp::min(buf.len(), self.custom_rx_len);
        buf[..len].copy_from_slice(&self.custom_rx_buf[..len]);
        self.custom_rx_ready = false;
        len
    }

    #[inline(always)]
    fn arm_ep0_setup(&self) {
        self.usbfs.out_ep(0).doeplen.write((24 << 0) | (1 << 19) | (3 << 29));
    }

    pub fn poll(&mut self) {
        let intr = self.usbfs.global().gintf.read();

        if (intr & gintf::RST) != 0 {
            self.handle_reset();
            return;
        }

        if (intr & gintf::ENUMFIF) != 0 {
            self.usbfs.device().dctl.set_bits(dctl::CGINAK);
            self.usbfs
                .global()
                .gusbcs
                .modify(|v| (v & !gusbcs::UTT_MASK) | (9 << 10));
            self.usbfs.global().gintf.write(gintf::ENUMFIF);
        }

        while (self.usbfs.global().gintf.read() & gintf::RXFNEIF) != 0 {
            self.handle_rxfifo();
        }

        let intr = self.usbfs.global().gintf.read();

        if (intr & gintf::OEPIF) != 0 {
            self.handle_epout();
        }

        if (intr & gintf::IEPIF) != 0 {
            self.handle_epin();
        }

        if (intr & gintf::SP) != 0 {
            self.usbfs.global().gintf.write(gintf::SP);
        }
        if (intr & gintf::WKUPIF) != 0 {
            self.usbfs.global().gintf.write(gintf::WKUPIF);
        }
        if (intr & gintf::SOF) != 0 {
            self.usbfs.global().gintf.write(gintf::SOF);
        }
    }

    fn handle_reset(&mut self) {
        self.usbfs.device().dctl.clear_bits(dctl::RWKUP);

        self.usbfs.global().grstctl.write((0x10 << 6) | grstctl::TXFF);
        while (self.usbfs.global().grstctl.read() & grstctl::TXFF) != 0 {}
        self.usbfs.global().grstctl.write(grstctl::RXFF);
        while (self.usbfs.global().grstctl.read() & grstctl::RXFF) != 0 {}

        for i in 1..4 {
            let epctl = self.usbfs.in_ep(i).diepctl.read();
            if (epctl & depctl::EPEN) != 0 {
                self.usbfs.in_ep(i).diepctl.write(
                    (epctl & !(depctl::SD0PID | depctl::SD1PID | depctl::CNAK))
                        | depctl::EPD
                        | depctl::SNAK,
                );
                let mut timeout = 1000u32;
                while (self.usbfs.in_ep(i).diepctl.read() & depctl::EPEN) != 0 && timeout > 0 {
                    timeout -= 1;
                }
            }
            self.usbfs.in_ep(i).diepctl.write(
                (epctl & !(depctl::EPEN | depctl::EPD | depctl::CNAK))
                    | depctl::SD0PID
                    | depctl::SNAK,
            );
            self.usbfs.in_ep(i).dieplen.write(0);
            self.usbfs.in_ep(i).diepintf.write(0xFF);
            self.usbfs.out_ep(i).doeplen.write(0);
            self.usbfs.out_ep(i).doepintf.write(0xFF);
        }

        self.usbfs.in_ep(0).dieplen.write(0);
        self.usbfs.in_ep(0).diepintf.write(0xFF);
        self.usbfs.out_ep(0).doeplen.write(0);
        self.usbfs.out_ep(0).doepintf.write(0xFF);

        self.usbfs
            .in_ep(0)
            .diepctl
            .write(depctl::EPACT | depctl::SD0PID);
        self.usbfs
            .out_ep(0)
            .doepctl
            .write(depctl::EPACT | depctl::SD0PID);

        self.pending_addr = 0;
        self.configured = false;
        self.ep1_busy = false;
        self.ep2_busy = false;
        self.ctl_state = ControlState::Idle;
        self.usbfs.device().dcfg.clear_bits(dcfg::DAR_MASK);

        self.usbfs.device().daepint.write(0xFFFF_FFFF);
        self.usbfs.device().daepinten.write(1 | (1 << 16));
        self.usbfs
            .device()
            .doepinten
            .write(doepinten::STPFEN | doepinten::TFEN);
        self.usbfs.device().diepinten.write(diepinten::TFEN);

        self.arm_ep0_setup();
        self.usbfs.global().gintf.write(gintf::RST);
    }

    fn handle_rxfifo(&mut self) {
        self.usbfs.global().ginten.clear_bits(ginten::RXFNEIE);
        let rxstat = self.usbfs.global().grstatp.read();
        self.usbfs.global().ginten.set_bits(ginten::RXFNEIE);

        let ep_num = (rxstat & grstat::EPNUM_MASK) as usize;
        let bcount = ((rxstat & grstat::BCOUNT_MASK) >> 4) as usize;
        let pktstatus = (rxstat & grstat::RPCKST_MASK) >> 17;

        if ep_num >= 4 {
            return;
        }

        match pktstatus {
            grstat::RSTAT_SETUP_UPDT => {
                if ep_num == 0 && bcount == 8 {
                    let w0 = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                    let w1 = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                    self.setup_pkt = SetupPacket {
                        bm_request_type: (w0 & 0xFF) as u8,
                        b_request: ((w0 >> 8) & 0xFF) as u8,
                        w_value: ((w0 >> 16) & 0xFFFF) as u16,
                        w_index: (w1 & 0xFFFF) as u16,
                        w_length: ((w1 >> 16) & 0xFFFF) as u16,
                    };
                }
            }
            grstat::RSTAT_DATA_UPDT => {
                if bcount > 0 {
                    let word_count = (bcount + 3) / 4;
                    let mut byte_idx = 0;
                    for _ in 0..word_count {
                        let w = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                        for b in 0..4 {
                            if byte_idx < bcount {
                                let byte = ((w >> (b * 8)) & 0xFF) as u8;
                                if ep_num == 0 && byte_idx < self.ep0_out_buf.len() {
                                    self.ep0_out_buf[byte_idx] = byte;
                                } else if ep_num == 2 && byte_idx < self.custom_rx_buf.len() {
                                    self.custom_rx_buf[byte_idx] = byte;
                                    self.custom_rx_len = byte_idx + 1;
                                    self.custom_rx_ready = true;
                                }
                                byte_idx += 1;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_epout(&mut self) {
        let daepint = self.usbfs.device().daepint.read();

        // Check EP0 OUT
        if (daepint & (1 << 16)) != 0 {
            let oepintr = self.usbfs.out_ep(0).doepintf.read();

            if (oepintr & doepintf::STPF) != 0 {
                self.usbfs.out_ep(0).doepintf.write(doepintf::STPF);
                self.handle_setup();
            }

            if (oepintr & doepintf::TF) != 0 {
                self.usbfs.out_ep(0).doepintf.write(doepintf::TF);
                match self.ctl_state {
                    ControlState::StatusOut => {
                        self.ctl_state = ControlState::Idle;
                        self.arm_ep0_setup();
                    }
                    ControlState::DataOut => {
                        self.ep0_send_status_in();
                    }
                    _ => {}
                }
            }

            let residual = self.usbfs.out_ep(0).doepintf.read();
            if residual != 0 {
                self.usbfs.out_ep(0).doepintf.write(residual);
            }
        }

        // Check EP2 OUT (Custom HID OUT)
        if (daepint & (1 << (16 + 2))) != 0 {
            let oepintr = self.usbfs.out_ep(2).doepintf.read();
            if (oepintr & doepintf::TF) != 0 {
                self.usbfs.out_ep(2).doepintf.write(doepintf::TF);
                self.usbfs.out_ep(2).doeplen.write((1 << 19) | 64);
                let ctl = self.usbfs.out_ep(2).doepctl.read();
                self.usbfs.out_ep(2).doepctl.write(
                    (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                        | depctl::EPEN
                        | depctl::CNAK,
                );
            }
            let residual = self.usbfs.out_ep(2).doepintf.read();
            if residual != 0 {
                self.usbfs.out_ep(2).doepintf.write(residual);
            }
        }
    }

    fn handle_epin(&mut self) {
        let daepint = self.usbfs.device().daepint.read();

        // Check EP0 IN
        if (daepint & (1 << 0)) != 0 {
            let iepintr = self.usbfs.in_ep(0).diepintf.read();

            if (iepintr & diepintf::TF) != 0 {
                self.usbfs.in_ep(0).diepintf.write(diepintf::TF);

                match self.ctl_state {
                    ControlState::DataIn => {
                        self.ep0_send_next_packet();
                    }
                    ControlState::LastDataIn => {
                        self.ctl_state = ControlState::StatusOut;
                        self.usbfs.out_ep(0).doeplen.write(1 << 19);
                        let ctl = self.usbfs.out_ep(0).doepctl.read();
                        self.usbfs.out_ep(0).doepctl.write(
                            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                                | depctl::EPEN
                                | depctl::CNAK,
                        );
                    }
                    ControlState::StatusIn => {
                        self.ctl_state = ControlState::Idle;
                        self.arm_ep0_setup();
                    }
                    _ => {}
                }
            }

            let residual = self.usbfs.in_ep(0).diepintf.read();
            if residual != 0 {
                self.usbfs.in_ep(0).diepintf.write(residual);
            }
        }

        // Check EP1 IN (Standard HID IN)
        if (daepint & (1 << 1)) != 0 {
            let iepintr = self.usbfs.in_ep(1).diepintf.read();
            if (iepintr & diepintf::TF) != 0 {
                self.usbfs.in_ep(1).diepintf.write(diepintf::TF);
                self.ep1_busy = false;
            }
            let residual = self.usbfs.in_ep(1).diepintf.read();
            if residual != 0 {
                self.usbfs.in_ep(1).diepintf.write(residual);
            }
        }

        // Check EP2 IN (Custom HID IN)
        if (daepint & (1 << 2)) != 0 {
            let iepintr = self.usbfs.in_ep(2).diepintf.read();
            if (iepintr & diepintf::TF) != 0 {
                self.usbfs.in_ep(2).diepintf.write(diepintf::TF);
                self.ep2_busy = false;
            }
            let residual = self.usbfs.in_ep(2).diepintf.read();
            if residual != 0 {
                self.usbfs.in_ep(2).diepintf.write(residual);
            }
        }
    }

    fn handle_setup(&mut self) {
        self.usbfs.in_ep(0).diepctl.clear_bits(depctl::STALL);
        self.usbfs.out_ep(0).doepctl.clear_bits(depctl::STALL);

        let req = self.setup_pkt;
        let req_type = req.bm_request_type & 0x60;

        match req_type {
            0x00 => match req.b_request {
                0x06 => {
                    let desc_type = (req.w_value >> 8) as u8;
                    let desc_index = (req.w_value & 0xFF) as u8;

                    match desc_type {
                        1 => {
                            let mut len = COMPOSITE_DEV_DESC.len();
                            if req.w_length == 64 {
                                len = 8;
                            } else if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&COMPOSITE_DEV_DESC[..len]);
                        }
                        2 => {
                            let mut len = COMPOSITE_CONFIG_DESC.len();
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&COMPOSITE_CONFIG_DESC[..len]);
                        }
                        3 => {
                            if desc_index >= 4 {
                                self.stall_ep0();
                            } else {
                                let str_bytes: &[u8] = match desc_index {
                                    0 => &STR_LANG_ID,
                                    1 => &STR_MANUFACTURER,
                                    2 => &STR_COMPOSITE_PRODUCT,
                                    3 => &STR_SERIAL,
                                    _ => unreachable!(),
                                };
                                let mut len = str_bytes.len();
                                if (req.w_length as usize) < len {
                                    len = req.w_length as usize;
                                }
                                self.ep0_start_in(&str_bytes[..len]);
                            }
                        }
                        0x21 => {
                            let (offset, size) = if (req.w_index & 0xFF) == 0 {
                                (18, 9)
                            } else {
                                (43, 9)
                            };
                            let mut len = size;
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&COMPOSITE_CONFIG_DESC[offset..offset + len]);
                        }
                        0x22 => {
                            let report_slice: &[u8] = if (req.w_index & 0xFF) == 0 {
                                &STD_HID_REPORT_DESC
                            } else {
                                &COMPOSITE_CUSTOM_HID_REPORT_DESC
                            };
                            let mut len = report_slice.len();
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&report_slice[..len]);
                        }
                        _ => {
                            self.stall_ep0();
                        }
                    }
                }
                0x05 => {
                    let addr = (req.w_value & 0x7F) as u8;
                    self.usbfs.device().dcfg.modify(|v| {
                        (v & !dcfg::DAR_MASK) | ((addr as u32) << 4)
                    });
                    self.ep0_send_status_in();
                }
                0x09 => {
                    let cfg = (req.w_value & 0xFF) as u8;
                    if cfg == 1 {
                        // EP1 IN: Standard HID (max 16, FIFO 1)
                        self.usbfs.in_ep(1).diepctl.write(
                            (16 << 0)
                                | depctl::EPACT
                                | (3 << 18)
                                | (1 << 22)
                                | depctl::SD0PID
                                | depctl::SNAK,
                        );

                        // EP2 IN: Custom HID (max 64, FIFO 2)
                        self.usbfs.in_ep(2).diepctl.write(
                            (64 << 0)
                                | depctl::EPACT
                                | (3 << 18)
                                | (2 << 22)
                                | depctl::SD0PID
                                | depctl::SNAK,
                        );

                        // EP2 OUT: Custom HID (max 64)
                        self.usbfs.out_ep(2).doepctl.write(
                            (64 << 0) | depctl::EPACT | (3 << 18) | depctl::SD0PID | depctl::SNAK,
                        );
                        self.usbfs.out_ep(2).doeplen.write((1 << 19) | 64);
                        self.usbfs.out_ep(2).doepctl.modify(|v| {
                            (v & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                                | depctl::EPEN
                                | depctl::CNAK
                        });

                        self.usbfs.device().daepinten.write(
                            (1 << 0) | (1 << 1) | (1 << 2) | (1 << 16) | (1 << (16 + 2)),
                        );

                        self.configured = true;
                        self.ep1_busy = false;
                        self.ep2_busy = false;
                    } else {
                        self.configured = false;
                    }
                    self.ep0_send_status_in();
                }
                0x08 => {
                    let val = if self.configured { 1u8 } else { 0u8 };
                    self.ep0_start_in(&[val]);
                }
                0x00 => {
                    self.ep0_start_in(&[0, 0]);
                }
                0x01 | 0x03 => {
                    let ep_addr = req.w_index as u8;
                    let ep_num = (ep_addr & 0x0F) as usize;
                    if ep_num < 4 {
                        if (ep_addr & 0x80) != 0 {
                            if req.b_request == 0x01 {
                                self.usbfs.in_ep(ep_num).diepctl.clear_bits(depctl::STALL);
                            } else {
                                self.usbfs.in_ep(ep_num).diepctl.set_bits(depctl::STALL);
                            }
                        } else {
                            if req.b_request == 0x01 {
                                self.usbfs.out_ep(ep_num).doepctl.clear_bits(depctl::STALL);
                            } else {
                                self.usbfs.out_ep(ep_num).doepctl.set_bits(depctl::STALL);
                            }
                        }
                    }
                    self.ep0_send_status_in();
                }
                _ => {
                    self.stall_ep0();
                }
            },

            0x20 | 0x21 => match req.b_request {
                0x0A => {
                    self.idle_rate = (req.w_value >> 8) as u8;
                    self.ep0_send_status_in();
                }
                0x02 => {
                    let rate = self.idle_rate;
                    self.ep0_start_in(&[rate]);
                }
                0x01 => {
                    self.ep0_start_in(&[0]);
                }
                _ => {
                    self.stall_ep0();
                }
            },

            _ => {
                self.stall_ep0();
            }
        }
    }

    fn ep0_start_in(&mut self, data: &[u8]) {
        let len = data.len();
        self.ep0_in_data[..len].copy_from_slice(data);
        self.ep0_in_len = len;
        self.ep0_in_offset = 0;
        self.ep0_send_next_packet();
    }

    fn ep0_send_next_packet(&mut self) {
        let remaining = self.ep0_in_len - self.ep0_in_offset;
        let chunk_len = core::cmp::min(remaining, 64);

        if remaining > 64 {
            self.ctl_state = ControlState::DataIn;
        } else {
            self.ctl_state = ControlState::LastDataIn;
        }

        self.usbfs
            .in_ep(0)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs.in_ep(0).dieplen.write((1 << 19) | (chunk_len as u32));
        let ctl = self.usbfs.in_ep(0).diepctl.read();
        self.usbfs.in_ep(0).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        if chunk_len > 0 {
            self.write_fifo(
                0,
                &self.ep0_in_data[self.ep0_in_offset..self.ep0_in_offset + chunk_len],
            );
        }

        self.ep0_in_offset += chunk_len;
    }

    fn ep0_send_status_in(&mut self) {
        self.ctl_state = ControlState::StatusIn;
        self.usbfs
            .in_ep(0)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs.in_ep(0).dieplen.write(1 << 19);
        let ctl = self.usbfs.in_ep(0).diepctl.read();
        self.usbfs.in_ep(0).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );
    }

    fn stall_ep0(&mut self) {
        self.ctl_state = ControlState::Stall;
        self.usbfs.in_ep(0).diepctl.set_bits(depctl::STALL);
        self.usbfs.out_ep(0).doepctl.set_bits(depctl::STALL);
        self.arm_ep0_setup();
    }

    fn write_fifo(&self, ep_num: usize, data: &[u8]) {
        let fifo_ptr = self.usbfs.fifo(ep_num);
        let len = data.len();
        let word_count = (len + 3) / 4;

        for i in 0..word_count {
            let mut word = 0u32;
            for b in 0..4 {
                let idx = i * 4 + b;
                if idx < len {
                    word |= (data[idx] as u32) << (b * 8);
                }
            }
            unsafe {
                core::ptr::write_volatile(fifo_ptr, word);
            }
        }
    }

    /// Sends a consumer control report (Report ID 3, 3 bytes) on EP1 IN.
    pub fn send_consumer_report(&mut self, key: u16) -> bool {
        if !self.configured || self.ep1_busy {
            return false;
        }

        let report: [u8; 3] = [0x03, (key & 0xFF) as u8, (key >> 8) as u8];

        self.usbfs
            .in_ep(1)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs.in_ep(1).dieplen.write((1 << 19) | 3);
        let ctl = self.usbfs.in_ep(1).diepctl.read();
        self.usbfs.in_ep(1).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        self.write_fifo(1, &report);
        self.ep1_busy = true;
        true
    }

    /// Sends a keyboard report (Report ID 2, 9 bytes) on EP1 IN.
    pub fn send_keyboard_report(&mut self, modifier: u8, keycodes: &[u8; 6]) -> bool {
        if !self.configured || self.ep1_busy {
            return false;
        }

        let report: [u8; 9] = [
            0x02,
            modifier,
            0x00,
            keycodes[0],
            keycodes[1],
            keycodes[2],
            keycodes[3],
            keycodes[4],
            keycodes[5],
        ];

        self.usbfs
            .in_ep(1)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs.in_ep(1).dieplen.write((1 << 19) | 9);
        let ctl = self.usbfs.in_ep(1).diepctl.read();
        self.usbfs.in_ep(1).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        self.write_fifo(1, &report);
        self.ep1_busy = true;
        true
    }

    /// Sends a custom HID report on EP2 IN.
    pub fn send_custom_report(&mut self, report: &[u8]) -> bool {
        if !self.configured || self.ep2_busy {
            return false;
        }

        let len = core::cmp::min(report.len(), 64);

        self.usbfs
            .in_ep(2)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs.in_ep(2).dieplen.write((1 << 19) | (len as u32));
        let ctl = self.usbfs.in_ep(2).diepctl.read();
        self.usbfs.in_ep(2).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        self.write_fifo(2, &report[..len]);
        self.ep2_busy = true;
        true
    }
}
