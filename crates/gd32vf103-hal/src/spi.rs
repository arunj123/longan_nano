use gd32vf103_pac::rcu::Rcu;
use gd32vf103_pac::spi::{ctl0, stat, Spi};
use crate::gpio::{mode, Pin, PortA, PortB};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Prescaler {
    Div2 = 0,
    Div4 = 1,
    Div8 = 2,
    Div16 = 3,
    Div32 = 4,
    Div64 = 5,
    Div128 = 6,
    Div256 = 7,
}

pub struct Spi0 {
    spi: Spi,
}

impl Spi0 {
    /// Initializes SPI0 as master with software NSS, Mode 0 (CPOL=0, CPHA=0).
    pub fn new_master(
        spi: Spi,
        _sck: Pin<PortA, 5, mode::Alternate<mode::PushPull>>,
        _mosi: Pin<PortA, 7, mode::Alternate<mode::PushPull>>,
        prescaler: Prescaler,
        rcu: &Rcu,
    ) -> Self {
        // Enable SPI0 clock on APB2 (Bit 12)
        rcu.regs().apb2en.set_bits(1 << 12);

        let regs = spi.regs();

        // Disable SPI before reconfiguring
        regs.ctl0.clear_bits(ctl0::SPIEN);

        let mut ctl = ctl0::MSTMOD | ctl0::SWNSS | ctl0::SWNSSEN;
        ctl |= (prescaler as u32) << 3;

        regs.ctl0.write(ctl);
        regs.ctl1.write(0);

        // Enable SPI
        regs.ctl0.set_bits(ctl0::SPIEN);

        Self { spi }
    }

    /// Wait until the SPI peripheral is idle and transmit buffer is empty.
    #[inline(always)]
    pub fn wait_idle(&self) {
        let regs = self.spi.regs();
        while (regs.stat.read() & stat::TRANS) != 0 {}
    }

    /// Set 8-bit frame format dynamically.
    #[inline]
    pub fn set_8bit(&mut self) {
        let regs = self.spi.regs();
        self.wait_idle();
        regs.ctl0.clear_bits(ctl0::SPIEN | ctl0::FF16);
        regs.ctl0.set_bits(ctl0::SPIEN);
    }

    /// Set 16-bit frame format dynamically.
    #[inline]
    pub fn set_16bit(&mut self) {
        let regs = self.spi.regs();
        self.wait_idle();
        regs.ctl0.clear_bits(ctl0::SPIEN);
        regs.ctl0.set_bits(ctl0::SPIEN | ctl0::FF16);
    }

    /// Transmit an 8-bit byte.
    #[inline(always)]
    pub fn send_u8(&mut self, byte: u8) {
        let regs = self.spi.regs();
        while (regs.stat.read() & stat::TBE) == 0 {}
        regs.data.write(byte as u32);
    }

    /// Transmit a 16-bit word.
    #[inline(always)]
    pub fn send_u16(&mut self, word: u16) {
        let regs = self.spi.regs();
        while (regs.stat.read() & stat::TBE) == 0 {}
        regs.data.write(word as u32);
    }

    /// Full-duplex 8-bit transfer.
    #[inline]
    pub fn transfer_u8(&mut self, byte: u8) -> u8 {
        let regs = self.spi.regs();
        while (regs.stat.read() & stat::TBE) == 0 {}
        regs.data.write(byte as u32);
        while (regs.stat.read() & stat::RBNE) == 0 {}
        (regs.data.read() & 0xFF) as u8
    }
}

pub struct Spi1 {
    spi: Spi,
}

impl Spi1 {
    /// Initializes SPI1 on PB13 (SCK), PB14 (MISO), PB15 (MOSI) on APB1.
    pub fn new_master(
        spi: Spi,
        _sck: Pin<PortB, 13, mode::Alternate<mode::PushPull>>,
        _miso: Pin<PortB, 14, mode::Input<mode::PullUp>>,
        _mosi: Pin<PortB, 15, mode::Alternate<mode::PushPull>>,
        prescaler: Prescaler,
        rcu: &Rcu,
    ) -> Self {
        // Enable SPI1 clock on APB1 (Bit 14)
        rcu.regs().apb1en.set_bits(1 << 14);

        let regs = spi.regs();

        // Disable SPI before reconfiguring
        regs.ctl0.clear_bits(ctl0::SPIEN);

        let mut ctl = ctl0::MSTMOD | ctl0::SWNSS | ctl0::SWNSSEN;
        ctl |= (prescaler as u32) << 3;

        regs.ctl0.write(ctl);
        regs.ctl1.write(0);

        // Enable SPI
        regs.ctl0.set_bits(ctl0::SPIEN);

        Self { spi }
    }

    /// Wait until the SPI peripheral is idle and transmit buffer is empty.
    #[inline(always)]
    pub fn wait_idle(&self) {
        let regs = self.spi.regs();
        while (regs.stat.read() & stat::TRANS) != 0 {}
    }

    /// Changes the SPI baud rate prescaler dynamically.
    #[inline]
    pub fn set_prescaler(&mut self, prescaler: Prescaler) {
        let regs = self.spi.regs();
        self.wait_idle();
        regs.ctl0.clear_bits(ctl0::SPIEN);
        regs.ctl0.modify(|val| (val & !(0x7 << 3)) | ((prescaler as u32) << 3));
        regs.ctl0.set_bits(ctl0::SPIEN);
    }

    /// Full-duplex 8-bit transfer.
    #[inline(always)]
    pub fn transfer_u8(&mut self, byte: u8) -> u8 {
        let regs = self.spi.regs();
        while (regs.stat.read() & stat::TBE) == 0 {}
        regs.data.write(byte as u32);
        while (regs.stat.read() & stat::RBNE) == 0 {}
        (regs.data.read() & 0xFF) as u8
    }

    /// Fast pipelined block read of 512 bytes.
    #[inline]
    pub fn read_block_fast(&mut self, buf: &mut [u8]) {
        let regs = self.spi.regs();
        let len = buf.len();
        if len == 0 {
            return;
        }

        // Prime the pipeline with the first 0xFF dummy byte
        while (regs.stat.read() & stat::TBE) == 0 {}
        regs.data.write(0xFF);

        for i in 0..len {
            // Send next byte while previous is in-flight
            if i + 1 < len {
                while (regs.stat.read() & stat::TBE) == 0 {}
                regs.data.write(0xFF);
            }
            while (regs.stat.read() & stat::RBNE) == 0 {}
            buf[i] = (regs.data.read() & 0xFF) as u8;
        }
    }

    /// Fast polling for token (0xFE) with timeout.
    #[inline]
    pub fn wait_token_fast(&mut self, max_bytes: u32) -> u8 {
        let regs = self.spi.regs();
        for _ in 0..max_bytes {
            while (regs.stat.read() & stat::TBE) == 0 {}
            regs.data.write(0xFF);
            while (regs.stat.read() & stat::RBNE) == 0 {}
            let b = (regs.data.read() & 0xFF) as u8;
            if b != 0xFF {
                return b;
            }
        }
        0xFF
    }

    /// Fast polling for card ready (MISO goes high 0xFF).
    #[inline]
    pub fn wait_ready_fast(&mut self, max_bytes: u32) -> bool {
        let regs = self.spi.regs();
        for _ in 0..max_bytes {
            while (regs.stat.read() & stat::TBE) == 0 {}
            regs.data.write(0xFF);
            while (regs.stat.read() & stat::RBNE) == 0 {}
            if (regs.data.read() & 0xFF) as u8 == 0xFF {
                return true;
            }
        }
        false
    }

    /// Fast block write of bytes.
    #[inline]
    pub fn write_block_fast(&mut self, buf: &[u8]) {
        let regs = self.spi.regs();
        for &byte in buf {
            while (regs.stat.read() & stat::TBE) == 0 {}
            regs.data.write(byte as u32);
            while (regs.stat.read() & stat::RBNE) == 0 {}
            let _ = regs.data.read(); // Clear RBNE
        }
    }
}
