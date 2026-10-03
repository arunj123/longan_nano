use core::fmt;
use embedded_hal::delay::DelayNs;
use gd32vf103_pac::rcu::{ahben, Rcu};
use gd32vf103_pac::usbfs::{
    dcfg, dctl, depctl, diepinten, diepintf, doepinten, doepintf, gahbcs, gccfg, gintf, ginten,
    grstat, grstctl, gusbcs, Usbfs,
};
use crate::delay::Delay;

// USB Standard Device Descriptor (18 bytes)
const DEV_DESC: [u8; 18] = [
    18,         // bLength
    0x01,       // bDescriptorType = Device
    0x00, 0x02, // bcdUSB = 2.00
    0x02,       // bDeviceClass = CDC
    0x00,       // bDeviceSubClass = 0
    0x00,       // bDeviceProtocol = 0
    64,         // bMaxPacketSize0 = 64
    0xE9, 0x28, // idVendor = 0x28E9 (GD)
    0x8A, 0x01, // idProduct = 0x018A
    0x00, 0x01, // bcdDevice = 1.00
    1,          // iManufacturer = String 1
    2,          // iProduct = String 2
    3,          // iSerialNumber = String 3
    1,          // bNumConfigurations = 1
];

// USB Configuration Descriptor Set (67 bytes)
const CONFIG_DESC: [u8; 67] = [
    // Configuration Descriptor (9 bytes)
    9,          // bLength
    0x02,       // bDescriptorType = Config
    67, 0,      // wTotalLength = 67
    2,          // bNumInterfaces = 2
    1,          // bConfigurationValue = 1
    0,          // iConfiguration = 0
    0x80,       // bmAttributes = Bus-powered
    50,         // bMaxPower = 100mA (50 * 2mA)

    // Interface 0: CDC Communication (9 bytes)
    9,          // bLength
    0x04,       // bDescriptorType = Interface
    0,          // bInterfaceNumber = 0
    0,          // bAlternateSetting = 0
    1,          // bNumEndpoints = 1
    0x02,       // bInterfaceClass = CDC
    0x02,       // bInterfaceSubClass = ACM
    0x01,       // bInterfaceProtocol = AT Commands
    0,          // iInterface = 0

    // Header Functional Descriptor (5 bytes)
    5,          // bLength
    0x24,       // bDescriptorType = CS_INTERFACE
    0x00,       // bDescriptorSubtype = Header
    0x10, 0x01, // bcdCDC = 1.10

    // Call Management Functional Descriptor (5 bytes)
    5,          // bLength
    0x24,       // bDescriptorType = CS_INTERFACE
    0x01,       // bDescriptorSubtype = Call Management
    0x00,       // bmCapabilities = none
    1,          // bDataInterface = 1

    // ACM Functional Descriptor (4 bytes)
    4,          // bLength
    0x24,       // bDescriptorType = CS_INTERFACE
    0x02,       // bDescriptorSubtype = ACM
    0x02,       // bmCapabilities = Line Coding & Serial State

    // Union Functional Descriptor (5 bytes)
    5,          // bLength
    0x24,       // bDescriptorType = CS_INTERFACE
    0x06,       // bDescriptorSubtype = Union
    0,          // bMasterInterface = 0
    1,          // bSlaveInterface0 = 1

    // Endpoint 2 IN: CDC Notification Interrupt (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x82,       // bEndpointAddress = EP2 IN
    0x03,       // bmAttributes = Interrupt
    8, 0,       // wMaxPacketSize = 8
    10,         // bInterval = 10ms

    // Interface 1: CDC Data (9 bytes)
    9,          // bLength
    0x04,       // bDescriptorType = Interface
    1,          // bInterfaceNumber = 1
    0,          // bAlternateSetting = 0
    2,          // bNumEndpoints = 2
    0x0A,       // bInterfaceClass = CDC Data
    0x00,       // bInterfaceSubClass = 0
    0x00,       // bInterfaceProtocol = 0
    0,          // iInterface = 0

    // Endpoint 3 OUT: Bulk OUT (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x03,       // bEndpointAddress = EP3 OUT
    0x02,       // bmAttributes = Bulk
    64, 0,      // wMaxPacketSize = 64
    0,          // bInterval = 0

    // Endpoint 1 IN: Bulk IN (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x81,       // bEndpointAddress = EP1 IN
    0x02,       // bmAttributes = Bulk
    64, 0,      // wMaxPacketSize = 64
    0,          // bInterval = 0
];

// String Descriptors
const STR_LANG_ID: [u8; 4] = [4, 0x03, 0x09, 0x04]; // English (0x0409)

const STR_MANUFACTURER: [u8; 24] = [
    24, 0x03,
    b'L', 0, b'o', 0, b'n', 0, b'g', 0, b'a', 0, b'n', 0,
    b' ', 0, b'N', 0, b'a', 0, b'n', 0, b'o', 0,
];

const STR_PRODUCT: [u8; 46] = [
    46, 0x03,
    b'L', 0, b'o', 0, b'n', 0, b'g', 0, b'a', 0, b'n', 0,
    b' ', 0, b'N', 0, b'a', 0, b'n', 0, b'o', 0,
    b' ', 0, b'U', 0, b'S', 0, b'B', 0,
    b' ', 0, b'S', 0, b'e', 0, b'r', 0, b'i', 0, b'a', 0, b'l', 0,
];

const STR_SERIAL: [u8; 26] = [
    26, 0x03,
    b'3', 0, b'9', 0, b'4', 0, b'1', 0, b'3', 0, b'1', 0,
    b'1', 0, b'9', 0, b'0', 0, b'1', 0, b'0', 0, b'3', 0,
];

#[derive(Copy, Clone, PartialEq, Eq)]
enum ControlState {
    Idle,
    DataIn,
    LastDataIn,
    StatusIn,
    StatusOut,
    DataOut,
    Stall,
}

#[derive(Copy, Clone, Default)]
struct SetupPacket {
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
}

const RX_BUF_SIZE: usize = 256;

pub struct UsbCdcAcm {
    usbfs: Usbfs,
    ctl_state: ControlState,
    setup_pkt: SetupPacket,
    ep0_in_data: [u8; 128],
    ep0_in_len: usize,
    ep0_in_offset: usize,
    ep0_out_buf: [u8; 64],
    pending_addr: u8,
    configured: bool,
    tx_busy: bool,
    line_coding: [u8; 7],

    // Simple FIFO ring buffer for CDC received data
    rx_buf: [u8; RX_BUF_SIZE],
    rx_head: usize,
    rx_tail: usize,
}

impl UsbCdcAcm {
    pub fn new(usbfs: Usbfs, rcu: &Rcu, delay: &mut Delay) -> Self {
        // Enable USBFS peripheral clock in RCU AHBEN
        rcu.regs().ahben.set_bits(ahben::USBFSEN);

        // Core Init: embedded PHY and Force Device Mode (FDM = 1 << 30)
        usbfs.global().gusbcs.set_bits(gusbcs::EMBPHY | gusbcs::FDM);

        // Reset core
        usbfs.global().grstctl.set_bits(grstctl::CSRST);
        let mut timeout = 100_000u32;
        while (usbfs.global().grstctl.read() & grstctl::CSRST) != 0 && timeout > 0 {
            timeout -= 1;
        }
        delay.delay_us(3);

        // Power on transceiver and VBUS detection ignore
        usbfs.global().gccfg.set_bits(
            gccfg::PWRON | gccfg::VBUSACEN | gccfg::VBUSBCEN | gccfg::SOFOEN | gccfg::VBUSIG,
        );

        // Delay 20 ms for PHY stabilization
        delay.delay_ms(20);

        // Soft disconnect before initializing device registers
        usbfs.device().dctl.set_bits(dctl::SDIS);
        delay.delay_ms(100);

        // Restart PHY clock
        usbfs.pwrclkctl().write(0);

        // Full speed device configuration & 80% EOF threshold
        usbfs.device().dcfg.modify(|val| {
            (val & !(dcfg::DS_MASK | dcfg::EOPFT_MASK)) | dcfg::DS_FULL | dcfg::EOPFT_80
        });

        // Allocate FIFOs: RX=128 words, TX0=32 words, TX1=64 words, TX2=16 words
        usbfs.global().grflen.write(128);
        usbfs.global().diep0tflen_hnptflen.write((32 << 16) | 128);
        usbfs.global().dieptflen[0].write((64 << 16) | (128 + 32));
        usbfs.global().dieptflen[1].write((16 << 16) | (128 + 32 + 64));

        // Inactive endpoints zeroed (Mandatory Invariant 4)
        for i in 0..4 {
            usbfs.in_ep(i).diepctl.write(0);
            usbfs.out_ep(i).doepctl.write(0);
        }

        // Flush FIFOs
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

        // Clear and mask interrupts
        usbfs.device().diepinten.write(0);
        usbfs.device().doepinten.write(0);
        usbfs.device().daepint.write(0xFFFF_FFFF);
        usbfs.device().daepinten.write(0);

        // Unmask core interrupts
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

        // Unmask global interrupt in AHB
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
            tx_busy: false,
            line_coding: [
                0x00, 0xC2, 0x01, 0x00, // 115200 baud
                0x00,                   // 1 stop bit
                0x00,                   // no parity
                0x08,                   // 8 data bits
            ],
            rx_buf: [0u8; RX_BUF_SIZE],
            rx_head: 0,
            rx_tail: 0,
        };

        // Prepare EP0 endpoints
        instance.handle_reset();

        // Connect device (clear SDIS)
        instance.usbfs.device().dctl.clear_bits(dctl::SDIS);
        delay.delay_ms(3);

        instance
    }

    #[inline(always)]
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    /// Arm EP0 OUT to receive SETUP packets (Mandatory Invariant 2)
    #[inline(always)]
    fn arm_ep0_setup(&self) {
        self.usbfs.out_ep(0).doeplen.write((24 << 0) | (1 << 19) | (3 << 29));
    }

    /// High-frequency non-blocking USB polling engine.
    /// Services hardware events, control transfers, and CDC data streams.
    pub fn poll(&mut self) {
        let intr = self.usbfs.global().gintf.read();

        // 1. Bus Reset
        if (intr & gintf::RST) != 0 {
            self.handle_reset();
            return;
        }

        // 2. Enumeration Finished
        if (intr & gintf::ENUMFIF) != 0 {
            self.usbfs.device().dctl.set_bits(dctl::CGINAK);
            // Set 48 MHz PHY turnaround time (UTT = 9)
            self.usbfs
                .global()
                .gusbcs
                .modify(|v| (v & !gusbcs::UTT_MASK) | (9 << 10));
            self.usbfs.global().gintf.write(gintf::ENUMFIF);
        }

        // 3. RX FIFO Non-Empty: drain all pending status queue packets
        while (self.usbfs.global().gintf.read() & gintf::RXFNEIF) != 0 {
            self.handle_rxfifo();
        }

        let intr = self.usbfs.global().gintf.read();

        // 4. OUT Endpoint Interrupt
        if (intr & gintf::OEPIF) != 0 {
            self.handle_epout();
        }

        // 5. IN Endpoint Interrupt
        if (intr & gintf::IEPIF) != 0 {
            self.handle_epin();
        }

        // 6. Suspend / Wakeup / SOF
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

        // Flush FIFOs
        self.usbfs.global().grstctl.write((0x10 << 6) | grstctl::TXFF);
        while (self.usbfs.global().grstctl.read() & grstctl::TXFF) != 0 {}
        self.usbfs.global().grstctl.write(grstctl::RXFF);
        while (self.usbfs.global().grstctl.read() & grstctl::RXFF) != 0 {}

        // Reset non-control endpoints
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

        // EP0 state reset (Invariant: never set SNAK on EP0)
        self.usbfs.in_ep(0).dieplen.write(0);
        self.usbfs.in_ep(0).diepintf.write(0xFF);
        self.usbfs.out_ep(0).doeplen.write(0);
        self.usbfs.out_ep(0).doepintf.write(0xFF);

        // Activate EP0 with DATA0 and EPACT (Mandatory Invariant)
        self.usbfs
            .in_ep(0)
            .diepctl
            .write(depctl::EPACT | depctl::SD0PID);
        self.usbfs
            .out_ep(0)
            .doepctl
            .write(depctl::EPACT | depctl::SD0PID);

        // Reset address & status
        self.pending_addr = 0;
        self.configured = false;
        self.tx_busy = false;
        self.ctl_state = ControlState::Idle;
        self.usbfs.device().dcfg.clear_bits(dcfg::DAR_MASK);

        // Enable EP0 interrupts in device controller
        self.usbfs.device().daepint.write(0xFFFF_FFFF);
        self.usbfs.device().daepinten.write(1 | (1 << 16));
        self.usbfs
            .device()
            .doepinten
            .write(doepinten::STPFEN | doepinten::TFEN);
        self.usbfs.device().diepinten.write(diepinten::TFEN);

        // Arm EP0 OUT for SETUP packet (Mandatory Invariant 2)
        self.arm_ep0_setup();

        self.usbfs.global().gintf.write(gintf::RST);
    }

    fn handle_rxfifo(&mut self) {
        // Pop status from GRSTATP with RXFNEIE temporarily masked
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
                                if ep_num == 0 {
                                    if byte_idx < self.ep0_out_buf.len() {
                                        self.ep0_out_buf[byte_idx] = byte;
                                    }
                                } else if ep_num == 3 {
                                    // CDC Bulk OUT: Push to ring buffer
                                    let next_head = (self.rx_head + 1) % RX_BUF_SIZE;
                                    if next_head != self.rx_tail {
                                        self.rx_buf[self.rx_head] = byte;
                                        self.rx_head = next_head;
                                    }
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
                        // Host completed Status OUT stage (Mandatory Invariant 2)
                        self.ctl_state = ControlState::Idle;
                        self.arm_ep0_setup();
                    }
                    ControlState::DataOut => {
                        // Data stage completed (e.g., SET_LINE_CODING payload)
                        if self.setup_pkt.b_request == 0x20 && self.setup_pkt.w_length == 7 {
                            self.line_coding.copy_from_slice(&self.ep0_out_buf[..7]);
                        }
                        self.ep0_send_status_in();
                    }
                    _ => {}
                }
            }

            // Clear any residual flags
            let residual = self.usbfs.out_ep(0).doepintf.read();
            if residual != 0 {
                self.usbfs.out_ep(0).doepintf.write(residual);
            }
        }

        // Check EP3 OUT (CDC Data OUT)
        if (daepint & (1 << (16 + 3))) != 0 {
            let oepintr = self.usbfs.out_ep(3).doepintf.read();
            if (oepintr & doepintf::TF) != 0 {
                self.usbfs.out_ep(3).doepintf.write(doepintf::TF);
                // Re-arm EP3 OUT for next packet
                self.usbfs.out_ep(3).doeplen.write((1 << 19) | 64);
                let ctl = self.usbfs.out_ep(3).doepctl.read();
                self.usbfs.out_ep(3).doepctl.write(
                    (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                        | depctl::EPEN
                        | depctl::CNAK,
                );
            }
            let residual = self.usbfs.out_ep(3).doepintf.read();
            if residual != 0 {
                self.usbfs.out_ep(3).doepintf.write(residual);
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
                        // Data stage done: arm EP0 OUT for 0-byte Status OUT from host
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

        // Check EP1 IN (CDC Data IN)
        if (daepint & (1 << 1)) != 0 {
            let iepintr = self.usbfs.in_ep(1).diepintf.read();
            if (iepintr & diepintf::TF) != 0 {
                self.usbfs.in_ep(1).diepintf.write(diepintf::TF);
                self.tx_busy = false;
            }
            let residual = self.usbfs.in_ep(1).diepintf.read();
            if residual != 0 {
                self.usbfs.in_ep(1).diepintf.write(residual);
            }
        }
    }

    fn handle_setup(&mut self) {
        // Clear any previous STALL on EP0 (USB 2.0 §8.5.3.4)
        self.usbfs.in_ep(0).diepctl.clear_bits(depctl::STALL);
        self.usbfs.out_ep(0).doepctl.clear_bits(depctl::STALL);

        let req = self.setup_pkt;
        let req_type = req.bm_request_type & 0x60;

        match req_type {
            // Standard Requests
            0x00 => match req.b_request {
                // GET_DESCRIPTOR
                0x06 => {
                    let desc_type = (req.w_value >> 8) as u8;
                    let desc_index = (req.w_value & 0xFF) as u8;

                    match desc_type {
                        // Device Descriptor
                        1 => {
                            let mut len = DEV_DESC.len();
                            // Windows xHCI 8-Byte Initial Probe Invariant (Mandatory Invariant 5)
                            if req.w_length == 64 {
                                len = 8;
                            } else if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&DEV_DESC[..len]);
                        }
                        // Configuration Descriptor
                        2 => {
                            let mut len = CONFIG_DESC.len();
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&CONFIG_DESC[..len]);
                        }
                        // String Descriptors
                        3 => {
                            // String Descriptor Bounds Invariant (Mandatory Invariant 6)
                            if desc_index >= 4 {
                                self.stall_ep0();
                            } else {
                                let str_bytes: &[u8] = match desc_index {
                                    0 => &STR_LANG_ID,
                                    1 => &STR_MANUFACTURER,
                                    2 => &STR_PRODUCT,
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
                        // Out of bounds descriptor type (e.g. BOS descriptor)
                        _ => {
                            self.stall_ep0();
                        }
                    }
                }
                // SET_ADDRESS
                0x05 => {
                    let addr = (req.w_value & 0x7F) as u8;
                    self.usbfs.device().dcfg.modify(|v| {
                        (v & !dcfg::DAR_MASK) | ((addr as u32) << 4)
                    });
                    self.ep0_send_status_in();
                }
                // SET_CONFIGURATION
                0x09 => {
                    let cfg = (req.w_value & 0xFF) as u8;
                    if cfg == 1 {
                        // Configure EP1 IN (Bulk IN, max 64, FIFO 1)
                        self.usbfs.in_ep(1).diepctl.write(
                            (64 << 0)
                                | depctl::EPACT
                                | (2 << 18)
                                | (1 << 22)
                                | depctl::SD0PID
                                | depctl::SNAK,
                        );

                        // Configure EP3 OUT (Bulk OUT, max 64)
                        self.usbfs.out_ep(3).doepctl.write(
                            (64 << 0) | depctl::EPACT | (2 << 18) | depctl::SD0PID | depctl::SNAK,
                        );
                        // Arm EP3 OUT to receive data
                        self.usbfs.out_ep(3).doeplen.write((1 << 19) | 64);
                        self.usbfs.out_ep(3).doepctl.modify(|v| {
                            (v & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                                | depctl::EPEN
                                | depctl::CNAK
                        });

                        // Configure EP2 IN (Interrupt IN, max 8, FIFO 2)
                        self.usbfs.in_ep(2).diepctl.write(
                            (8 << 0)
                                | depctl::EPACT
                                | (3 << 18)
                                | (2 << 22)
                                | depctl::SD0PID
                                | depctl::SNAK,
                        );

                        // Enable endpoint interrupts in DAEPINTEN
                        self.usbfs
                            .device()
                            .daepinten
                            .write((1 << 0) | (1 << 1) | (1 << 2) | (1 << 16) | (1 << (16 + 3)));

                        self.configured = true;
                        self.tx_busy = false;
                    } else {
                        self.configured = false;
                    }
                    self.ep0_send_status_in();
                }
                // GET_CONFIGURATION
                0x08 => {
                    let val = if self.configured { 1u8 } else { 0u8 };
                    self.ep0_start_in(&[val]);
                }
                // GET_STATUS
                0x00 => {
                    self.ep0_start_in(&[0, 0]);
                }
                // CLEAR_FEATURE / SET_FEATURE
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

            // CDC Class Requests
            0x20 => match req.b_request {
                // SET_LINE_CODING
                0x20 => {
                    self.ctl_state = ControlState::DataOut;
                    self.usbfs.out_ep(0).doeplen.write((1 << 19) | 7);
                    let ctl = self.usbfs.out_ep(0).doepctl.read();
                    self.usbfs.out_ep(0).doepctl.write(
                        (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                            | depctl::EPEN
                            | depctl::CNAK,
                    );
                }
                // GET_LINE_CODING
                0x21 => {
                    let line = self.line_coding;
                    self.ep0_start_in(&line);
                }
                // SET_CONTROL_LINE_STATE
                0x22 => {
                    self.ep0_send_status_in();
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

        // Control EP0 Slave Mode Invariant (Mandatory Invariant 3):
        // DIEPCTL EPEN & CNAK must be asserted BEFORE writing data to the FIFO!
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

        // Write data chunk to FIFO 0
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
        // Arm EP0 IN for 0-byte Status IN packet
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

    fn write_fifo(&self, ep: usize, data: &[u8]) {
        let len = data.len();
        let word_count = (len + 3) / 4;
        let fifo_ptr = self.usbfs.fifo(ep);

        for w in 0..word_count {
            let mut val = 0u32;
            for b in 0..4 {
                let idx = w * 4 + b;
                if idx < len {
                    val |= (data[idx] as u32) << (b * 8);
                }
            }
            unsafe { core::ptr::write_volatile(fifo_ptr, val) };
        }
    }

    /// Transmits a slice of bytes over CDC-ACM Bulk IN (EP1).
    /// Returns the number of bytes queued for transmission (up to 64 bytes).
    pub fn write(&mut self, buf: &[u8]) -> usize {
        if !self.configured || self.tx_busy || buf.is_empty() {
            return 0;
        }

        let len = core::cmp::min(buf.len(), 64);

        // Arm EP1 IN (Bulk IN): DIEPLEN must precede FIFO write
        self.usbfs
            .in_ep(1)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs.in_ep(1).dieplen.write((1 << 19) | (len as u32));
        let ctl = self.usbfs.in_ep(1).diepctl.read();
        self.usbfs.in_ep(1).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        // Write data to FIFO 1
        self.write_fifo(1, &buf[..len]);
        self.tx_busy = true;

        len
    }

    /// Reads received bytes from the CDC-ACM Bulk OUT ring buffer.
    /// Returns the number of bytes copied into `dest`.
    pub fn read(&mut self, dest: &mut [u8]) -> usize {
        let mut count = 0;
        while count < dest.len() && self.rx_head != self.rx_tail {
            dest[count] = self.rx_buf[self.rx_tail];
            self.rx_tail = (self.rx_tail + 1) % RX_BUF_SIZE;
            count += 1;
        }
        count
    }

    /// Checks if received bytes are waiting in the ring buffer.
    #[inline(always)]
    pub fn has_rx(&self) -> bool {
        self.rx_head != self.rx_tail
    }
}

impl fmt::Write for UsbCdcAcm {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let mut offset = 0;
        while offset < bytes.len() {
            // Must keep polling USB to process in-flight packets while waiting for TX ready
            self.poll();
            if self.configured && !self.tx_busy {
                let chunk_len = self.write(&bytes[offset..]);
                offset += chunk_len;
            }
        }
        Ok(())
    }
}

// ============================================================================
// USB HID Custom Device Driver (for INA219 Current Monitor / Custom Reports)
// ============================================================================

const HID_DEV_DESC: [u8; 18] = [
    18,         // bLength
    0x01,       // bDescriptorType = Device
    0x00, 0x02, // bcdUSB = 2.00
    0x00,       // bDeviceClass = 0 (Interface class)
    0x00,       // bDeviceSubClass = 0
    0x00,       // bDeviceProtocol = 0
    64,         // bMaxPacketSize0 = 64
    0xE9, 0x28, // idVendor = 0x28E9
    0x34, 0x12, // idProduct = 0x1234
    0x00, 0x01, // bcdDevice = 1.00
    1,          // iManufacturer = String 1
    2,          // iProduct = String 2
    3,          // iSerialNumber = String 3
    1,          // bNumConfigurations = 1
];

const HID_CONFIG_DESC: [u8; 41] = [
    // Configuration Descriptor (9 bytes)
    9,          // bLength
    0x02,       // bDescriptorType = Config
    41, 0,      // wTotalLength = 41
    1,          // bNumInterfaces = 1
    1,          // bConfigurationValue = 1
    0,          // iConfiguration = 0
    0x80,       // bmAttributes = Bus-powered
    0xFA,       // bMaxPower = 500mA (0xFA * 2mA)

    // Interface 0: Custom HID (9 bytes)
    9,          // bLength
    0x04,       // bDescriptorType = Interface
    0,          // bInterfaceNumber = 0
    0,          // bAlternateSetting = 0
    2,          // bNumEndpoints = 2
    0x03,       // bInterfaceClass = HID
    0x00,       // bInterfaceSubClass = 0
    0x00,       // bInterfaceProtocol = 0
    0,          // iInterface = 0

    // HID Descriptor (9 bytes)
    9,          // bLength
    0x21,       // bDescriptorType = HID
    0x11, 0x01, // bcdHID = 1.11
    0x00,       // bCountryCode = 0
    1,          // bNumDescriptors = 1
    0x22,       // bDescriptorType = Report
    23, 0,      // wDescriptorLength = 23

    // Endpoint 1 IN: Interrupt IN (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x81,       // bEndpointAddress = EP1 IN
    0x03,       // bmAttributes = Interrupt
    64, 0,      // wMaxPacketSize = 64
    1,          // bInterval = 1ms

    // Endpoint 1 OUT: Interrupt OUT (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x01,       // bEndpointAddress = EP1 OUT
    0x03,       // bmAttributes = Interrupt
    64, 0,      // wMaxPacketSize = 64
    1,          // bInterval = 1ms
];

const CUSTOM_HID_REPORT_DESC: [u8; 23] = [
    0x06, 0x00, 0xFF,  // Usage Page (Vendor-Defined 0xFF00)
    0x09, 0x01,        // Usage (Vendor-Defined 1)
    0xA1, 0x01,        // Collection (Application)
    0x85, 0x01,        //   Report ID (1)
    0x09, 0x03,        //   Usage (Vendor-Defined 3)
    0x15, 0x00,        //   Logical Minimum (0)
    0x26, 0xFF, 0x00,  //   Logical Maximum (255)
    0x75, 0x08,        //   Report Size (8 bits)
    0x95, 0x08,        //   Report Count (8 bytes)
    0x81, 0x02,        //   Input (Data, Var, Abs)
    0xC0,              // End Collection
];

const STR_HID_PRODUCT: [u8; 52] = [
    52, 0x03,
    b'L', 0, b'o', 0, b'n', 0, b'g', 0, b'a', 0, b'n', 0,
    b' ', 0, b'N', 0, b'a', 0, b'n', 0, b'o', 0,
    b' ', 0, b'P', 0, b'o', 0, b'w', 0, b'e', 0, b'r', 0,
    b' ', 0, b'M', 0, b'o', 0, b'n', 0, b'i', 0, b't', 0, b'o', 0, b'r', 0,
];

pub struct UsbHid {
    usbfs: Usbfs,
    ctl_state: ControlState,
    setup_pkt: SetupPacket,
    ep0_in_data: [u8; 128],
    ep0_in_len: usize,
    ep0_in_offset: usize,
    ep0_out_buf: [u8; 64],
    pending_addr: u8,
    configured: bool,
    tx_busy: bool,
    idle_rate: u8,
}

impl UsbHid {
    pub fn new(usbfs: Usbfs, rcu: &Rcu, delay: &mut Delay) -> Self {
        // Enable USBFS peripheral clock in RCU AHBEN
        rcu.regs().ahben.set_bits(ahben::USBFSEN);

        // Core Init: embedded PHY and Force Device Mode (FDM = 1 << 30)
        usbfs.global().gusbcs.set_bits(gusbcs::EMBPHY | gusbcs::FDM);

        // Reset core
        usbfs.global().grstctl.set_bits(grstctl::CSRST);
        let mut timeout = 100_000u32;
        while (usbfs.global().grstctl.read() & grstctl::CSRST) != 0 && timeout > 0 {
            timeout -= 1;
        }
        delay.delay_us(3);

        // Power on transceiver and VBUS detection ignore
        usbfs.global().gccfg.set_bits(
            gccfg::PWRON | gccfg::VBUSACEN | gccfg::VBUSBCEN | gccfg::SOFOEN | gccfg::VBUSIG,
        );

        // Delay 20 ms for PHY stabilization
        delay.delay_ms(20);

        // Soft disconnect before initializing device registers
        usbfs.device().dctl.set_bits(dctl::SDIS);
        delay.delay_ms(100);

        // Restart PHY clock
        usbfs.pwrclkctl().write(0);

        // Full speed device configuration & 80% EOF threshold
        usbfs.device().dcfg.modify(|val| {
            (val & !(dcfg::DS_MASK | dcfg::EOPFT_MASK)) | dcfg::DS_FULL | dcfg::EOPFT_80
        });

        // Allocate FIFOs: RX=128 words, TX0=32 words, TX1=64 words, TX2=16 words
        usbfs.global().grflen.write(128);
        usbfs.global().diep0tflen_hnptflen.write((32 << 16) | 128);
        usbfs.global().dieptflen[0].write((64 << 16) | (128 + 32));
        usbfs.global().dieptflen[1].write((16 << 16) | (128 + 32 + 64));

        // Inactive endpoints zeroed (Mandatory Invariant 4)
        for i in 0..4 {
            usbfs.in_ep(i).diepctl.write(0);
            usbfs.out_ep(i).doepctl.write(0);
        }

        // Flush FIFOs
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

        // Clear and mask interrupts
        usbfs.device().diepinten.write(0);
        usbfs.device().doepinten.write(0);
        usbfs.device().daepint.write(0xFFFF_FFFF);
        usbfs.device().daepinten.write(0);

        // Unmask core interrupts
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

        // Unmask global interrupt in AHB
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
            tx_busy: false,
            idle_rate: 0,
        };

        // Prepare EP0 endpoints
        instance.handle_reset();

        // Connect device (clear SDIS)
        instance.usbfs.device().dctl.clear_bits(dctl::SDIS);
        delay.delay_ms(3);

        instance
    }

    #[inline(always)]
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    #[inline(always)]
    pub fn is_tx_busy(&self) -> bool {
        self.tx_busy
    }

    #[inline(always)]
    fn arm_ep0_setup(&self) {
        self.usbfs.out_ep(0).doeplen.write((24 << 0) | (1 << 19) | (3 << 29));
    }

    pub fn poll(&mut self) {
        let intr = self.usbfs.global().gintf.read();

        // 1. Bus Reset
        if (intr & gintf::RST) != 0 {
            self.handle_reset();
            return;
        }

        // 2. Enumeration Finished
        if (intr & gintf::ENUMFIF) != 0 {
            self.usbfs.device().dctl.set_bits(dctl::CGINAK);
            // Set 48 MHz PHY turnaround time (UTT = 9)
            self.usbfs
                .global()
                .gusbcs
                .modify(|v| (v & !gusbcs::UTT_MASK) | (9 << 10));
            self.usbfs.global().gintf.write(gintf::ENUMFIF);
        }

        // 3. RX FIFO Non-Empty: drain all pending status queue packets
        while (self.usbfs.global().gintf.read() & gintf::RXFNEIF) != 0 {
            self.handle_rxfifo();
        }

        let intr = self.usbfs.global().gintf.read();

        // 4. OUT Endpoint Interrupt
        if (intr & gintf::OEPIF) != 0 {
            self.handle_epout();
        }

        // 5. IN Endpoint Interrupt
        if (intr & gintf::IEPIF) != 0 {
            self.handle_epin();
        }

        // 6. Suspend / Wakeup / SOF
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

        // Flush FIFOs
        self.usbfs.global().grstctl.write((0x10 << 6) | grstctl::TXFF);
        while (self.usbfs.global().grstctl.read() & grstctl::TXFF) != 0 {}
        self.usbfs.global().grstctl.write(grstctl::RXFF);
        while (self.usbfs.global().grstctl.read() & grstctl::RXFF) != 0 {}

        // Reset non-control endpoints
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

        // EP0 state reset (Invariant: never set SNAK on EP0)
        self.usbfs.in_ep(0).dieplen.write(0);
        self.usbfs.in_ep(0).diepintf.write(0xFF);
        self.usbfs.out_ep(0).doeplen.write(0);
        self.usbfs.out_ep(0).doepintf.write(0xFF);

        // Activate EP0 with DATA0 and EPACT (Mandatory Invariant)
        self.usbfs
            .in_ep(0)
            .diepctl
            .write(depctl::EPACT | depctl::SD0PID);
        self.usbfs
            .out_ep(0)
            .doepctl
            .write(depctl::EPACT | depctl::SD0PID);

        // Reset address & status
        self.pending_addr = 0;
        self.configured = false;
        self.tx_busy = false;
        self.ctl_state = ControlState::Idle;
        self.usbfs.device().dcfg.clear_bits(dcfg::DAR_MASK);

        // Enable EP0 interrupts in device controller
        self.usbfs.device().daepint.write(0xFFFF_FFFF);
        self.usbfs.device().daepinten.write(1 | (1 << 16));
        self.usbfs
            .device()
            .doepinten
            .write(doepinten::STPFEN | doepinten::TFEN);
        self.usbfs.device().diepinten.write(diepinten::TFEN);

        // Arm EP0 OUT for SETUP packet (Mandatory Invariant 2)
        self.arm_ep0_setup();

        self.usbfs.global().gintf.write(gintf::RST);
    }

    fn handle_rxfifo(&mut self) {
        // Pop status from GRSTATP with RXFNEIE temporarily masked
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
                        // Host completed Status OUT stage (Mandatory Invariant 2)
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

        // Check EP1 OUT (Custom HID OUT)
        if (daepint & (1 << (16 + 1))) != 0 {
            let oepintr = self.usbfs.out_ep(1).doepintf.read();
            if (oepintr & doepintf::TF) != 0 {
                self.usbfs.out_ep(1).doepintf.write(doepintf::TF);
                // Re-arm EP1 OUT for next packet
                self.usbfs.out_ep(1).doeplen.write((1 << 19) | 64);
                let ctl = self.usbfs.out_ep(1).doepctl.read();
                self.usbfs.out_ep(1).doepctl.write(
                    (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                        | depctl::EPEN
                        | depctl::CNAK,
                );
            }
            let residual = self.usbfs.out_ep(1).doepintf.read();
            if residual != 0 {
                self.usbfs.out_ep(1).doepintf.write(residual);
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
                        // Data stage done: arm EP0 OUT for 0-byte Status OUT from host
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

        // Check EP1 IN (Custom HID IN)
        if (daepint & (1 << 1)) != 0 {
            let iepintr = self.usbfs.in_ep(1).diepintf.read();
            if (iepintr & diepintf::TF) != 0 {
                self.usbfs.in_ep(1).diepintf.write(diepintf::TF);
                self.tx_busy = false;
            }
            let residual = self.usbfs.in_ep(1).diepintf.read();
            if residual != 0 {
                self.usbfs.in_ep(1).diepintf.write(residual);
            }
        }
    }

    fn handle_setup(&mut self) {
        self.usbfs.in_ep(0).diepctl.clear_bits(depctl::STALL);
        self.usbfs.out_ep(0).doepctl.clear_bits(depctl::STALL);

        let req = self.setup_pkt;
        let req_type = req.bm_request_type & 0x60;

        match req_type {
            // Standard Requests
            0x00 => match req.b_request {
                // GET_DESCRIPTOR
                0x06 => {
                    let desc_type = (req.w_value >> 8) as u8;
                    let desc_index = (req.w_value & 0xFF) as u8;

                    match desc_type {
                        // Device Descriptor
                        1 => {
                            let mut len = HID_DEV_DESC.len();
                            // Windows xHCI 8-Byte Initial Probe Invariant (Mandatory Invariant 5)
                            if req.w_length == 64 {
                                len = 8;
                            } else if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&HID_DEV_DESC[..len]);
                        }
                        // Configuration Descriptor
                        2 => {
                            let mut len = HID_CONFIG_DESC.len();
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&HID_CONFIG_DESC[..len]);
                        }
                        // String Descriptors
                        3 => {
                            if desc_index >= 4 {
                                self.stall_ep0();
                            } else {
                                let str_bytes: &[u8] = match desc_index {
                                    0 => &STR_LANG_ID,
                                    1 => &STR_MANUFACTURER,
                                    2 => &STR_HID_PRODUCT,
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
                        // HID Descriptor (0x21)
                        0x21 => {
                            let mut len = 9;
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&HID_CONFIG_DESC[18..18 + len]);
                        }
                        // Report Descriptor (0x22)
                        0x22 => {
                            let mut len = CUSTOM_HID_REPORT_DESC.len();
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&CUSTOM_HID_REPORT_DESC[..len]);
                        }
                        _ => {
                            self.stall_ep0();
                        }
                    }
                }
                // SET_ADDRESS
                0x05 => {
                    let addr = (req.w_value & 0x7F) as u8;
                    self.usbfs.device().dcfg.modify(|v| {
                        (v & !dcfg::DAR_MASK) | ((addr as u32) << 4)
                    });
                    self.ep0_send_status_in();
                }
                // SET_CONFIGURATION
                0x09 => {
                    let cfg = (req.w_value & 0xFF) as u8;
                    if cfg == 1 {
                        // Configure EP1 IN (Interrupt IN, max 64, FIFO 1)
                        self.usbfs.in_ep(1).diepctl.write(
                            (64 << 0)
                                | depctl::EPACT
                                | (3 << 18)
                                | (1 << 22)
                                | depctl::SD0PID
                                | depctl::SNAK,
                        );

                        // Configure EP1 OUT (Interrupt OUT, max 64)
                        self.usbfs.out_ep(1).doepctl.write(
                            (64 << 0) | depctl::EPACT | (3 << 18) | depctl::SD0PID | depctl::SNAK,
                        );
                        // Arm EP1 OUT to receive data
                        self.usbfs.out_ep(1).doeplen.write((1 << 19) | 64);
                        self.usbfs.out_ep(1).doepctl.modify(|v| {
                            (v & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                                | depctl::EPEN
                                | depctl::CNAK
                        });

                        // Enable endpoint interrupts in DAEPINTEN
                        self.usbfs
                            .device()
                            .daepinten
                            .write((1 << 0) | (1 << 1) | (1 << 16) | (1 << (16 + 1)));

                        self.configured = true;
                        self.tx_busy = false;
                    } else {
                        self.configured = false;
                    }
                    self.ep0_send_status_in();
                }
                // GET_CONFIGURATION
                0x08 => {
                    let val = if self.configured { 1u8 } else { 0u8 };
                    self.ep0_start_in(&[val]);
                }
                // GET_STATUS
                0x00 => {
                    self.ep0_start_in(&[0, 0]);
                }
                // CLEAR_FEATURE / SET_FEATURE
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

            // Class Requests (0x20: Class request to Device/Interface/Endpoint)
            0x20 => match req.b_request {
                // SET_IDLE (0x0A)
                0x0A => {
                    self.idle_rate = (req.w_value >> 8) as u8;
                    self.ep0_send_status_in();
                }
                // GET_IDLE (0x02)
                0x02 => {
                    let rate = self.idle_rate;
                    self.ep0_start_in(&[rate]);
                }
                // GET_REPORT (0x01)
                0x01 => {
                    let default_report = [0x01u8, 0, 0, 0, 0, 0, 0, 0, 0];
                    let mut len = default_report.len();
                    if (req.w_length as usize) < len {
                        len = req.w_length as usize;
                    }
                    self.ep0_start_in(&default_report[..len]);
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

    /// Sends a custom HID report on EP1 IN.
    /// Returns true if queued successfully, false if busy or not configured.
    pub fn send_report(&mut self, report: &[u8]) -> bool {
        if !self.configured || self.tx_busy {
            return false;
        }

        let len = core::cmp::min(report.len(), 64);

        // Pre-arm EP1 IN
        self.usbfs
            .in_ep(1)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs.in_ep(1).dieplen.write((1 << 19) | (len as u32));
        let ctl = self.usbfs.in_ep(1).diepctl.read();
        self.usbfs.in_ep(1).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        // Write data to FIFO 1
        self.write_fifo(1, &report[..len]);
        self.tx_busy = true;

        true
    }
}

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

// ============================================================================
// USB Mass Storage Class (MSC) - Bulk-Only Transport (BOT)
// ============================================================================

const MSC_DEV_DESC: [u8; 18] = [
    18,         // bLength
    0x01,       // bDescriptorType = Device
    0x00, 0x02, // bcdUSB = 2.00
    0x00,       // bDeviceClass = 0 (defined in interface)
    0x00,       // bDeviceSubClass = 0
    0x00,       // bDeviceProtocol = 0
    64,         // bMaxPacketSize0 = 64
    0xE9, 0x28, // idVendor = 0x28E9
    0xA3, 0xAB, // idProduct = 0xABA3
    0x00, 0x01, // bcdDevice = 1.00
    1,          // iManufacturer = 1
    2,          // iProduct = 2
    3,          // iSerialNumber = 3
    1,          // bNumConfigurations = 1
];

pub const MSC_CONFIG_DESC: [u8; 32] = [
    // Configuration Descriptor (9 bytes)
    9,          // bLength
    0x02,       // bDescriptorType = Config
    32, 0,      // wTotalLength = 32
    1,          // bNumInterfaces = 1
    1,          // bConfigurationValue = 1
    0,          // iConfiguration = 0
    0x80,       // bmAttributes = Bus-powered
    50,         // bMaxPower = 100mA (50 * 2mA)

    // Interface Descriptor (9 bytes)
    9,          // bLength
    0x04,       // bDescriptorType = Interface
    0,          // bInterfaceNumber = 0
    0,          // bAlternateSetting = 0
    2,          // bNumEndpoints = 2
    0x08,       // bInterfaceClass = Mass Storage (0x08)
    0x06,       // bInterfaceSubClass = SCSI Transparent (0x06)
    0x50,       // bInterfaceProtocol = Bulk-Only Transport (0x50)
    0,          // iInterface = 0

    // Endpoint 1 Bulk-OUT (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x01,       // bEndpointAddress = EP1 OUT
    0x02,       // bmAttributes = Bulk
    64, 0,      // wMaxPacketSize = 64
    0,          // bInterval = 0

    // Endpoint 1 Bulk-IN (7 bytes)
    7,          // bLength
    0x05,       // bDescriptorType = Endpoint
    0x81,       // bEndpointAddress = EP1 IN
    0x02,       // bmAttributes = Bulk
    64, 0,      // wMaxPacketSize = 64
    0,          // bInterval = 0
];

const STR_MSC_PRODUCT: [u8; 44] = [
    44, 0x03,
    b'L', 0, b'o', 0, b'n', 0, b'g', 0, b'a', 0, b'n', 0,
    b' ', 0, b'N', 0, b'a', 0, b'n', 0, b'o', 0,
    b' ', 0, b'S', 0, b'D', 0,
    b' ', 0, b'R', 0, b'e', 0, b'a', 0, b'd', 0, b'e', 0, b'r', 0,
];

const STR_MSC_SERIAL: [u8; 26] = [
    26, 0x03,
    b'L', 0, b'N', 0, b'M', 0, b'S', 0, b'C', 0,
    b'0', 0, b'0', 0, b'0', 0, b'0', 0, b'0', 0,
    b'A', 0, b'3', 0,
];

const MSC_INQUIRY_DATA: [u8; 36] = [
    0x00,                   // Direct access block device
    0x80,                   // Removable medium
    0x02,                   // ANSI SCSI-2
    0x02,                   // Response data format
    31,                     // Additional length (36 - 5)
    0x00, 0x00, 0x00,       // Reserved
    b'L', b'o', b'n', b'g', b'a', b'n', b' ', b' ',                                 // Vendor: 8 bytes
    b'L', b'o', b'n', b'g', b'a', b'n', b' ', b'N', b'a', b'n', b'o', b' ', b'S', b'D', b' ', b' ', // Product: 16 bytes
    b'1', b'.', b'0', b'0',                         // Revision: 4 bytes
];

pub trait MscBlockDevice {
    fn is_ready(&self) -> bool;
    fn capacity(&self) -> (u32, u32);
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) -> bool;
    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) -> bool;
}

#[derive(Copy, Clone, Default)]
pub struct MscStats {
    pub sectors_read: u32,
    pub sectors_written: u32,
    pub is_active: bool,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BbbState {
    Idle,
    DataOut,
    DataIn,
    SendData,
    StatusPending,
    SendCsw,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CswStatus {
    CmdPassed = 0x00,
    CmdFailed = 0x01,
    PhaseError = 0x02,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SenseKey {
    NoSense = 0x00,
    RecoveredError = 0x01,
    NotReady = 0x02,
    MediumError = 0x03,
    HardwareError = 0x04,
    IllegalRequest = 0x05,
    UnitAttention = 0x06,
    DataProtect = 0x07,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Asc {
    NoAsc = 0x00,
    UnrecoveredReadError = 0x11,
    WriteFault = 0x03,
    InvalidCdb = 0x20,
    AddressOutOfRange = 0x21,
    InvalidFieldInCommand = 0x24,
    WriteProtected = 0x27,
    MediumNotPresent = 0x3A,
}

#[derive(Copy, Clone, Default)]
pub struct BbbCbw {
    pub d_cbw_signature: u32,
    pub d_cbw_tag: u32,
    pub d_cbw_data_transfer_length: u32,
    pub bm_cbw_flags: u8,
    pub b_cbw_lun: u8,
    pub b_cbw_cb_length: u8,
    pub cbwcb: [u8; 16],
}

impl BbbCbw {
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 31 {
            return None;
        }
        let sig = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let tag = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let len = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let flags = buf[12];
        let lun = buf[13];
        let cb_len = buf[14];
        let mut cbwcb = [0u8; 16];
        cbwcb.copy_from_slice(&buf[15..31]);
        Some(Self {
            d_cbw_signature: sig,
            d_cbw_tag: tag,
            d_cbw_data_transfer_length: len,
            bm_cbw_flags: flags,
            b_cbw_lun: lun,
            b_cbw_cb_length: cb_len,
            cbwcb,
        })
    }
}

pub struct BbbCsw {
    pub d_csw_signature: u32,
    pub d_csw_tag: u32,
    pub d_csw_data_residue: u32,
    pub b_csw_status: u8,
}

impl BbbCsw {
    pub fn write_to_slice(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.d_csw_signature.to_le_bytes());
        buf[4..8].copy_from_slice(&self.d_csw_tag.to_le_bytes());
        buf[8..12].copy_from_slice(&self.d_csw_data_residue.to_le_bytes());
        buf[12] = self.b_csw_status;
    }
}

pub struct UsbMsc {
    usbfs: Usbfs,
    ctl_state: ControlState,
    setup_pkt: SetupPacket,
    ep0_in_data: [u8; 128],
    ep0_in_len: usize,
    ep0_in_offset: usize,
    ep0_out_buf: [u8; 64],
    pending_addr: u8,
    configured: bool,

    // BOT State
    state: BbbState,
    cbw_buf: [u8; 64],
    cbw_rx_len: usize,
    cbw: BbbCbw,
    csw_residue: u32,
    csw_status: CswStatus,
    csw_pending: bool,

    sense_key: SenseKey,
    sense_asc: Asc,

    lba: u32,
    remaining_bytes: u32,
    data_done_time: u64,
    need_read: bool,
    need_write: bool,

    media_buf: [u8; 512],
    media_rx_offset: usize,

    pub stats: MscStats,
}

impl UsbMsc {
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

        // Allocate FIFOs: Total 320 words (1280 bytes)
        // RX: 96 words (384 bytes)
        // TX0: 32 words (128 bytes)
        // TX1: 192 words (768 bytes)
        usbfs.global().grflen.write(96);
        usbfs.global().diep0tflen_hnptflen.write((32 << 16) | 96);
        usbfs.global().dieptflen[0].write((192 << 16) | (96 + 32));
        usbfs.global().dieptflen[1].write((0 << 16) | (96 + 32 + 192));
        usbfs.global().dieptflen[2].write((0 << 16) | (96 + 32 + 192));

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
            state: BbbState::Idle,
            cbw_buf: [0u8; 64],
            cbw_rx_len: 0,
            cbw: BbbCbw::default(),
            csw_residue: 0,
            csw_status: CswStatus::CmdPassed,
            csw_pending: false,
            sense_key: SenseKey::NoSense,
            sense_asc: Asc::NoAsc,
            lba: 0,
            remaining_bytes: 0,
            data_done_time: 0,
            need_read: false,
            need_write: false,
            media_buf: [0u8; 512],
            media_rx_offset: 0,
            stats: MscStats::default(),
        };

        instance.handle_reset();
        instance.usbfs.device().dctl.clear_bits(dctl::SDIS);
        delay.delay_ms(3);

        instance
    }

    #[inline(always)]
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    #[inline(always)]
    pub fn clear_active(&mut self) {
        self.stats.is_active = false;
    }

    pub fn set_sense(&mut self, key: SenseKey, asc: Asc) {
        self.sense_key = key;
        self.sense_asc = asc;
    }

    fn arm_ep0_setup(&mut self) {
        self.usbfs.out_ep(0).doeplen.write(
            (1 << 29) | (1 << 19) | 24, // STPCNT=1, PCNT=1, TLEN=24
        );
        let ctl = self.usbfs.out_ep(0).doepctl.read();
        self.usbfs.out_ep(0).doepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );
    }

    fn ep0_start_in(&mut self, data: &[u8]) {
        let len = data.len();
        self.ep0_in_len = len;
        self.ep0_in_offset = 0;
        self.ep0_in_data[..len].copy_from_slice(data);

        let chunk = core::cmp::min(len, 64);
        self.usbfs
            .in_ep(0)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs
            .in_ep(0)
            .dieplen
            .write((1 << 19) | (chunk as u32));
        let ctl = self.usbfs.in_ep(0).diepctl.read();
        self.usbfs.in_ep(0).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        self.write_fifo(0, &self.ep0_in_data[..chunk]);
        self.ep0_in_offset = chunk;

        if chunk < len {
            self.ctl_state = ControlState::DataIn;
        } else {
            self.ctl_state = ControlState::LastDataIn;
        }
    }

    fn ep0_send_next_packet(&mut self) {
        let remain = self.ep0_in_len - self.ep0_in_offset;
        let chunk = core::cmp::min(remain, 64);

        self.usbfs
            .in_ep(0)
            .diepintf
            .write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);
        self.usbfs
            .in_ep(0)
            .dieplen
            .write((1 << 19) | (chunk as u32));
        let ctl = self.usbfs.in_ep(0).diepctl.read();
        self.usbfs.in_ep(0).diepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );

        self.write_fifo(0, &self.ep0_in_data[self.ep0_in_offset..self.ep0_in_offset + chunk]);
        self.ep0_in_offset += chunk;

        if self.ep0_in_offset < self.ep0_in_len {
            self.ctl_state = ControlState::DataIn;
        } else {
            self.ctl_state = ControlState::LastDataIn;
        }
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
        self.usbfs.in_ep(0).diepctl.set_bits(depctl::STALL);
        self.usbfs.out_ep(0).doepctl.set_bits(depctl::STALL);
        self.ctl_state = ControlState::Stall;
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

    fn set_ep_snak(&mut self, ep_num: usize) {
        let mut ctl = self.usbfs.in_ep(ep_num).diepctl.read();
        ctl &= !(depctl::SD0PID | depctl::SD1PID | depctl::EPD | depctl::CNAK | depctl::SNAK);
        self.usbfs.in_ep(ep_num).diepctl.write(ctl | depctl::SNAK);
        let mut timeout = 1000u32;
        while (self.usbfs.in_ep(ep_num).diepctl.read() & depctl::NAKS) == 0 && timeout > 0 {
            timeout -= 1;
        }
    }

    fn ep_nak_arm(&mut self, ep_num: usize, len: u32) {
        let packet_count = (len + 63) / 64;
        let packet_count = if packet_count == 0 { 1 } else { packet_count };
        let target_dieplen = (packet_count << 19) | len;

        let live_ctl = self.usbfs.in_ep(ep_num).diepctl.read();
        let live_len = self.usbfs.in_ep(ep_num).dieplen.read();

        // Idempotent check
        if (live_ctl & depctl::EPEN) != 0 && (live_ctl & depctl::NAKS) != 0 && live_len == target_dieplen {
            return;
        }

        // Step 1: Assert SNAK and wait for NAKS
        let mut epctl = live_ctl;
        epctl &= !(depctl::SD0PID | depctl::SD1PID | depctl::EPD | depctl::CNAK | depctl::SNAK);
        self.usbfs.in_ep(ep_num).diepctl.write(epctl | depctl::SNAK);

        let mut timeout = 1000u32;
        while (self.usbfs.in_ep(ep_num).diepctl.read() & depctl::NAKS) == 0 && timeout > 0 {
            timeout -= 1;
        }

        // Step 2: Clear stale flags
        self.usbfs.in_ep(ep_num).diepintf.write(diepintf::TF | diepintf::EPDIS | diepintf::TXFUD);

        // Step 3: Program DIEPLEN
        self.usbfs.in_ep(ep_num).dieplen.write(target_dieplen);

        // Step 4: Assert EPEN with NAK confirmed
        epctl = self.usbfs.in_ep(ep_num).diepctl.read();
        epctl &= !(depctl::SD0PID | depctl::SD1PID | depctl::EPD | depctl::CNAK | depctl::SNAK);
        self.usbfs.in_ep(ep_num).diepctl.write(epctl | depctl::EPEN);
    }

    fn ep_send(&mut self, ep_num: usize, data: &[u8]) {
        let len = data.len() as u32;
        self.ep_nak_arm(ep_num, len);
        self.write_fifo(ep_num, data);

        let mut cnak_ctl = self.usbfs.in_ep(ep_num).diepctl.read();
        cnak_ctl &= !(depctl::SD0PID | depctl::SD1PID | depctl::EPD | depctl::SNAK);
        cnak_ctl |= depctl::EPEN | depctl::CNAK;
        self.usbfs.in_ep(ep_num).diepctl.write(cnak_ctl);
    }

    fn ep_send_media(&mut self, len: usize) {
        let len_u32 = len as u32;
        self.ep_nak_arm(1, len_u32);

        let fifo_ptr = self.usbfs.fifo(1);
        let word_count = (len + 3) / 4;
        for i in 0..word_count {
            let mut word = 0u32;
            for b in 0..4 {
                let idx = i * 4 + b;
                if idx < len {
                    word |= (self.media_buf[idx] as u32) << (b * 8);
                }
            }
            unsafe {
                core::ptr::write_volatile(fifo_ptr, word);
            }
        }

        let mut cnak_ctl = self.usbfs.in_ep(1).diepctl.read();
        cnak_ctl &= !(depctl::SD0PID | depctl::SD1PID | depctl::EPD | depctl::SNAK);
        cnak_ctl |= depctl::EPEN | depctl::CNAK;
        self.usbfs.in_ep(1).diepctl.write(cnak_ctl);
    }

    fn send_data(&mut self, data: &[u8]) {
        let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, data.len());
        self.csw_residue = self.cbw.d_cbw_data_transfer_length - (len as u32);
        self.remaining_bytes = 0;
        self.state = BbbState::SendData;

        self.ep_send(1, &data[..len]);
    }

    fn send_csw(&mut self, status: CswStatus) {
        let csw = BbbCsw {
            d_csw_signature: 0x53425355, // "USBS"
            d_csw_tag: self.cbw.d_cbw_tag,
            d_csw_data_residue: self.csw_residue,
            b_csw_status: status as u8,
        };
        let mut csw_buf = [0u8; 13];
        csw.write_to_slice(&mut csw_buf);
        self.state = BbbState::SendCsw;
        self.csw_pending = false;
        self.ep_send(1, &csw_buf);
    }

    fn abort_transfer(&mut self) {
        self.need_read = false;
        self.need_write = false;
        self.csw_residue = self.remaining_bytes;
        if self.cbw.d_cbw_data_transfer_length == 0 {
            self.send_csw(CswStatus::CmdFailed);
            return;
        }
        if (self.cbw.bm_cbw_flags & 0x80) == 0 {
            self.usbfs.out_ep(1).doepctl.set_bits(depctl::STALL);
        }
        self.usbfs.in_ep(1).diepctl.set_bits(depctl::STALL);
        self.csw_pending = true;
    }

    fn arm_ep1_out_cbw(&mut self) {
        self.cbw_rx_len = 0;
        self.usbfs.out_ep(1).doeplen.write((1 << 19) | 31);
        let ctl = self.usbfs.out_ep(1).doepctl.read();
        self.usbfs.out_ep(1).doepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );
    }

    fn arm_ep1_out(&mut self, len: u32) {
        let packet_count = (len + 63) / 64;
        let packet_count = if packet_count == 0 { 1 } else { packet_count };
        self.usbfs.out_ep(1).doeplen.write((packet_count << 19) | len);
        let ctl = self.usbfs.out_ep(1).doepctl.read();
        self.usbfs.out_ep(1).doepctl.write(
            (ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::SNAK))
                | depctl::EPEN
                | depctl::CNAK,
        );
    }

    fn decode_cbw<D: MscBlockDevice>(&mut self, disk: &mut D) {
        if self.cbw_rx_len != 31 {
            self.abort_transfer();
            return;
        }
        let Some(cbw) = BbbCbw::from_bytes(&self.cbw_buf[..31]) else {
            self.abort_transfer();
            return;
        };
        if cbw.d_cbw_signature != 0x43425355 || cbw.b_cbw_lun != 0 || cbw.b_cbw_cb_length < 1 || cbw.b_cbw_cb_length > 16 {
            self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
            self.abort_transfer();
            return;
        }

        self.cbw = cbw;
        self.remaining_bytes = cbw.d_cbw_data_transfer_length;
        self.csw_residue = cbw.d_cbw_data_transfer_length;
        self.csw_pending = false;
        self.data_done_time = 0;

        self.process_scsi(disk);
    }

    fn process_scsi<D: MscBlockDevice>(&mut self, disk: &mut D) {
        let opcode = self.cbw.cbwcb[0];
        match opcode {
            0x00 => { // TEST_UNIT_READY
                if self.cbw.d_cbw_data_transfer_length != 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer();
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer();
                    return;
                }
                self.csw_residue = 0;
                self.send_csw(CswStatus::CmdPassed);
            }

            0x03 => { // REQUEST_SENSE
                let mut data = [0u8; 18];
                data[0] = 0x70; // Current error
                data[2] = self.sense_key as u8;
                data[7] = 10; // Additional length = 18 - 8
                data[12] = self.sense_asc as u8;

                self.sense_key = SenseKey::NoSense;
                self.sense_asc = Asc::NoAsc;

                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 18);
                self.send_data(&data[..len]);
            }

            0x12 => { // INQUIRY
                if (self.cbw.cbwcb[1] & 0x01) != 0 {
                    // EVPD
                    let page = self.cbw.cbwcb[2];
                    match page {
                        0x00 => {
                            let data: [u8; 7] = [0x00, 0x00, 0x00, 0x03, 0x00, 0x80, 0x83];
                            let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 7);
                            self.send_data(&data[..len]);
                        }
                        0x80 => {
                            let mut data = [0u8; 16];
                            data[0] = 0x00;
                            data[1] = 0x80;
                            data[3] = 12;
                            data[4..16].copy_from_slice(b"LNMSC00000A3");
                            let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 16);
                            self.send_data(&data[..len]);
                        }
                        0x83 => {
                            let mut data = [0u8; 16];
                            data[0] = 0x00;
                            data[1] = 0x83;
                            data[3] = 12;
                            data[4] = 0x02; // ASCII
                            data[5] = 0x01; // T10 Vendor ID
                            data[7] = 8;
                            data[8..16].copy_from_slice(b"Longan01");
                            let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 16);
                            self.send_data(&data[..len]);
                        }
                        _ => {
                            self.set_sense(SenseKey::IllegalRequest, Asc::InvalidFieldInCommand);
                            self.abort_transfer();
                        }
                    }
                } else {
                    let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, MSC_INQUIRY_DATA.len());
                    self.send_data(&MSC_INQUIRY_DATA[..len]);
                }
            }

            0x1A => { // MODE_SENSE_6
                let data: [u8; 4] = [0x03, 0x00, 0x00, 0x00];
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 4);
                self.send_data(&data[..len]);
            }

            0x5A => { // MODE_SENSE_10
                let data: [u8; 8] = [0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 8);
                self.send_data(&data[..len]);
            }

            0x23 => { // READ_FORMAT_CAPACITIES
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer();
                    return;
                }
                let (blk_cnt, blk_sz) = disk.capacity();
                let mut data = [0u8; 12];
                data[3] = 0x08; // list length
                data[4..8].copy_from_slice(&blk_cnt.to_be_bytes());
                data[8] = 0x02; // Formatted media
                data[9..12].copy_from_slice(&blk_sz.to_be_bytes()[1..4]);
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 12);
                self.send_data(&data[..len]);
            }

            0x25 => { // READ_CAPACITY_10
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer();
                    return;
                }
                let (blk_cnt, blk_sz) = disk.capacity();
                let last_lba = if blk_cnt > 0 { blk_cnt - 1 } else { 0 };
                let mut data = [0u8; 8];
                data[0..4].copy_from_slice(&last_lba.to_be_bytes());
                data[4..8].copy_from_slice(&blk_sz.to_be_bytes());
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 8);
                self.send_data(&data[..len]);
            }

            0x9E => { // SERVICE_ACTION_IN_16
                if (self.cbw.cbwcb[1] & 0x1F) == 0x10 { // READ_CAPACITY_16
                    if !disk.is_ready() {
                        self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                        self.abort_transfer();
                        return;
                    }
                    let (blk_cnt, blk_sz) = disk.capacity();
                    let last_lba = if blk_cnt > 0 { (blk_cnt - 1) as u64 } else { 0 };
                    let mut data = [0u8; 32];
                    data[0..8].copy_from_slice(&last_lba.to_be_bytes());
                    data[8..12].copy_from_slice(&blk_sz.to_be_bytes());
                    let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 32);
                    self.send_data(&data[..len]);
                } else {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidFieldInCommand);
                    self.abort_transfer();
                }
            }

            0xA0 => { // REPORT_LUNS
                let mut data = [0u8; 16];
                data[3] = 0x08; // list length = 8
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 16);
                self.send_data(&data[..len]);
            }

            0xA2 => { // SECURITY_PROTOCOL_IN
                if self.cbw.cbwcb[1] == 0x00 {
                    let data = [0u8; 8];
                    let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 8);
                    self.send_data(&data[..len]);
                } else {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidFieldInCommand);
                    self.abort_transfer();
                }
            }

            0x1B | 0x1E | 0x2F | 0x35 => { // START_STOP_UNIT, PREVENT_ALLOW_REMOVAL, VERIFY_10, SYNCHRONIZE_CACHE_10
                self.csw_residue = 0;
                self.send_csw(CswStatus::CmdPassed);
            }

            0x08 => { // READ_6
                if (self.cbw.bm_cbw_flags & 0x80) == 0 || !disk.is_ready() {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer();
                    return;
                }
                let lba = (((self.cbw.cbwcb[1] & 0x1F) as u32) << 16)
                    | ((self.cbw.cbwcb[2] as u32) << 8)
                    | (self.cbw.cbwcb[3] as u32);
                let mut blocks = if self.cbw.cbwcb[4] == 0 { 256 } else { self.cbw.cbwcb[4] as u32 };
                if blocks == 0 && self.cbw.d_cbw_data_transfer_length > 0 {
                    blocks = self.cbw.d_cbw_data_transfer_length / 512;
                }
                let (blk_cnt, _) = disk.capacity();
                if lba + blocks > blk_cnt {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer();
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks * 512);
                self.lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataIn;

                let first_len = core::cmp::min(self.remaining_bytes, 512);
                self.ep_nak_arm(1, first_len);
                self.need_read = true;
            }

            0x28 => { // READ_10
                if (self.cbw.bm_cbw_flags & 0x80) == 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer();
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer();
                    return;
                }
                let lba = u32::from_be_bytes([
                    self.cbw.cbwcb[2],
                    self.cbw.cbwcb[3],
                    self.cbw.cbwcb[4],
                    self.cbw.cbwcb[5],
                ]);
                let mut blocks = u16::from_be_bytes([self.cbw.cbwcb[7], self.cbw.cbwcb[8]]) as u32;
                if blocks == 0 && self.cbw.d_cbw_data_transfer_length > 0 {
                    blocks = self.cbw.d_cbw_data_transfer_length / 512;
                }
                if blocks == 0 {
                    self.csw_residue = 0;
                    self.send_csw(CswStatus::CmdPassed);
                    return;
                }
                let (blk_cnt, _) = disk.capacity();
                if lba + blocks > blk_cnt {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer();
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks * 512);
                self.lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataIn;

                let first_len = core::cmp::min(self.remaining_bytes, 512);
                self.ep_nak_arm(1, first_len);
                self.need_read = true;
            }

            0x0A => { // WRITE_6
                if (self.cbw.bm_cbw_flags & 0x80) != 0 || !disk.is_ready() {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer();
                    return;
                }
                let lba = (((self.cbw.cbwcb[1] & 0x1F) as u32) << 16)
                    | ((self.cbw.cbwcb[2] as u32) << 8)
                    | (self.cbw.cbwcb[3] as u32);
                let mut blocks = if self.cbw.cbwcb[4] == 0 { 256 } else { self.cbw.cbwcb[4] as u32 };
                if blocks == 0 && self.cbw.d_cbw_data_transfer_length > 0 {
                    blocks = self.cbw.d_cbw_data_transfer_length / 512;
                }
                let (blk_cnt, _) = disk.capacity();
                if lba + blocks > blk_cnt {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer();
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks * 512);
                self.lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataOut;
                self.media_rx_offset = 0;
                let rx_len = core::cmp::min(self.remaining_bytes, 512);
                self.arm_ep1_out(rx_len);
            }

            0x2A => { // WRITE_10
                if (self.cbw.bm_cbw_flags & 0x80) != 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer();
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer();
                    return;
                }
                let lba = u32::from_be_bytes([
                    self.cbw.cbwcb[2],
                    self.cbw.cbwcb[3],
                    self.cbw.cbwcb[4],
                    self.cbw.cbwcb[5],
                ]);
                let mut blocks = u16::from_be_bytes([self.cbw.cbwcb[7], self.cbw.cbwcb[8]]) as u32;
                if blocks == 0 && self.cbw.d_cbw_data_transfer_length > 0 {
                    blocks = self.cbw.d_cbw_data_transfer_length / 512;
                }
                if blocks == 0 {
                    self.csw_residue = 0;
                    self.send_csw(CswStatus::CmdPassed);
                    return;
                }
                let (blk_cnt, _) = disk.capacity();
                if lba + blocks > blk_cnt {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer();
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks * 512);
                self.lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataOut;
                self.media_rx_offset = 0;
                let rx_len = core::cmp::min(self.remaining_bytes, 512);
                self.arm_ep1_out(rx_len);
            }

            _ => {
                self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                self.abort_transfer();
            }
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
        self.ctl_state = ControlState::Idle;
        self.state = BbbState::Idle;
        self.need_read = false;
        self.need_write = false;
        self.csw_pending = false;
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
                                } else if ep_num == 1 {
                                    match self.state {
                                        BbbState::Idle => {
                                            if byte_idx < self.cbw_buf.len() {
                                                self.cbw_buf[byte_idx] = byte;
                                            }
                                        }
                                        BbbState::DataOut => {
                                            let offset = self.media_rx_offset + byte_idx;
                                            if offset < self.media_buf.len() {
                                                self.media_buf[offset] = byte;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                byte_idx += 1;
                            }
                        }
                    }
                    if ep_num == 1 {
                        match self.state {
                            BbbState::Idle => {
                                self.cbw_rx_len = bcount;
                            }
                            BbbState::DataOut => {
                                self.media_rx_offset += bcount;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_epout<D: MscBlockDevice>(&mut self, disk: &mut D) {
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

        // Check EP1 OUT
        if (daepint & (1 << (16 + 1))) != 0 {
            let oepintr = self.usbfs.out_ep(1).doepintf.read();
            if (oepintr & doepintf::TF) != 0 {
                self.usbfs.out_ep(1).doepintf.write(doepintf::TF);
                match self.state {
                    BbbState::Idle => {
                        self.decode_cbw(disk);
                    }
                    BbbState::DataOut => {
                        if self.media_rx_offset >= 512 || self.media_rx_offset >= self.remaining_bytes as usize {
                            self.need_write = true;
                        }
                    }
                    _ => {}
                }
            }
            let residual = self.usbfs.out_ep(1).doepintf.read();
            if residual != 0 {
                self.usbfs.out_ep(1).doepintf.write(residual);
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

        // Check EP1 IN
        if (daepint & (1 << 1)) != 0 {
            let iepintr = self.usbfs.in_ep(1).diepintf.read();
            if (iepintr & diepintf::TF) != 0 {
                self.usbfs.in_ep(1).diepintf.write(diepintf::TF);
                match self.state {
                    BbbState::DataIn => {
                        if self.remaining_bytes > 0 {
                            let next_len = core::cmp::min(self.remaining_bytes, 512);
                            self.ep_nak_arm(1, next_len);
                            self.need_read = true;
                        } else {
                            self.set_ep_snak(1);
                            self.csw_status = CswStatus::CmdPassed;
                            self.data_done_time = unsafe { gd32vf103_pac::Mtime::steal().read_ticks() };
                            self.state = BbbState::StatusPending;
                        }
                    }
                    BbbState::SendData => {
                        self.set_ep_snak(1);
                        self.csw_status = CswStatus::CmdPassed;
                        self.data_done_time = unsafe { gd32vf103_pac::Mtime::steal().read_ticks() };
                        self.state = BbbState::StatusPending;
                    }
                    BbbState::SendCsw => {
                        self.set_ep_snak(1);
                        self.state = BbbState::Idle;
                        self.arm_ep1_out_cbw();
                    }
                    _ => {}
                }
            }
            let residual = self.usbfs.in_ep(1).diepintf.read();
            if residual != 0 {
                self.usbfs.in_ep(1).diepintf.write(residual);
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
                            let mut len = MSC_DEV_DESC.len();
                            if req.w_length == 64 {
                                len = 8;
                            } else if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&MSC_DEV_DESC[..len]);
                        }
                        2 => {
                            let mut len = MSC_CONFIG_DESC.len();
                            if (req.w_length as usize) < len {
                                len = req.w_length as usize;
                            }
                            self.ep0_start_in(&MSC_CONFIG_DESC[..len]);
                        }
                        3 => {
                            if desc_index >= 4 {
                                self.stall_ep0();
                            } else {
                                let str_bytes: &[u8] = match desc_index {
                                    0 => &STR_LANG_ID,
                                    1 => &STR_MANUFACTURER,
                                    2 => &STR_MSC_PRODUCT,
                                    3 => &STR_MSC_SERIAL,
                                    _ => unreachable!(),
                                };
                                let mut len = str_bytes.len();
                                if (req.w_length as usize) < len {
                                    len = req.w_length as usize;
                                }
                                self.ep0_start_in(&str_bytes[..len]);
                            }
                        }
                        _ => self.stall_ep0(),
                    }
                }
                0x05 => {
                    self.pending_addr = (req.w_value & 0x7F) as u8;
                    let addr = self.pending_addr;
                    self.usbfs.device().dcfg.modify(|v| {
                        (v & !dcfg::DAR_MASK) | ((addr as u32) << 4)
                    });
                    self.ep0_send_status_in();
                }
                0x09 => {
                    let cfg = (req.w_value & 0xFF) as u8;
                    if cfg == 1 {
                        // EP1 IN: Bulk IN (max 64, FIFO 1)
                        self.usbfs.in_ep(1).diepctl.write(
                            (64 << 0)
                                | depctl::EPACT
                                | (2 << 18) // Bulk
                                | (1 << 22) // FIFO 1
                                | depctl::SD0PID
                                | depctl::SNAK,
                        );

                        // EP1 OUT: Bulk OUT (max 64)
                        self.usbfs.out_ep(1).doepctl.write(
                            (64 << 0) | depctl::EPACT | (2 << 18) | depctl::SD0PID | depctl::SNAK,
                        );

                        self.arm_ep1_out_cbw();

                        self.usbfs.device().daepinten.write(
                            (1 << 0) | (1 << 1) | (1 << 16) | (1 << 17),
                        );

                        self.configured = true;
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
                0x01 => { // CLEAR_FEATURE
                    let ep_addr = req.w_index as u8;
                    let ep_num = (ep_addr & 0x0F) as usize;
                    if ep_num < 4 {
                        if (ep_addr & 0x80) != 0 {
                            let mut diepctl = self.usbfs.in_ep(ep_num).diepctl.read();
                            diepctl &= !(depctl::STALL | depctl::CNAK);
                            self.usbfs.in_ep(ep_num).diepctl.write(diepctl | depctl::SD0PID | depctl::SNAK);

                            if ep_num == 1 && self.csw_pending {
                                self.csw_pending = false;
                                self.send_csw(CswStatus::CmdFailed);
                            }
                        } else {
                            let mut doepctl = self.usbfs.out_ep(ep_num).doepctl.read();
                            doepctl &= !(depctl::STALL | depctl::SNAK);
                            self.usbfs.out_ep(ep_num).doepctl.write(doepctl | depctl::SD0PID | depctl::CNAK);

                            if ep_num == 1 {
                                self.arm_ep1_out_cbw();
                            }
                        }
                    }
                    self.ep0_send_status_in();
                }
                0x03 => { // SET_FEATURE
                    let ep_addr = req.w_index as u8;
                    let ep_num = (ep_addr & 0x0F) as usize;
                    if ep_num < 4 {
                        if (ep_addr & 0x80) != 0 {
                            self.usbfs.in_ep(ep_num).diepctl.set_bits(depctl::STALL);
                        } else {
                            self.usbfs.out_ep(ep_num).doepctl.set_bits(depctl::STALL);
                        }
                    }
                    self.ep0_send_status_in();
                }
                _ => self.stall_ep0(),
            },
            0x20 => match req.b_request {
                0xFE => { // REQ_GET_MAX_LUN
                    self.ep0_start_in(&[0]);
                }
                0xFF => { // REQ_BBB_RESET
                    self.state = BbbState::Idle;
                    self.need_read = false;
                    self.need_write = false;
                    self.csw_pending = false;

                    self.usbfs.in_ep(1).diepctl.clear_bits(depctl::STALL);
                    self.usbfs.out_ep(1).doepctl.clear_bits(depctl::STALL);

                    self.usbfs.global().grstctl.write((0x10 << 6) | grstctl::TXFF);
                    while (self.usbfs.global().grstctl.read() & grstctl::TXFF) != 0 {}
                    self.usbfs.global().grstctl.write(grstctl::RXFF);
                    while (self.usbfs.global().grstctl.read() & grstctl::RXFF) != 0 {}

                    let mut in_ctl = self.usbfs.in_ep(1).diepctl.read();
                    in_ctl &= !(depctl::EPEN | depctl::EPD | depctl::CNAK);
                    self.usbfs.in_ep(1).diepctl.write(in_ctl | depctl::SD0PID | depctl::SNAK);

                    let out_ctl = self.usbfs.out_ep(1).doepctl.read();
                    self.usbfs.out_ep(1).doepctl.write(out_ctl | depctl::SD0PID | depctl::SNAK);

                    self.arm_ep1_out_cbw();
                    self.ep0_send_status_in();
                }
                _ => self.stall_ep0(),
            },
            _ => self.stall_ep0(),
        }
    }

    fn handle_events<D: MscBlockDevice>(&mut self, disk: &mut D) {
        let intr = self.usbfs.global().gintf.read();

        if (intr & gintf::RST) != 0 {
            self.handle_reset();
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
            self.handle_epout(disk);
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

    pub fn poll<D: MscBlockDevice>(&mut self, disk: &mut D) {
        self.handle_events(disk);

        if self.need_read {
            self.need_read = false;
            let len = core::cmp::min(self.remaining_bytes, 512);
            if disk.read_sector(self.lba, &mut self.media_buf) {
                self.stats.sectors_read += 1;
                self.stats.is_active = true;
                self.lba += 1;
                self.remaining_bytes -= len;
                self.ep_send_media(len as usize);
            } else {
                self.set_sense(SenseKey::HardwareError, Asc::UnrecoveredReadError);
                self.abort_transfer();
            }
        } else if self.need_write {
            self.need_write = false;
            let len = core::cmp::min(self.remaining_bytes, 512);
            if disk.write_sector(self.lba, &self.media_buf) {
                self.stats.sectors_written += 1;
                self.stats.is_active = true;
                self.lba += 1;
                self.remaining_bytes -= len;
                self.media_rx_offset = 0;
                if self.remaining_bytes == 0 {
                    self.csw_residue = 0;
                    self.send_csw(CswStatus::CmdPassed);
                } else {
                    let rx_len = core::cmp::min(self.remaining_bytes, 512);
                    self.arm_ep1_out(rx_len);
                }
            } else {
                self.set_sense(SenseKey::HardwareError, Asc::WriteFault);
                self.abort_transfer();
            }
        } else if self.state == BbbState::StatusPending {
            let now = unsafe { gd32vf103_pac::Mtime::steal().read_ticks() };
            let guard_elapsed = self.data_done_time != 0 && now.wrapping_sub(self.data_done_time) >= 60_000;
            let dieplen = self.usbfs.in_ep(1).dieplen.read();
            let hw_done = (dieplen & 0x7FFFF) == 0;
            let ctl = self.usbfs.in_ep(1).diepctl.read();
            let ep_naking = (ctl & depctl::NAKS) != 0;

            if guard_elapsed && hw_done && ep_naking {
                self.send_csw(self.csw_status);
            }
        }
    }
}

