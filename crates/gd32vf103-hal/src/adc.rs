use gd32vf103_pac::adc::{ctl1, stat, Adc as PacAdc};
use gd32vf103_pac::rcu::{apb2en, Rcu};
use crate::delay::Delay;
use embedded_hal::delay::DelayNs;

pub struct Adc0TempSensor {
    adc: PacAdc,
}

impl Adc0TempSensor {
    /// Initializes ADC0 and enables the internal temperature sensor on Channel 16.
    pub fn init(adc: PacAdc, rcu: &Rcu, delay: &mut Delay) -> Self {
        // 1. Enable ADC0 clock in RCU APB2EN and configure ADCPSC to /8
        rcu.regs().apb2en.set_bits(apb2en::ADC0EN);
        rcu.regs().cfg0.modify(|v| (v & !gd32vf103_pac::rcu::cfg0::ADCPSC_MASK) | gd32vf103_pac::rcu::cfg0::ADCPSC_DIV8);

        let regs = adc.regs();

        // 2. Enable temperature sensor and Vrefint (TSVREN in CTL1)
        regs.ctl1.set_bits(ctl1::TSVREN);

        // 3. Configure sampling time for Channel 16 & 17 to maximum (239.5 cycles >= 17.1 us)
        // Channel 16: bits 18..20, Channel 17: bits 21..23 in SAMPT0
        regs.sampt0.modify(|v| (v & !((7 << 18) | (7 << 21))) | (7 << 18) | (7 << 21));

        // 4. Configure regular channel sequence: length = 1 conversion
        regs.rsq0.modify(|v| v & !(0xF << 20));

        // 5. Channel 16 as default 1st conversion in regular sequence (RSQ2 bits 0..4)
        regs.rsq2.modify(|v| (v & !0x1F) | 16);

        // 6. External trigger source: None (software trigger only) + enable regular external trigger
        regs.ctl1.modify(|v| (v & !(7 << 17)) | ctl1::ETSRC_NONE | ctl1::ETERC);

        // 7. Power on ADC0
        regs.ctl1.set_bits(ctl1::ADCON);

        // 8. Stabilization delay (~15 us)
        delay.delay_us(15);

        // 9. Reset calibration
        regs.ctl1.set_bits(ctl1::RSTCLB);
        let mut timeout = 10_000;
        while (regs.ctl1.read() & ctl1::RSTCLB) != 0 && timeout > 0 {
            timeout -= 1;
        }

        // 10. Start ADC calibration
        regs.ctl1.set_bits(ctl1::CLB);
        timeout = 10_000;
        while (regs.ctl1.read() & ctl1::CLB) != 0 && timeout > 0 {
            timeout -= 1;
        }

        Self { adc }
    }

    #[inline]
    fn convert_channel(&self, ch: u8) -> u32 {
        let regs = self.adc.regs();
        regs.rsq2.modify(|v| (v & !0x1F) | (ch as u32));
        regs.stat.clear_bits(stat::EOC);
        regs.ctl1.set_bits(ctl1::SWRCST);
        let mut timeout = 20_000;
        while (regs.stat.read() & stat::EOC) == 0 && timeout > 0 {
            timeout -= 1;
        }
        (regs.rdata.read() & 0x0FFF) as u32
    }

    /// Reads internal Vrefint (Channel 17) to calculate the actual VDDA supply rail voltage in millivolts.
    pub fn read_vdda_mv(&mut self) -> u16 {
        let raw_vref = self.convert_channel(17);
        if raw_vref > 0 {
            // GD32VF103 typical Vrefint = 1200 mV (1.20 V)
            let vdda = (1200u32 * 4095) / raw_vref;
            vdda.clamp(2000, 4000) as u16
        } else {
            3300
        }
    }

    /// Performs an ADC conversion on Channel 16 and returns the MCU die temperature in tenths of a degree Celsius (e.g. 342 = 34.2 °C).
    /// Dynamically calibrates for actual VDDA supply voltage variations using Channel 17 (Vrefint).
    pub fn read_temperature_tenth_c(&mut self) -> i16 {
        let vdda_mv = self.read_vdda_mv() as u32;
        let raw_temp = self.convert_channel(16);

        // Convert 12-bit ADC code to mV using measured VDDA rail
        let v_sense_mv = (raw_temp * vdda_mv) / 4095;

        // GD32VF103 Datasheet formula:
        // Temp (°C) = ((V25 - Vsense) / Avg_Slope) + 25
        // V25 ≈ 1450 mV, Avg_Slope ≈ 4.3 mV/°C (= 43 / 10 mV/°C)
        // In tenths of °C:
        // Temp_tenth = (((1450 - Vsense) * 100) / 43) + 250
        let temp_tenth = (((1450i32 - v_sense_mv as i32) * 100) / 43) + 250;
        temp_tenth.clamp(-400, 1250) as i16
    }
}
