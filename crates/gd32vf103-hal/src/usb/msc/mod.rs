pub mod bot;
pub mod cache;
pub mod scsi;

pub use bot::{validate_bot_case, BbbCbw, BbbCsw, BbbState, BotCase, BotValidation, CswStatus};
pub use cache::{CacheEntry, LbaCache, CACHE_SECTORS};
pub use scsi::{scsi_cmd, Asc, SenseKey, MSC_INQUIRY_DATA};

use embedded_hal::delay::DelayNs;
use gd32vf103_pac::rcu::{ahben, Rcu};
use gd32vf103_pac::usbfs::{
    dcfg, dctl, depctl, diepinten, diepintf, doepinten, doepintf, gahbcs, gccfg, gintf, ginten,
    grstat, grstctl, gusbcs, Usbfs,
};

use crate::delay::Delay;
use super::{ControlState, SetupPacket, STR_LANG_ID, STR_MANUFACTURER};

const MSC_DEV_DESC: [u8; 18] = [
    18,         // bLength
    0x01,       // bDescriptorType = Device
    0x00, 0x02, // bcdUSB = 2.00
    0x00,       // bDeviceClass = 0 (defined in interface)
    0x00,       // bDeviceSubClass = 0
    0x00,       // bDeviceProtocol = 0
    64,         // bMaxPacketSize0 = 64
    0xE9, 0x28, // idVendor = 0x28E9
    0xD4, 0xAB, // idProduct = 0xABD4
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
    b'E', 0, b'0', 0,
];

#[repr(C, align(4))]
#[derive(Copy, Clone)]
pub struct SectorBuf(pub [u8; 512]);

impl SectorBuf {
    pub const fn new() -> Self {
        Self([0u8; 512])
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8; 512] {
        &self.0
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u8; 512] {
        &mut self.0
    }

    #[inline(always)]
    pub fn as_words(&self) -> &[u32; 128] {
        unsafe { &*(self.0.as_ptr() as *const [u32; 128]) }
    }

    #[inline(always)]
    pub fn as_mut_words(&mut self) -> &mut [u32; 128] {
        unsafe { &mut *(self.0.as_mut_ptr() as *mut [u32; 128]) }
    }
}

impl Default for SectorBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl core::ops::Deref for SectorBuf {
    type Target = [u8; 512];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for SectorBuf {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait MscBlockDevice {
    fn is_ready(&self) -> bool;
    fn check_ready(&mut self) -> bool {
        self.is_ready()
    }
    fn capacity(&self) -> (u32, u32);
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) -> bool;
    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) -> bool;
    fn begin_multi_read(&mut self, _start_lba: u32) -> bool {
        false
    }
    fn end_multi_read(&mut self) {}
    fn is_streaming(&self) -> bool {
        false
    }
    fn begin_multi_write(&mut self, _start_lba: u32) -> bool {
        false
    }
    fn write_next_stream_sector(&mut self, _buf: &[u8; 512]) -> bool {
        false
    }
    fn end_multi_write(&mut self) {}
    fn is_writing_stream(&self) -> bool {
        false
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct MscStats {
    pub sectors_read: u32,
    pub sectors_written: u32,
    pub last_lba: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub is_active: bool,
}

impl MscStats {
    /// Returns the cache hit rate as tenths of a percent (e.g. 984 = 98.4%).
    /// Returns 0 if no cache accesses have occurred.
    pub fn cache_hit_rate_tenth_percent(&self) -> u16 {
        let total = self.cache_hits.saturating_add(self.cache_misses);
        if total == 0 {
            0
        } else {
            ((self.cache_hits as u64 * 1000) / total as u64) as u16
        }
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
    ep1_in_prearmed: bool,

    sense_key: SenseKey,
    sense_asc: Asc,

    lba: u32,
    remaining_bytes: u32,
    data_done_time: u64,
    need_read: bool,
    need_write: bool,

    pub cache: LbaCache,
    media_buf: [SectorBuf; 2],
    buf_idx: usize,
    has_read_ahead: bool,
    read_ahead_len: usize,
    media_rx_offset: usize,

    pub media_present: bool,
    pub unit_attention: bool,

    pub stats: MscStats,
}

impl UsbMsc {
    #[inline(always)]
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

        // 1. Force hard Single-Ended Zero (SE0) on D+/D- via GPIOA PA11 & PA12 for 200 ms
        rcu.regs().apb2en.set_bits(1 << 2); // GPIOA clock
        let gpioa = unsafe { gd32vf103_pac::gpio::GpioPort::steal(gd32vf103_pac::gpio::Port::A) };
        let regs = gpioa.regs();
        let old_ctl1 = regs.ctl1.read();
        // PA11 (bits 12..15), PA12 (bits 16..19): Output Push-Pull 50MHz (0x3 each -> 0x33 << 12)
        regs.ctl1.modify(|v| (v & !(0xFF << 12)) | (0x33 << 12));
        // Force PA11 and PA12 LOW
        regs.bc.write((1 << 11) | (1 << 12));
        delay.delay_ms(200);
        // Restore GPIOA CTL1
        regs.ctl1.write(old_ctl1);

        // 2. Soft disconnect before initializing device registers for 800 ms (total >= 1000 ms)
        usbfs.device().dctl.set_bits(dctl::SDIS);
        delay.delay_ms(800);

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
            ep1_in_prearmed: false,
            sense_key: SenseKey::NoSense,
            sense_asc: Asc::NoAsc,
            lba: 0,
            remaining_bytes: 0,
            data_done_time: 0,
            need_read: false,
            need_write: false,
            cache: LbaCache::new(),
            media_buf: [SectorBuf::new(), SectorBuf::new()],
            buf_idx: 0,
            has_read_ahead: false,
            read_ahead_len: 0,
            media_rx_offset: 0,
            media_present: false,
            unit_attention: false,
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
    pub fn is_idle(&self) -> bool {
        self.state == BbbState::Idle && !self.need_read && !self.need_write
    }

    #[inline(always)]
    pub fn is_media_present(&self) -> bool {
        self.media_present
    }

    pub fn set_media_present(&mut self, present: bool) {
        if !self.media_present && present {
            self.unit_attention = true;
            self.cache.clear();
        } else if self.media_present && !present {
            self.unit_attention = false;
            self.cache.clear();
        }
        self.media_present = present;
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

        if ep_num > 0 {
            let mut timeout = 100_000u32;
            while ((self.usbfs.in_ep(ep_num).dieptfstat.read() & 0xFFFF) < word_count as u32) && timeout > 0 {
                timeout -= 1;
            }
        }

        let chunks = data.chunks_exact(4);
        let rem = chunks.remainder();
        for chunk in chunks {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            unsafe {
                core::ptr::write_volatile(fifo_ptr, word);
            }
        }
        if !rem.is_empty() {
            let mut word = 0u32;
            for (b, &byte) in rem.iter().enumerate() {
                word |= (byte as u32) << (b * 8);
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
            if ep_num == 1 {
                self.ep1_in_prearmed = true;
            }
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

        if ep_num == 1 {
            self.ep1_in_prearmed = true;
        }
    }

    fn ep_send(&mut self, ep_num: usize, data: &[u8]) {
        let len = data.len() as u32;
        if ep_num == 1 && self.ep1_in_prearmed {
            self.ep1_in_prearmed = false;
        } else {
            self.ep_nak_arm(ep_num, len);
            if ep_num == 1 {
                self.ep1_in_prearmed = false;
            }
        }
        self.write_fifo(ep_num, data);

        let mut cnak_ctl = self.usbfs.in_ep(ep_num).diepctl.read();
        cnak_ctl &= !(depctl::SD0PID | depctl::SD1PID | depctl::EPD | depctl::SNAK);
        cnak_ctl |= depctl::EPEN | depctl::CNAK;
        self.usbfs.in_ep(ep_num).diepctl.write(cnak_ctl);
    }

    fn ep_send_media(&mut self, buf_idx: usize, len: usize) {
        if self.ep1_in_prearmed {
            self.ep1_in_prearmed = false;
        } else {
            self.ep_nak_arm(1, len as u32);
            self.ep1_in_prearmed = false;
        }

        let word_count = (len + 3) / 4;
        let mut timeout = 100_000u32;
        while ((self.usbfs.in_ep(1).dieptfstat.read() & 0xFFFF) < word_count as u32) && timeout > 0 {
            timeout -= 1;
        }

        // Fast 32-bit word transfer into DFIFO[1] (128 words for 512B)
        let fifo_ptr = self.usbfs.fifo(1);
        let full_words = len / 4;
        let words = self.media_buf[buf_idx].as_words();

        for i in 0..full_words {
            unsafe {
                core::ptr::write_volatile(fifo_ptr, words[i]);
            }
        }

        let rem = len % 4;
        if rem > 0 {
            let offset = full_words * 4;
            let mut last_word = 0u32;
            for b in 0..rem {
                last_word |= (self.media_buf[buf_idx][offset + b] as u32) << (b * 8);
            }
            unsafe {
                core::ptr::write_volatile(fifo_ptr, last_word);
            }
        }

        // Release NAK and serialize onto wire
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

    fn abort_transfer<D: MscBlockDevice>(&mut self, disk: &mut D) {
        disk.end_multi_read();
        disk.end_multi_write();
        self.ep1_in_prearmed = false;
        self.has_read_ahead = false;
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
            self.abort_transfer(disk);
            return;
        }
        let Some(cbw) = BbbCbw::from_bytes(&self.cbw_buf[..31]) else {
            self.abort_transfer(disk);
            return;
        };
        if !cbw.is_valid() || cbw.b_cbw_lun != 0 {
            self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
            self.abort_transfer(disk);
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
            scsi_cmd::TEST_UNIT_READY => {
                if self.cbw.d_cbw_data_transfer_length != 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer(disk);
                    return;
                }
                let ready = disk.check_ready();
                if !self.media_present && ready {
                    self.media_present = true;
                    self.unit_attention = true;
                    self.cache.clear();
                } else if self.media_present && !ready {
                    self.media_present = false;
                    self.unit_attention = false;
                    self.cache.clear();
                }

                if self.unit_attention {
                    self.unit_attention = false;
                    self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                    self.abort_transfer(disk);
                    return;
                }

                if !ready {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
                    return;
                }
                self.csw_residue = 0;
                self.send_csw(CswStatus::CmdPassed);
            }

            scsi_cmd::REQUEST_SENSE => {
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

            scsi_cmd::INQUIRY => {
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
                            data[4..16].copy_from_slice(b"LNMSC00000E0");
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
                            self.abort_transfer(disk);
                        }
                    }
                } else {
                    let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, MSC_INQUIRY_DATA.len());
                    self.send_data(&MSC_INQUIRY_DATA[..len]);
                }
            }

            scsi_cmd::MODE_SENSE_6 => {
                let data: [u8; 4] = [0x03, 0x00, 0x00, 0x00];
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 4);
                self.send_data(&data[..len]);
            }

            scsi_cmd::MODE_SENSE_10 => {
                let data: [u8; 8] = [0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 8);
                self.send_data(&data[..len]);
            }

            scsi_cmd::READ_FORMAT_CAPACITIES => {
                if self.unit_attention {
                    self.unit_attention = false;
                    self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                    self.abort_transfer(disk);
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
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

            scsi_cmd::READ_CAPACITY_10 => {
                if self.unit_attention {
                    self.unit_attention = false;
                    self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                    self.abort_transfer(disk);
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
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

            scsi_cmd::SERVICE_ACTION_IN_16 => {
                if (self.cbw.cbwcb[1] & 0x1F) == 0x10 { // READ_CAPACITY_16
                    if self.unit_attention {
                        self.unit_attention = false;
                        self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                        self.abort_transfer(disk);
                        return;
                    }
                    if !disk.is_ready() {
                        self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                        self.abort_transfer(disk);
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
                    self.abort_transfer(disk);
                }
            }

            0xA0 => { // REPORT_LUNS
                let mut data = [0u8; 16];
                data[3] = 0x08; // list length = 8
                let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 16);
                self.send_data(&data[..len]);
            }

            scsi_cmd::SECURITY_PROTOCOL_IN => {
                if self.cbw.cbwcb[1] == 0x00 {
                    let data = [0u8; 8];
                    let len = core::cmp::min(self.cbw.d_cbw_data_transfer_length as usize, 8);
                    self.send_data(&data[..len]);
                } else {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidFieldInCommand);
                    self.abort_transfer(disk);
                }
            }

            scsi_cmd::START_STOP_UNIT | scsi_cmd::PREVENT_ALLOW_MEDIUM_REMOVAL | 0x2F => { // START_STOP_UNIT, PREVENT_ALLOW_REMOVAL, VERIFY_10
                self.csw_residue = 0;
                self.send_csw(CswStatus::CmdPassed);
            }

            scsi_cmd::SYNCHRONIZE_CACHE_10 => {
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
                    return;
                }
                self.csw_residue = 0;
                self.send_csw(CswStatus::CmdPassed);
            }

            scsi_cmd::READ_6 => {
                if (self.cbw.bm_cbw_flags & 0x80) == 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer(disk);
                    return;
                }
                if self.unit_attention {
                    self.unit_attention = false;
                    self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                    self.abort_transfer(disk);
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
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
                if lba.checked_add(blocks).map_or(true, |end| end > blk_cnt) {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer(disk);
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks.saturating_mul(512));
                self.lba = lba;
                self.stats.last_lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataIn;
                self.buf_idx = 0;
                self.has_read_ahead = false;

                // Pre-arm EP1 IN with EPEN=1, confirmed NAKS=1, and DIEPLEN=first_len
                let first_len = core::cmp::min(to_transfer, 512);
                self.ep_nak_arm(1, first_len);

                self.need_read = true;
            }

            scsi_cmd::READ_10 => {
                if (self.cbw.bm_cbw_flags & 0x80) == 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer(disk);
                    return;
                }
                if self.unit_attention {
                    self.unit_attention = false;
                    self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                    self.abort_transfer(disk);
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
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
                if lba.checked_add(blocks).map_or(true, |end| end > blk_cnt) {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer(disk);
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks.saturating_mul(512));
                self.lba = lba;
                self.stats.last_lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataIn;
                self.buf_idx = 0;
                self.has_read_ahead = false;

                // Pre-arm EP1 IN with EPEN=1, confirmed NAKS=1, and DIEPLEN=first_len
                let first_len = core::cmp::min(to_transfer, 512);
                self.ep_nak_arm(1, first_len);

                self.need_read = true;
            }

            scsi_cmd::WRITE_6 => {
                if (self.cbw.bm_cbw_flags & 0x80) != 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer(disk);
                    return;
                }
                if self.unit_attention {
                    self.unit_attention = false;
                    self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                    self.abort_transfer(disk);
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
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
                if lba.checked_add(blocks).map_or(true, |end| end > blk_cnt) {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer(disk);
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks.saturating_mul(512));
                self.lba = lba;
                self.stats.last_lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataOut;
                self.buf_idx = 0;
                self.media_rx_offset = 0;
                let rx_len = core::cmp::min(self.remaining_bytes, 512);
                self.arm_ep1_out(rx_len);
            }

            scsi_cmd::WRITE_10 => {
                if (self.cbw.bm_cbw_flags & 0x80) != 0 {
                    self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                    self.abort_transfer(disk);
                    return;
                }
                if self.unit_attention {
                    self.unit_attention = false;
                    self.set_sense(SenseKey::UnitAttention, Asc::NotReadyToReadyTransition);
                    self.abort_transfer(disk);
                    return;
                }
                if !disk.is_ready() {
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                    self.abort_transfer(disk);
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
                if lba.checked_add(blocks).map_or(true, |end| end > blk_cnt) {
                    self.set_sense(SenseKey::IllegalRequest, Asc::AddressOutOfRange);
                    self.abort_transfer(disk);
                    return;
                }
                let to_transfer = core::cmp::min(self.cbw.d_cbw_data_transfer_length, blocks.saturating_mul(512));
                self.lba = lba;
                self.stats.last_lba = lba;
                self.remaining_bytes = to_transfer;
                self.csw_residue = self.cbw.d_cbw_data_transfer_length - to_transfer;
                self.state = BbbState::DataOut;
                self.buf_idx = 0;
                self.media_rx_offset = 0;
                let rx_len = core::cmp::min(self.remaining_bytes, 512);
                self.arm_ep1_out(rx_len);
            }

            _ => {
                self.set_sense(SenseKey::IllegalRequest, Asc::InvalidCdb);
                self.abort_transfer(disk);
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
        self.ep1_in_prearmed = false;
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
                    if ep_num == 1 && self.state == BbbState::DataOut {
                        // High-throughput 32-bit word transfer for Bulk-OUT data phase
                        let offset = self.media_rx_offset;
                        let rx_idx = self.buf_idx;
                        if (offset & 3) == 0 && offset + bcount <= 512 {
                            let word_offset = offset / 4;
                            let full_words = bcount / 4;
                            let rem = bcount % 4;

                            let words_mut = self.media_buf[rx_idx].as_mut_words();
                            for i in 0..full_words {
                                let w = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                                words_mut[word_offset + i] = w;
                            }
                            if rem > 0 {
                                let w = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                                let byte_base = offset + full_words * 4;
                                for b in 0..rem {
                                    self.media_buf[rx_idx][byte_base + b] = ((w >> (b * 8)) & 0xFF) as u8;
                                }
                            }
                        } else {
                            let mut byte_idx = 0;
                            for _ in 0..word_count {
                                let w = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                                for b in 0..4 {
                                    if byte_idx < bcount {
                                        let off = self.media_rx_offset + byte_idx;
                                        if off < self.media_buf[rx_idx].len() {
                                            self.media_buf[rx_idx][off] = ((w >> (b * 8)) & 0xFF) as u8;
                                        }
                                        byte_idx += 1;
                                    }
                                }
                            }
                        }
                        self.media_rx_offset += bcount;
                    } else if ep_num == 1 && self.state == BbbState::Idle {
                        let mut byte_idx = 0;
                        for _ in 0..word_count {
                            let w = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                            for b in 0..4 {
                                if byte_idx < bcount {
                                    if byte_idx < self.cbw_buf.len() {
                                        self.cbw_buf[byte_idx] = ((w >> (b * 8)) & 0xFF) as u8;
                                    }
                                    byte_idx += 1;
                                }
                            }
                        }
                        self.cbw_rx_len = bcount;
                    } else if ep_num == 0 {
                        let mut byte_idx = 0;
                        for _ in 0..word_count {
                            let w = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
                            for b in 0..4 {
                                if byte_idx < bcount {
                                    if byte_idx < self.ep0_out_buf.len() {
                                        self.ep0_out_buf[byte_idx] = ((w >> (b * 8)) & 0xFF) as u8;
                                    }
                                    byte_idx += 1;
                                }
                            }
                        }
                    } else {
                        for _ in 0..word_count {
                            let _ = unsafe { core::ptr::read_volatile(self.usbfs.fifo(0)) };
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
                self.handle_setup(disk);
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

    fn handle_epin<D: MscBlockDevice>(&mut self, disk: &mut D) {
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
                        if self.has_read_ahead {
                            // Wire transmission of previous sector finished!
                            // Immediately load the read-ahead sector from SRAM into TX1 FIFO (~12 µs)
                            let cur_idx = self.buf_idx;
                            let cur_len = self.read_ahead_len;
                            self.has_read_ahead = false;
                            self.ep_send_media(cur_idx, cur_len);

                            // Pipelined read-ahead: If more sectors remain, overlap wire transmission with next read
                            if self.remaining_bytes > 0 {
                                let next_idx = 1 - cur_idx;
                                let next_len = core::cmp::min(self.remaining_bytes, 512) as usize;
                                let lba = self.lba;

                                let read_ok = if let Some(cached) = self.cache.lookup(lba) {
                                    self.stats.cache_hits += 1;
                                    disk.end_multi_read();
                                    self.media_buf[next_idx].copy_from_slice(cached);
                                    true
                                } else {
                                    self.stats.cache_misses += 1;
                                    if !disk.is_streaming() && self.remaining_bytes > 512 {
                                        disk.begin_multi_read(lba);
                                    }
                                    disk.read_sector(lba, &mut self.media_buf[next_idx])
                                };

                                if read_ok {
                                    self.stats.sectors_read += 1;
                                    self.stats.last_lba = lba;
                                    self.stats.is_active = true;
                                    self.lba += 1;
                                    self.remaining_bytes -= next_len as u32;
                                    self.buf_idx = next_idx;
                                    self.read_ahead_len = next_len;
                                    self.has_read_ahead = true;
                                } else {
                                    if !disk.check_ready() {
                                        self.media_present = false;
                                        self.unit_attention = false;
                                        self.cache.clear();
                                        self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                                    } else {
                                        self.set_sense(SenseKey::HardwareError, Asc::UnrecoveredReadError);
                                    }
                                    self.abort_transfer(disk);
                                }
                            } else {
                                disk.end_multi_read();
                            }
                        } else if self.remaining_bytes > 0 {
                            self.set_ep_snak(1);
                            self.need_read = true;
                        } else {
                            disk.end_multi_read();
                            self.set_ep_snak(1);
                            self.csw_status = CswStatus::CmdPassed;
                            self.data_done_time = unsafe { gd32vf103_pac::Mtime::steal().read_ticks() };
                            self.state = BbbState::StatusPending;
                        }
                    }
                    BbbState::SendData => {
                        let sent_len = (self.cbw.d_cbw_data_transfer_length - self.csw_residue) as usize;
                        if self.csw_residue > 0 && (sent_len % 64) == 0 {
                            self.state = BbbState::SendZlp;
                            self.ep_send(1, &[]);
                        } else {
                            self.set_ep_snak(1);
                            self.csw_status = CswStatus::CmdPassed;
                            self.data_done_time = unsafe { gd32vf103_pac::Mtime::steal().read_ticks() };
                            self.state = BbbState::StatusPending;
                        }
                    }
                    BbbState::SendZlp => {
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

    fn handle_setup<D: MscBlockDevice>(&mut self, disk: &mut D) {
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
                    disk.end_multi_read();
                    disk.end_multi_write();
                    self.state = BbbState::Idle;
                    self.need_read = false;
                    self.need_write = false;
                    self.has_read_ahead = false;
                    self.csw_pending = false;
                    self.ep1_in_prearmed = false;

                    self.usbfs.in_ep(1).diepctl.clear_bits(depctl::STALL);
                    self.usbfs.out_ep(1).doepctl.clear_bits(depctl::STALL);

                    let in_ctl = self.usbfs.in_ep(1).diepctl.read();
                    if (in_ctl & depctl::EPEN) != 0 {
                        self.usbfs.in_ep(1).diepctl.write(
                            (in_ctl & !(depctl::SD0PID | depctl::SD1PID | depctl::CNAK))
                                | depctl::EPD
                                | depctl::SNAK,
                        );
                        let mut timeout = 1000u32;
                        while (self.usbfs.in_ep(1).diepctl.read() & depctl::EPEN) != 0 && timeout > 0 {
                            timeout -= 1;
                        }
                    }

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
            disk.end_multi_read();
            disk.end_multi_write();
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
            self.handle_epin(disk);
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
            let len = core::cmp::min(self.remaining_bytes, 512) as usize;
            let lba = self.lba;

            let read_ok = if let Some(cached) = self.cache.lookup(lba) {
                self.stats.cache_hits += 1;
                disk.end_multi_read();
                self.media_buf[0].copy_from_slice(cached);
                true
            } else {
                self.stats.cache_misses += 1;
                if !disk.is_streaming() && self.remaining_bytes > 512 {
                    disk.begin_multi_read(lba);
                }
                disk.read_sector(lba, &mut self.media_buf[0])
            };

            if read_ok {
                self.stats.sectors_read += 1;
                self.stats.last_lba = lba;
                self.stats.is_active = true;
                self.lba += 1;
                self.remaining_bytes -= len as u32;

                // Load Sector 0 into TX1 FIFO and assert EPEN | CNAK
                self.ep_send_media(0, len);

                // Pipelined read-ahead: If more sectors remain, read next sector into buffer 1 right now!
                if self.remaining_bytes > 0 {
                    let next_len = core::cmp::min(self.remaining_bytes, 512) as usize;
                    let next_lba = self.lba;

                    let next_ok = if let Some(cached) = self.cache.lookup(next_lba) {
                        self.stats.cache_hits += 1;
                        disk.end_multi_read();
                        self.media_buf[1].copy_from_slice(cached);
                        true
                    } else {
                        self.stats.cache_misses += 1;
                        if !disk.is_streaming() && self.remaining_bytes > 512 {
                            disk.begin_multi_read(next_lba);
                        }
                        disk.read_sector(next_lba, &mut self.media_buf[1])
                    };

                    if next_ok {
                        self.stats.sectors_read += 1;
                        self.stats.last_lba = next_lba;
                        self.stats.is_active = true;
                        self.lba += 1;
                        self.remaining_bytes -= next_len as u32;
                        self.buf_idx = 1;
                        self.read_ahead_len = next_len;
                        self.has_read_ahead = true;
                    } else {
                        if !disk.check_ready() {
                            self.media_present = false;
                            self.unit_attention = false;
                            self.cache.clear();
                            self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                        } else {
                            self.set_sense(SenseKey::HardwareError, Asc::UnrecoveredReadError);
                        }
                        self.abort_transfer(disk);
                    }
                } else {
                    disk.end_multi_read();
                }
            } else {
                if !disk.check_ready() {
                    self.media_present = false;
                    self.unit_attention = false;
                    self.cache.clear();
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                } else {
                    self.set_sense(SenseKey::HardwareError, Asc::UnrecoveredReadError);
                }
                self.abort_transfer(disk);
            }
        } else if self.need_write {
            self.need_write = false;
            let len = core::cmp::min(self.remaining_bytes, 512);
            let cur_idx = self.buf_idx;

            // Double-buffered write pipelining:
            // If more sectors remain to be received from the host, switch RX buffer
            // to the alternate slot (1 - cur_idx) and arm EP1 OUT immediately so USB reception
            // of sector N+1 overlaps concurrently with SPI flash programming of sector N.
            let has_more = self.remaining_bytes > len;
            if has_more {
                let next_idx = 1 - cur_idx;
                self.buf_idx = next_idx;
                self.media_rx_offset = 0;
                let next_rx_len = core::cmp::min(self.remaining_bytes - len, 512);
                self.arm_ep1_out(next_rx_len);
            }

            // Stream multi-block writes via CMD25 when multiple sectors are requested
            let write_ok = if has_more {
                if !disk.is_writing_stream() {
                    disk.begin_multi_write(self.lba);
                }
                disk.write_sector(self.lba, &self.media_buf[cur_idx])
            } else {
                disk.write_sector(self.lba, &self.media_buf[cur_idx])
            };

            if write_ok {
                self.cache.update(self.lba, &self.media_buf[cur_idx]);
                self.stats.sectors_written += 1;
                self.stats.last_lba = self.lba;
                self.stats.is_active = true;
                self.lba += 1;
                self.remaining_bytes -= len;
                if self.remaining_bytes == 0 {
                    disk.end_multi_write();
                    self.media_rx_offset = 0;
                    self.csw_residue = 0;
                    self.send_csw(CswStatus::CmdPassed);
                }
            } else {
                disk.end_multi_write();
                self.cache.invalidate(self.lba);
                if !disk.check_ready() {
                    self.media_present = false;
                    self.unit_attention = false;
                    self.cache.clear();
                    self.set_sense(SenseKey::NotReady, Asc::MediumNotPresent);
                } else {
                    self.set_sense(SenseKey::HardwareError, Asc::WriteFault);
                }
                self.abort_transfer(disk);
            }
        } else if self.state == BbbState::StatusPending {
            let now = unsafe { gd32vf103_pac::Mtime::steal().read_ticks() };
            let elapsed = if self.data_done_time != 0 { now.wrapping_sub(self.data_done_time) } else { 0 };
            let dieplen = self.usbfs.in_ep(1).dieplen.read();
            let hw_done = (dieplen & 0x7FFFF) == 0 && ((dieplen >> 19) & 0x7FF) == 0;
            let fifo_empty = (self.usbfs.in_ep(1).dieptfstat.read() & 0xFFFF) >= 192;
            let ctl = self.usbfs.in_ep(1).diepctl.read();
            let ep_naking = (ctl & depctl::NAKS) != 0;

            // Task 3.2: Eliminate artificial 1,200-tick (50 µs) spin wait before sending CSW.
            // As soon as the previous data transfer has completed in hardware (hw_done),
            // the TX1 FIFO has completely emptied (fifo_empty), and Endpoint 1 is confirmed
            // responding NAK to host IN tokens (ep_naking), dispatch CSW immediately without delay.
            // Retain 2.0s watchdog (48,000,000 ticks @ 24 MHz) to recover from any bus stalls.
            if (hw_done && fifo_empty && ep_naking) || (elapsed >= 48_000_000) {
                self.send_csw(self.csw_status);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_buf_alignment_and_size() {
        assert_eq!(core::mem::align_of::<SectorBuf>(), 4);
        assert_eq!(core::mem::size_of::<SectorBuf>(), 512);
    }

    #[test]
    fn test_sector_buf_word_access() {
        let mut buf = SectorBuf::new();
        // Modify via words
        {
            let words = buf.as_mut_words();
            assert_eq!(words.len(), 128);
            words[0] = 0x12345678;
            words[127] = 0xDEADBEEF;
        }

        // Verify little-endian byte ordering matches wire format
        assert_eq!(buf[0], 0x78);
        assert_eq!(buf[1], 0x56);
        assert_eq!(buf[2], 0x34);
        assert_eq!(buf[3], 0x12);

        assert_eq!(buf[508], 0xEF);
        assert_eq!(buf[509], 0xBE);
        assert_eq!(buf[510], 0xAD);
        assert_eq!(buf[511], 0xDE);

        // Read back via words
        assert_eq!(buf.as_words()[0], 0x12345678);
        assert_eq!(buf.as_words()[127], 0xDEADBEEF);
    }

    #[test]
    fn test_sector_buf_deref_slice() {
        let mut buf = SectorBuf::new();
        let test_data = [0xA5u8; 512];
        buf.copy_from_slice(&test_data);
        assert_eq!(buf[0], 0xA5);
        assert_eq!(buf[511], 0xA5);
        assert_eq!(buf.as_words()[0], 0xA5A5A5A5);
        assert_eq!(buf.as_words()[127], 0xA5A5A5A5);
    }

    struct MockBlockDevice {
        streaming_write: bool,
        write_sectors: u32,
    }

    impl MscBlockDevice for MockBlockDevice {
        fn is_ready(&self) -> bool { true }
        fn capacity(&self) -> (u32, u32) { (1024, 512) }
        fn read_sector(&mut self, _lba: u32, _buf: &mut [u8; 512]) -> bool { true }
        fn write_sector(&mut self, _lba: u32, _buf: &[u8; 512]) -> bool {
            self.write_sectors += 1;
            true
        }
        fn begin_multi_write(&mut self, _start_lba: u32) -> bool {
            self.streaming_write = true;
            true
        }
        fn write_next_stream_sector(&mut self, _buf: &[u8; 512]) -> bool {
            self.write_sectors += 1;
            true
        }
        fn end_multi_write(&mut self) {
            self.streaming_write = false;
        }
        fn is_writing_stream(&self) -> bool {
            self.streaming_write
        }
    }

    #[test]
    fn test_msc_block_device_write_streaming_mock() {
        let mut disk = MockBlockDevice {
            streaming_write: false,
            write_sectors: 0,
        };

        assert!(!disk.is_writing_stream());
        assert!(disk.begin_multi_write(100));
        assert!(disk.is_writing_stream());

        let dummy_buf = [0x55u8; 512];
        assert!(disk.write_next_stream_sector(&dummy_buf));
        assert!(disk.write_next_stream_sector(&dummy_buf));
        assert_eq!(disk.write_sectors, 2);

        disk.end_multi_write();
        assert!(!disk.is_writing_stream());
    }

    #[test]
    fn test_double_buffer_ping_pong_indices() {
        let mut rx_idx = 0usize;
        let mut committed = [0usize; 4];

        for i in 0..4 {
            let cur_commit = rx_idx;
            committed[i] = cur_commit;
            let has_more = i < 3;
            if has_more {
                rx_idx = 1 - cur_idx(rx_idx);
            }
        }

        fn cur_idx(idx: usize) -> usize { idx }

        assert_eq!(committed, [0, 1, 0, 1]);
        assert_eq!(rx_idx, 1);
    }

    #[test]
    fn test_msc_stats_default_and_updates() {
        let mut stats = MscStats::default();
        assert_eq!(stats.sectors_read, 0);
        assert_eq!(stats.sectors_written, 0);
        assert_eq!(stats.last_lba, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert!(!stats.is_active);

        stats.sectors_read += 16;
        stats.last_lba = 1024;
        stats.cache_hits += 12;
        stats.cache_misses += 4;
        stats.is_active = true;

        assert_eq!(stats.sectors_read, 16);
        assert_eq!(stats.last_lba, 1024);
        assert_eq!(stats.cache_hits, 12);
        assert_eq!(stats.cache_misses, 4);
        assert!(stats.is_active);
    }

    #[test]
    fn test_msc_stats_cache_hit_rate() {
        let mut stats = MscStats::default();
        // Zero total accesses
        assert_eq!(stats.cache_hit_rate_tenth_percent(), 0);

        // 100% hits
        stats.cache_hits = 16;
        stats.cache_misses = 0;
        assert_eq!(stats.cache_hit_rate_tenth_percent(), 1000); // 100.0%

        // 50% hits
        stats.cache_hits = 10;
        stats.cache_misses = 10;
        assert_eq!(stats.cache_hit_rate_tenth_percent(), 500); // 50.0%

        // 98.4% hits
        stats.cache_hits = 984;
        stats.cache_misses = 16;
        assert_eq!(stats.cache_hit_rate_tenth_percent(), 984); // 98.4%
    }

    #[test]
    fn test_lba_cache_clear() {
        let mut cache = LbaCache::new();
        let buf = [0x77u8; 512];
        cache.insert(0, 100, &buf);
        cache.insert(1, 200, &buf);
        assert!(cache.lookup(100).is_some());
        assert!(cache.lookup(200).is_some());

        cache.clear();
        assert!(cache.lookup(100).is_none());
        assert!(cache.lookup(200).is_none());
        for entry in cache.entries.iter() {
            assert!(!entry.valid);
            assert_eq!(entry.lba, 0xFFFF_FFFF);
        }
    }

    #[test]
    fn test_set_media_present_transitions() {
        // Mock minimal UsbMsc fields for media transitions
        let mut media_present = false;
        let mut unit_attention = false;
        let mut cache = LbaCache::new();
        let buf = [0x33u8; 512];
        cache.insert(0, 50, &buf);

        // Transition: Not Ready -> Ready (Card Insertion)
        fn set_presence(
            present: bool,
            media_present: &mut bool,
            unit_attention: &mut bool,
            cache: &mut LbaCache,
        ) {
            if !*media_present && present {
                *unit_attention = true;
                cache.clear();
            } else if *media_present && !present {
                *unit_attention = false;
                cache.clear();
            }
            *media_present = present;
        }

        set_presence(true, &mut media_present, &mut unit_attention, &mut cache);
        assert!(media_present);
        assert!(unit_attention);
        assert!(cache.lookup(50).is_none()); // Cache cleared

        // Idempotent: already present
        unit_attention = false; // Simulated UA reported & cleared
        cache.insert(0, 50, &buf);
        set_presence(true, &mut media_present, &mut unit_attention, &mut cache);
        assert!(media_present);
        assert!(!unit_attention); // Stays false
        assert!(cache.lookup(50).is_some()); // Not cleared

        // Transition: Ready -> Not Ready (Card Removal)
        set_presence(false, &mut media_present, &mut unit_attention, &mut cache);
        assert!(!media_present);
        assert!(!unit_attention);
        assert!(cache.lookup(50).is_none()); // Cache cleared on removal
    }

    #[test]
    fn test_scsi_unit_attention_asc_code() {
        assert_eq!(SenseKey::UnitAttention as u8, 0x06);
        assert_eq!(Asc::NotReadyToReadyTransition as u8, 0x28);
        assert_eq!(Asc::MediumNotPresent as u8, 0x3A);
    }
}
