use core::fmt;
use gd32vf103_pac::rcu::Rcu;
use gd32vf103_pac::usart::{ctl0, stat, Usart};
use crate::rcu::Clocks;
use crate::gpio::{mode, Pin, PortA};

pub struct Uart0 {
    usart: Usart,
}

impl Uart0 {
    /// Initializes USART0 at the specified baud rate with PA9 (TX) and PA10 (RX).
    pub fn new(
        usart: Usart,
        _tx: Pin<PortA, 9, mode::Alternate<mode::PushPull>>,
        _rx: Pin<PortA, 10, mode::Input<mode::Floating>>,
        baud: u32,
        clocks: &Clocks,
        rcu: &Rcu,
    ) -> Self {
        // Enable USART0 clock on APB2 (Bit 14)
        rcu.regs().apb2en.set_bits(1 << 14);

        let regs = usart.regs();

        // Calculate and configure baud rate divisor based on APB2 frequency
        let apb_clock = clocks.apb2;
        let udiv = (apb_clock + baud / 2) / baud;
        regs.baud.write(udiv);

        // Enable USART peripheral, transmitter, and receiver
        regs.ctl0.write(ctl0::UEN | ctl0::TEN | ctl0::REN);

        Self { usart }
    }

    /// Transmits a single byte (blocks until transmit data buffer is empty).
    #[inline]
    pub fn write_byte(&mut self, byte: u8) {
        let regs = self.usart.regs();
        while (regs.stat.read() & stat::TBE) == 0 {}
        regs.data.write(byte as u32);
    }

    /// Flushes transmission until complete.
    #[inline]
    pub fn flush(&mut self) {
        let regs = self.usart.regs();
        while (regs.stat.read() & stat::TC) == 0 {}
    }

    /// Checks if a received byte is available.
    #[inline]
    pub fn has_rx(&self) -> bool {
        (self.usart.regs().stat.read() & stat::RBNE) != 0
    }

    /// Reads a received byte if available.
    #[inline]
    pub fn read_byte(&mut self) -> Option<u8> {
        let stat = self.usart.regs().stat.read();
        if (stat & stat::RBNE) != 0 {
            Some((self.usart.regs().data.read() & 0xFF) as u8)
        } else {
            if (stat & (stat::ORERR | stat::FERR | stat::NERR)) != 0 {
                let _ = self.usart.regs().data.read();
            }
            None
        }
    }

    /// Reads a received byte (blocking until available).
    #[inline]
    pub fn read_byte_blocking(&mut self) -> u8 {
        while !self.has_rx() {}
        (self.usart.regs().data.read() & 0xFF) as u8
    }
}

impl fmt::Write for Uart0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
        Ok(())
    }
}
