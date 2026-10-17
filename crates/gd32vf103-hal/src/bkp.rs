use gd32vf103_pac as pac;
use gd32vf103_pac::rcu::apb1en;
use gd32vf103_pac::pmu::ctl as pmu_ctl;

/// Backup Domain Register Interface (GD32VF103).
///
/// Provides access to the 10 16-bit battery-backed / VDD-backed data registers
/// (BKP_DATA0 through BKP_DATA9) preserved across software and watchdog resets.
pub struct Bkp {
    bkp: pac::Bkp,
    pmu: pac::Pmu,
}

impl Bkp {
    /// Initializes Backup Domain peripheral by enabling PMU and BKP interface clocks
    /// and enabling backup domain write access.
    pub fn init(bkp: pac::Bkp, pmu: pac::Pmu, rcu: &pac::Rcu) -> Self {
        // 1. Enable PMU and BKP interface clocks on APB1
        rcu.regs().apb1en.modify(|r| r | apb1en::PMUEN | apb1en::BKPIEN);

        // 2. Enable write access to Backup Domain registers
        pmu.regs().ctl.modify(|r| r | pmu_ctl::BKPWEN);

        Self { bkp, pmu }
    }

    /// Reads a 16-bit value from Backup Domain Data Register (index 0..=9).
    pub fn read_reg(&self, index: usize) -> u16 {
        let regs = self.bkp.regs();
        let val = match index {
            0 => regs.data0.read(),
            1 => regs.data1.read(),
            2 => regs.data2.read(),
            3 => regs.data3.read(),
            4 => regs.data4.read(),
            5 => regs.data5.read(),
            6 => regs.data6.read(),
            7 => regs.data7.read(),
            8 => regs.data8.read(),
            9 => regs.data9.read(),
            _ => 0,
        };
        (val & 0xFFFF) as u16
    }

    /// Writes a 16-bit value to Backup Domain Data Register (index 0..=9).
    pub fn write_reg(&mut self, index: usize, val: u16) {
        self.pmu.regs().ctl.modify(|r| r | pmu_ctl::BKPWEN);
        let regs = self.bkp.regs();
        let val32 = (val as u32) & 0xFFFF;
        match index {
            0 => regs.data0.write(val32),
            1 => regs.data1.write(val32),
            2 => regs.data2.write(val32),
            3 => regs.data3.write(val32),
            4 => regs.data4.write(val32),
            5 => regs.data5.write(val32),
            6 => regs.data6.write(val32),
            7 => regs.data7.write(val32),
            8 => regs.data8.write(val32),
            9 => regs.data9.write(val32),
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bkp_constants_and_limits() {
        assert_eq!(apb1en::PMUEN, 1 << 28);
        assert_eq!(apb1en::BKPIEN, 1 << 27);
        assert_eq!(pmu_ctl::BKPWEN, 1 << 8);
    }
}

