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

    /// Checks if INA219 lost its calibration or config (e.g. from brownout/POR) and restores it.
    /// Returns Ok(true) if calibration was restored, Ok(false) if calibration was intact.
    pub fn check_and_restore_calibration(&mut self) -> Result<bool, I2cError> {
        let cal = self.i2c.read_reg16(self.addr, Self::REG_CALIBRATION)?;
        let cfg = self.i2c.read_reg16(self.addr, Self::REG_CONFIG)?;
        if cal != 4096 || cfg != INA219_CONFIG_DEFAULT {
            self.init(4096)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reads bus voltage in millivolts.
    pub fn read_voltage_mv(&mut self) -> Result<u16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_BUSVOLTAGE)?;
        Ok((raw >> 3) * 4)
    }

    /// Reads shunt current in milliamperes (signed, after tare compensation).
    /// Uses raw Shunt Voltage register (1 LSB = 10 uV = 0.1 mA across 0.1 ohm shunt)
    /// to remain immune to volatile calibration wipes.
    pub fn read_current_ma(&mut self) -> Result<i16, I2cError> {
        let raw = self.i2c.read_reg16(self.addr, Self::REG_SHUNTVOLTAGE)?;
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

    /// Pure decoder for INA219 raw register measurements.
    /// Decodes Bus Voltage (0x02) and Shunt Voltage (0x01) registers,
    /// and applies tare compensation using fixed-point integer arithmetic.
    #[inline]
    pub fn decode_raw(v_raw: u16, s_raw: u16, tare_offset_tenth: i16) -> Ina219Data {
        let math_overflow = (v_raw & 0x01) != 0;
        let conversion_ready = (v_raw & 0x02) != 0;
        let voltage_mv = (v_raw >> 3) * 4;

        // 1 LSB of Shunt Voltage = 10 uV.
        // Across 0.1 ohm shunt: 10 uV / 0.1 ohm = 100 uA = 0.1 mA (1 tenth of a mA).
        // Direct read from Shunt Voltage eliminates the redundant Current register (0x04)
        // read and is completely immune to volatile calibration wipes (brownout/POR).
        let raw_c_tenth = s_raw as i16;

        // Apply tare compensation with saturation guarding
        let current_tenth_ma = raw_c_tenth.saturating_sub(tare_offset_tenth);

        let abs_c_tenth = current_tenth_ma.unsigned_abs() as u32;
        let power_tenth_mw = (voltage_mv as u32 * abs_c_tenth) / 1000;
        let power_mw = (power_tenth_mw / 10) as u16;

        Ina219Data {
            voltage_mv,
            current_ma: current_tenth_ma / 10,
            current_tenth_ma,
            power_mw,
            power_tenth_mw,
            overflow: math_overflow,
            conversion_ready,
        }
    }

    /// Reads bus voltage, shunt current and flags into an `Ina219Data` record.
    /// Also clears the CNVR flag by reading the Power register per the INA219 datasheet.
    ///
    /// Optimized I2C Transaction Pipeline:
    /// Reads Bus Voltage (0x02), Shunt Voltage (0x01), and Power (0x03) to clear CNVR.
    /// Eliminates the redundant Current register (0x04) read, cutting I2C bus transactions
    /// from 4 to 3 (and eliminating redundant pre-conversion polling).
    pub fn read_all(&mut self) -> Result<Ina219Data, I2cError> {
        let v_raw = self.i2c.read_reg16(self.addr, Self::REG_BUSVOLTAGE)?;
        let s_raw = self.i2c.read_reg16(self.addr, Self::REG_SHUNTVOLTAGE)?;
        // Reading Power register clears CNVR per INA219 specification
        let _ = self.i2c.read_reg16(self.addr, Self::REG_POWER);

        Ok(Self::decode_raw(v_raw, s_raw, self.tare_offset_tenth))
    }
}

/// Pure decoder helper for INA219 raw register measurements.
#[inline]
pub fn decode_ina219_raw(v_raw: u16, s_raw: u16, tare_offset_tenth: i16) -> Ina219Data {
    Ina219::decode_raw(v_raw, s_raw, tare_offset_tenth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_positive_measurement() {
        // 5000 mV bus voltage: (5000 / 4) << 3 = 1250 << 3 = 10000 = 0x2710
        // CNVR = 1 (bit 1), OVF = 0 (bit 0) -> v_raw = 0x2712 = 10002
        let v_raw = 10002u16;
        // 500.0 mA current -> 5000 tenths-mA across 0.1 ohm shunt = 5000
        let s_raw = 5000u16;
        let data = decode_ina219_raw(v_raw, s_raw, 0);

        assert_eq!(data.voltage_mv, 5000);
        assert_eq!(data.current_tenth_ma, 5000);
        assert_eq!(data.current_ma, 500);
        // Power: (5000 mV * 5000 tenth-mA) / 1000 = 25000 tenth-mW = 2500 mW
        assert_eq!(data.power_tenth_mw, 25000);
        assert_eq!(data.power_mw, 2500);
        assert!(data.conversion_ready);
        assert!(!data.overflow);
    }

    #[test]
    fn test_decode_negative_reverse_current() {
        let v_raw = 10002u16; // 5000 mV
        // -250.4 mA -> -2504 tenth-mA = 0xF638
        let s_raw = (-2504i16) as u16;
        let data = decode_ina219_raw(v_raw, s_raw, 0);

        assert_eq!(data.voltage_mv, 5000);
        assert_eq!(data.current_tenth_ma, -2504);
        assert_eq!(data.current_ma, -250);
        // Power: (5000 * 2504) / 1000 = 12520 tenth-mW = 1252 mW
        assert_eq!(data.power_tenth_mw, 12520);
        assert_eq!(data.power_mw, 1252);
    }

    #[test]
    fn test_decode_tare_offset_compensation() {
        let v_raw = 10000u16; // 5000 mV, CNVR=0, OVF=0
        let s_raw = 1500u16;  // raw 150.0 mA
        let tare_offset = 500i16; // 50.0 mA offset
        let data = decode_ina219_raw(v_raw, s_raw, tare_offset);

        assert_eq!(data.voltage_mv, 5000);
        assert_eq!(data.current_tenth_ma, 1000); // 1500 - 500 = 1000 (100.0 mA)
        assert_eq!(data.current_ma, 100);
        assert!(!data.conversion_ready);
        assert!(!data.overflow);
    }

    #[test]
    fn test_decode_overflow_and_cnvr_flags() {
        let v_raw = (825u16 << 3) | 0x03; // 3300 mV, CNVR=1, OVF=1
        let s_raw = 0u16;
        let data = decode_ina219_raw(v_raw, s_raw, 0);

        assert_eq!(data.voltage_mv, 3300);
        assert_eq!(data.current_tenth_ma, 0);
        assert_eq!(data.power_mw, 0);
        assert!(data.conversion_ready);
        assert!(data.overflow);
    }
}
