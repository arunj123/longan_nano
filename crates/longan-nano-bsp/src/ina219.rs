use gd32vf103_hal::i2c::{I2c, I2cError};

pub const INA219_ADDR_DEFAULT: u8 = 0x40;

/// Fixed-point measurement snapshot from INA219.
/// Strictly uses integer arithmetic (mV, mA, mW) with zero software float emulation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ina219Data {
    pub voltage_mv: u16,
    pub current_ma: i16,
    pub current_tenth_ma: i16,
    pub power_mw: u16,
}

pub struct Ina219 {
    i2c: I2c,
    addr: u8,
}

impl Ina219 {
    pub const REG_CONFIG: u8 = 0x00;
    pub const REG_SHUNTVOLTAGE: u8 = 0x01;
    pub const REG_BUSVOLTAGE: u8 = 0x02;
    pub const REG_POWER: u8 = 0x03;
    pub const REG_CURRENT: u8 = 0x04;
    pub const REG_CALIBRATION: u8 = 0x05;

    pub fn new(i2c: I2c, addr: u8) -> Self {
        Self { i2c, addr }
    }

    /// Initializes INA219 with shunt calibration (default 4096 for 0.1 ohm shunt and 3.2A max).
    pub fn init(&mut self, cal: u16) -> Result<(), I2cError> {
        self.i2c.write_reg16(self.addr, Self::REG_CALIBRATION, cal)
    }

    /// Reads bus voltage in millivolts.
    pub fn read_voltage_mv(&mut self) -> Result<u16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_BUSVOLTAGE)?;
        Ok((raw >> 3) * 4)
    }

    /// Reads shunt current in milliamperes (signed).
    pub fn read_current_ma(&mut self) -> Result<i16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_CURRENT)?;
        Ok((raw as i16) / 10)
    }

    /// Reads calculated power in milliwatts.
    pub fn read_power_mw(&mut self) -> Result<u16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_POWER)?;
        Ok(raw * 2)
    }

    /// Reads all three measurements into Ina219Data.
    pub fn read_all(&mut self) -> Result<Ina219Data, I2cError> {
        let v_raw = self.i2c.read_reg16(self.addr, Self::REG_BUSVOLTAGE)?;
        let c_raw = self.i2c.read_reg16(self.addr, Self::REG_CURRENT)?;
        let p_raw = self.i2c.read_reg16(self.addr, Self::REG_POWER)?;

        let c_tenth = c_raw as i16;
        Ok(Ina219Data {
            voltage_mv: (v_raw >> 3) * 4,
            current_ma: c_tenth / 10,
            current_tenth_ma: c_tenth,
            power_mw: p_raw * 2,
        })
    }
}
