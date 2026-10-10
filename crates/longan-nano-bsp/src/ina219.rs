use gd32vf103_hal::i2c::{I2c, I2cError};

pub const INA219_ADDR_DEFAULT: u8 = 0x40;

/// Default high-accuracy configuration for INA219:
/// - Bus Voltage Range: 32V (BRNG = 1)
/// - Shunt PGA Gain: /8 (±320mV, up to 3.2A with 0.1 ohm shunt) (PG = 0b11)
/// - Bus ADC: 12-bit, 16 samples averaged (8.51 ms conversion) (BADC = 0b1100)
/// - Shunt ADC: 12-bit, 128 samples averaged (68.10 ms conversion) (SADC = 0b1111)
/// - Operating Mode: Shunt and Bus Continuous (MODE = 0b111)
/// Total conversion cycle ≈ 76.6 ms (comfortably within 100 ms period, with ~68% true time coverage).
pub const INA219_CONFIG_DEFAULT: u16 = (1 << 13) | (0b11 << 11) | (0b1100 << 7) | (0b1111 << 3) | 0b111; // 0x3E7F

/// Fixed-point measurement snapshot from INA219.
/// Strictly uses integer arithmetic (mV, mA, mW) with zero software float emulation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ina219Data {
    pub voltage_mv: u16,
    pub current_ma: i16,
    pub current_tenth_ma: i16,
    pub power_mw: u16,
    pub power_tenth_mw: u32,
    pub overflow: bool,
    pub conversion_ready: bool,
}

pub struct Ina219 {
    i2c: I2c,
    addr: u8,
    pub tare_offset_tenth: i16,
}

impl Ina219 {
    pub const REG_CONFIG: u8 = 0x00;
    pub const REG_SHUNTVOLTAGE: u8 = 0x01;
    pub const REG_BUSVOLTAGE: u8 = 0x02;
    pub const REG_POWER: u8 = 0x03;
    pub const REG_CURRENT: u8 = 0x04;
    pub const REG_CALIBRATION: u8 = 0x05;

    pub fn new(i2c: I2c, addr: u8) -> Self {
        Self {
            i2c,
            addr,
            tare_offset_tenth: 0,
        }
    }

    /// Initializes INA219 with default configuration and calibration.
    /// Default cal = 4096 for 0.1 ohm shunt (0.1 mA LSB, 3.2A max).
    pub fn init(&mut self, cal: u16) -> Result<(), I2cError> {
        self.init_with_config(cal, INA219_CONFIG_DEFAULT)
    }

    /// Initializes INA219 with custom configuration and calibration register values.
    pub fn init_with_config(&mut self, cal: u16, config: u16) -> Result<(), I2cError> {
        self.i2c.write_reg16(self.addr, Self::REG_CONFIG, config)?;
        self.i2c.write_reg16(self.addr, Self::REG_CALIBRATION, cal)?;
        Ok(())
    }

    /// Performs I2C 9-cycle bus recovery to clear bus lockup before reinitializing.
    pub fn recover_bus(&mut self, delay: &mut gd32vf103_hal::delay::Delay) {
        self.i2c.recover_bus(delay);
    }

    /// Sets the tare (zero-offset) current in tenths of mA.
    #[inline]
    pub fn set_tare(&mut self, offset_tenth_ma: i16) {
        self.tare_offset_tenth = offset_tenth_ma;
    }

    /// Reads bus voltage in millivolts.
    pub fn read_voltage_mv(&mut self) -> Result<u16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_BUSVOLTAGE)?;
        Ok((raw >> 3) * 4)
    }

    /// Reads shunt current in milliamperes (signed, after tare compensation).
    pub fn read_current_ma(&mut self) -> Result<i16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_CURRENT)?;
        let c_tenth = (raw as i16).saturating_sub(self.tare_offset_tenth);
        Ok(c_tenth / 10)
    }

    /// Checks if a new conversion has completed (CNVR bit in Bus Voltage register).
    pub fn is_conversion_ready(&mut self) -> Result<bool, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_BUSVOLTAGE)?;
        Ok((raw & 0x02) != 0)
    }

    /// Reads calculated power in milliwatts.
    pub fn read_power_mw(&mut self) -> Result<u16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_POWER)?;
        Ok(raw * 2)
    }

    /// Reads bus voltage, shunt current and flags into an `Ina219Data` record.
    /// Also clears the CNVR flag by reading the Power register per the INA219 datasheet.
    pub fn read_all(&mut self) -> Result<Ina219Data, I2cError> {
        let v_raw = self.i2c.read_reg16(self.addr, Self::REG_BUSVOLTAGE)?;
        let c_raw = self.i2c.read_reg16(self.addr, Self::REG_CURRENT)?;
        // Reading Power register clears CNVR per INA219 specification
        let _ = self.i2c.read_reg16(self.addr, Self::REG_POWER);

        let math_overflow = (v_raw & 0x01) != 0;
        let conversion_ready = (v_raw & 0x02) != 0;
        let voltage_mv = (v_raw >> 3) * 4;

        // Apply tare compensation with saturation guarding
        let raw_c_tenth = c_raw as i16;
        let current_tenth_ma = raw_c_tenth.saturating_sub(self.tare_offset_tenth);

        let abs_c_tenth = current_tenth_ma.unsigned_abs() as u32;
        let power_tenth_mw = (voltage_mv as u32 * abs_c_tenth) / 1000;
        let power_mw = (power_tenth_mw / 10) as u16;

        Ok(Ina219Data {
            voltage_mv,
            current_ma: current_tenth_ma / 10,
            current_tenth_ma,
            power_mw,
            power_tenth_mw,
            overflow: math_overflow,
            conversion_ready,
        })
    }
}
