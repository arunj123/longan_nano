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
