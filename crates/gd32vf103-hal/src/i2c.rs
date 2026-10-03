use gd32vf103_pac::i2c::{ckcfg, ctl0, ctl1, stat0, stat1, I2c as PacI2c};
use gd32vf103_pac::rcu::{apb1en, Rcu};
use crate::rcu::Clocks;

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

        if clkspeed <= 100_000 {
            // Standard mode: maximum rise time is 1000ns
            let mut risetime = (pclk1 / 1_000_000) + 1;
            if risetime > 54 {
                risetime = 54;
            }
            if risetime < 2 {
                risetime = 2;
            }
            regs.rt.write(risetime);

            let mut clkc = pclk1 / (clkspeed * 2);
            if clkc < 4 {
                clkc = 4;
            }
            regs.ckcfg.write(clkc & ckcfg::CLKC_MASK);
        } else {
            // Fast mode: maximum rise time is 300ns
            let risetime = ((freq * 300) / 1000) + 1;
            regs.rt.write(risetime);

            let mut clkc = pclk1 / (clkspeed * 3);
            if clkc < 1 {
                clkc = 1;
            }
            regs.ckcfg.write(ckcfg::FAST | (clkc & ckcfg::CLKC_MASK));
        }

        // Configure 7-bit addressing, slave address 0
        regs.ctl0.clear_bits(ctl0::SMBEN);
        regs.saddr0.write(0);

        // Enable peripheral and ACK
        regs.ctl0.set_bits(ctl0::I2CEN | ctl0::ACKEN);

        Self { i2c }
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
