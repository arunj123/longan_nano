use gd32vf103_pac::i2c::{ckcfg, ctl0, ctl1, stat0, stat1, I2c as PacI2c};
use gd32vf103_pac::rcu::{apb1en, Rcu};
use crate::rcu::Clocks;
use embedded_hal::delay::DelayNs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cError {
    BusBusy,
    StartTimeout,
    AddrTimeout,
    TxTimeout,
    RxTimeout,
    AcknowledgeError,
    StopTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DutyCycle {
    Ratio2,
    Ratio16_9,
}

const DEFAULT_TIMEOUT: u32 = 50_000;

pub struct I2c {
    i2c: PacI2c,
    freq: u32,
    risetime: u32,
    ckcfg: u32,
}

impl I2c {
    /// Creates and initializes an I2C master interface.
    pub fn new<SCL, SDA>(
        i2c: PacI2c,
        _scl: SCL,
        _sda: SDA,
        clkspeed: u32,
        clocks: &Clocks,
        rcu: &Rcu,
    ) -> Self {
        // Enable I2C0 / I2C1 peripheral clock in RCU APB1EN
        let base_addr = i2c.regs() as *const _ as usize;
        if base_addr == gd32vf103_pac::i2c::I2C0_BASE {
            rcu.regs().apb1en.set_bits(apb1en::I2C0EN);
        } else {
            rcu.regs().apb1en.set_bits(apb1en::I2C1EN);
        }

        let pclk1 = clocks.apb1;

        // Peripheral clock frequency in MHz (2 to 54 MHz for GD32VF103)
        let mut freq = pclk1 / 1_000_000;
        if freq > 54 {
            freq = 54;
        }
        if freq < 2 {
            freq = 2;
        }

        let regs = i2c.regs();

        // Disable peripheral during configuration
        regs.ctl0.clear_bits(ctl0::I2CEN);

        // Configure peripheral clock in CTL1
        regs.ctl1.modify(|curr| (curr & !ctl1::I2CCLK_MASK) | (freq & ctl1::I2CCLK_MASK));

        let (risetime, ckcfg_val) = if clkspeed <= 100_000 {
            // Standard mode: maximum rise time is 1000ns
            let mut rt = (pclk1 / 1_000_000) + 1;
            if rt > 54 {
                rt = 54;
            }
            if rt < 2 {
                rt = 2;
            }
            regs.rt.write(rt);

            let mut clkc = pclk1 / (clkspeed * 2);
            if clkc < 4 {
                clkc = 4;
            }
            let cfg = clkc & ckcfg::CLKC_MASK;
            regs.ckcfg.write(cfg);
            (rt, cfg)
        } else {
            // Fast mode: maximum rise time is 300ns
            let rt = ((freq * 300) / 1000) + 1;
            regs.rt.write(rt);

            let mut clkc = pclk1 / (clkspeed * 3);
            if clkc < 1 {
                clkc = 1;
            }
            let cfg = ckcfg::FAST | (clkc & ckcfg::CLKC_MASK);
            regs.ckcfg.write(cfg);
            (rt, cfg)
        };

        // Configure 7-bit addressing, slave address 0
        regs.ctl0.clear_bits(ctl0::SMBEN);
        regs.saddr0.write(0);

        // Enable peripheral and ACK
        regs.ctl0.set_bits(ctl0::I2CEN | ctl0::ACKEN);

        Self {
            i2c,
            freq,
            risetime,
            ckcfg: ckcfg_val,
        }
    }

    /// Performs 9-cycle SCL bus recovery to clear stuck slave holding SDA low,
    /// emits a manual STOP condition, and resets the peripheral.
    pub fn recover_bus(&mut self, delay: &mut crate::delay::Delay) {
        use gd32vf103_pac::gpio::{GpioPort, Port};
        use gd32vf103_pac::rcu::{apb1rst, Rcu};

        let base_addr = self.i2c.regs() as *const _ as usize;

        // 1. Disable I2C peripheral
        self.i2c.regs().ctl0.clear_bits(ctl0::I2CEN);

        // 2. Configure PB6 (SCL) and PB7 (SDA) as GPIO Open-Drain 50 MHz (bits: 0b0111 = 0x7)
        let gpiob = unsafe { GpioPort::steal(Port::B) };
        let regs_b = gpiob.regs();

        // Pin 6: bits 24..27, Pin 7: bits 28..31 in CTL0
        regs_b.ctl0.modify(|curr| {
            let mask = (0xF << 24) | (0xF << 28);
            let val = (0x7 << 24) | (0x7 << 28);
            (curr & !mask) | val
        });

        // Set SCL and SDA HIGH initially
        regs_b.bop.write((1 << 6) | (1 << 7));
        delay.delay_us(5);

        // 3. Up to 9 SCL clock pulses to free SDA
        for _ in 0..9 {
            if (regs_b.istat.read() & (1 << 7)) != 0 {
                break;
            }
            // SCL LOW
            regs_b.bc.write(1 << 6);
            delay.delay_us(5);
            // SCL HIGH
            regs_b.bop.write(1 << 6);
            delay.delay_us(5);
        }

        // 4. Generate manual STOP condition (SCL LOW -> SDA LOW -> SCL HIGH -> SDA HIGH)
        regs_b.bc.write(1 << 6);
        delay.delay_us(5);
        regs_b.bc.write(1 << 7);
        delay.delay_us(5);
        regs_b.bop.write(1 << 6);
        delay.delay_us(5);
        regs_b.bop.write(1 << 7);
        delay.delay_us(5);

        // 5. Restore PB6 and PB7 to Alternate Function Open-Drain 50 MHz (0b1111 = 0xF)
        regs_b.ctl0.modify(|curr| {
            let mask = (0xF << 24) | (0xF << 28);
            let val = (0xF << 24) | (0xF << 28);
            (curr & !mask) | val
        });

        // 6. Reset I2C peripheral via RCU APB1RST
        let rcu = unsafe { Rcu::steal() };
        if base_addr == gd32vf103_pac::i2c::I2C0_BASE {
            rcu.regs().apb1rst.set_bits(apb1rst::I2C0RST);
            delay.delay_us(2);
            rcu.regs().apb1rst.clear_bits(apb1rst::I2C0RST);
        } else {
            rcu.regs().apb1rst.set_bits(apb1rst::I2C1RST);
            delay.delay_us(2);
            rcu.regs().apb1rst.clear_bits(apb1rst::I2C1RST);
        }

        // 7. Restore I2C peripheral configuration registers
        let regs = self.i2c.regs();
        regs.ctl0.clear_bits(ctl0::I2CEN);
        regs.ctl1.modify(|curr| (curr & !ctl1::I2CCLK_MASK) | (self.freq & ctl1::I2CCLK_MASK));
        regs.rt.write(self.risetime);
        regs.ckcfg.write(self.ckcfg);
        regs.ctl0.clear_bits(ctl0::SMBEN);
        regs.saddr0.write(0);
        regs.ctl0.set_bits(ctl0::I2CEN | ctl0::ACKEN);
    }

    #[inline(always)]
    fn wait_clear(&self, mask: u32, is_stat1: bool) -> bool {
        let regs = self.i2c.regs();
        let mut timeout = DEFAULT_TIMEOUT;
        while timeout > 0 {
            let val = if is_stat1 {
                regs.stat1.read()
            } else {
                regs.ctl0.read()
            };
            if (val & mask) == 0 {
                return true;
            }
            timeout -= 1;
        }
        false
    }

    #[inline(always)]
    fn wait_set_stat0(&self, mask: u32) -> bool {
        let regs = self.i2c.regs();
        let mut timeout = DEFAULT_TIMEOUT;
        while timeout > 0 {
            let s0 = regs.stat0.read();
            if (s0 & stat0::AERR) != 0 {
                // Acknowledge failure
                regs.stat0.clear_bits(stat0::AERR);
                return false;
            }
            if (s0 & mask) != 0 {
                return true;
            }
            timeout -= 1;
        }
        false
    }

    #[inline(always)]
    fn clear_addsend(&self) {
        let regs = self.i2c.regs();
        let _ = regs.stat0.read();
        let _ = regs.stat1.read();
    }

    /// Performs a 16-bit big-endian register write to an I2C device.
    pub fn write_reg16(&mut self, dev_addr: u8, reg_addr: u8, value: u16) -> Result<(), I2cError> {
        let regs = self.i2c.regs();

        // Wait until I2C bus is idle
        if !self.wait_clear(stat1::I2CBSY, true) {
            return Err(I2cError::BusBusy);
        }

        // Generate START condition
        regs.ctl0.set_bits(ctl0::START);
        if !self.wait_set_stat0(stat0::SBSEND) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::StartTimeout);
        }

        // Send 7-bit device address in transmitter mode (bit 0 = 0)
        regs.data.write((dev_addr as u32) << 1);
        if !self.wait_set_stat0(stat0::ADDSEND) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::AddrTimeout);
        }
        self.clear_addsend();

        // Send register address
        if !self.wait_set_stat0(stat0::TBE) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::TxTimeout);
        }
        regs.data.write(reg_addr as u32);

        // Send MSB
        if !self.wait_set_stat0(stat0::TBE) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::TxTimeout);
        }
        regs.data.write((value >> 8) as u32);

        // Send LSB
        if !self.wait_set_stat0(stat0::TBE) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::TxTimeout);
        }
        regs.data.write((value & 0xFF) as u32);

        // Wait for byte transfer complete
        if !self.wait_set_stat0(stat0::TBE) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::TxTimeout);
        }

        // Send STOP condition
        regs.ctl0.set_bits(ctl0::STOP);
        if !self.wait_clear(ctl0::STOP, false) {
            return Err(I2cError::StopTimeout);
        }

        Ok(())
    }

    /// Performs a 16-bit big-endian register read from an I2C device.
    pub fn read_reg16(&mut self, dev_addr: u8, reg_addr: u8) -> Result<u16, I2cError> {
        let regs = self.i2c.regs();

        // Wait until I2C bus is idle
        if !self.wait_clear(stat1::I2CBSY, true) {
            return Err(I2cError::BusBusy);
        }

        // Generate START condition
        regs.ctl0.set_bits(ctl0::START);
        if !self.wait_set_stat0(stat0::SBSEND) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::StartTimeout);
        }

        // Send device address in transmitter mode
        regs.data.write((dev_addr as u32) << 1);
        if !self.wait_set_stat0(stat0::ADDSEND) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::AddrTimeout);
        }
        self.clear_addsend();

        // Send register address
        if !self.wait_set_stat0(stat0::TBE) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::TxTimeout);
        }
        regs.data.write(reg_addr as u32);

        if !self.wait_set_stat0(stat0::TBE) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::TxTimeout);
        }

        // Generate Repeated START
        regs.ctl0.set_bits(ctl0::START);
        if !self.wait_set_stat0(stat0::SBSEND) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::StartTimeout);
        }

        // Send device address in receiver mode (bit 0 = 1)
        regs.data.write(((dev_addr as u32) << 1) | 1);
        if !self.wait_set_stat0(stat0::ADDSEND) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::AddrTimeout);
        }
        self.clear_addsend();

        // Read MSB
        if !self.wait_set_stat0(stat0::RBNE) {
            regs.ctl0.set_bits(ctl0::STOP);
            return Err(I2cError::RxTimeout);
        }
        let msb = (regs.data.read() & 0xFF) as u8;

        // Disable ACK before receiving the last byte
        regs.ctl0.clear_bits(ctl0::ACKEN);

        // Generate STOP condition
        regs.ctl0.set_bits(ctl0::STOP);

        // Read LSB
        if !self.wait_set_stat0(stat0::RBNE) {
            regs.ctl0.set_bits(ctl0::ACKEN);
            return Err(I2cError::RxTimeout);
        }
        let lsb = (regs.data.read() & 0xFF) as u8;

        let stop_ok = self.wait_clear(ctl0::STOP, false);

        // Re-enable ACK for subsequent transfers
        regs.ctl0.set_bits(ctl0::ACKEN);

        if !stop_ok {
            return Err(I2cError::StopTimeout);
        }

        Ok(((msb as u16) << 8) | (lsb as u16))
    }
}
