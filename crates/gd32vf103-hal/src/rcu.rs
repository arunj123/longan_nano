use gd32vf103_pac::rcu::{ctl, cfg0, cfg1, Rcu};

#[derive(Clone, Copy)]
pub struct Clocks {
    pub sysclk: u32,
    pub ahb: u32,
    pub apb1: u32,
    pub apb2: u32,
    pub mtime_freq: u32,
}

impl Clocks {
    pub const fn default_108mhz() -> Self {
        Self {
            sysclk: 108_000_000,
            ahb: 108_000_000,
            apb1: 54_000_000,
            apb2: 108_000_000,
            mtime_freq: 27_000_000, // sysclk / 4
        }
    }

    pub const fn default_96mhz() -> Self {
        Self {
            sysclk: 96_000_000,
            ahb: 96_000_000,
            apb1: 48_000_000,
            apb2: 96_000_000,
            mtime_freq: 24_000_000, // sysclk / 4
        }
    }

    pub const fn default_irc8m() -> Self {
        Self {
            sysclk: 8_000_000,
            ahb: 8_000_000,
            apb1: 8_000_000,
            apb2: 8_000_000,
            mtime_freq: 2_000_000, // sysclk / 4
        }
    }
}

pub struct RcuConfig {
    pub use_hxtal: bool,
    pub target_sysclk: u32,
}

impl Default for RcuConfig {
    fn default() -> Self {
        Self {
            use_hxtal: true,
            target_sysclk: 108_000_000,
        }
    }
}

pub trait RcuExt {
    fn freeze(self, config: RcuConfig) -> (Rcu, Clocks);
}

impl RcuExt for Rcu {
    fn freeze(self, config: RcuConfig) -> (Rcu, Clocks) {
        let regs = self.regs();

        // 1. Enable IRC8M internal oscillator
        regs.ctl.set_bits(ctl::IRC8MEN);
        while (regs.ctl.read() & ctl::IRC8MSTB) == 0 {}

        // Reset clock config registers
        regs.cfg0.clear_bits(0x08FF_0FFF);
        regs.ctl.clear_bits(ctl::HXTALEN | ctl::CKMEN | ctl::PLLEN | ctl::HXTALBPS);
        regs.cfg0.clear_bits(0x203F_0000);
        regs.cfg1.write(0);
        regs.ctl.clear_bits(ctl::PLLEN | ctl::PLL1EN | ctl::PLL2EN | ctl::CKMEN | ctl::HXTALEN);
        regs.int.write(0x00FF_0000);

        if !config.use_hxtal {
            return (self, Clocks::default_irc8m());
        }

        // Enable HXTAL (8 MHz crystal)
        regs.ctl.set_bits(ctl::HXTALEN);
        let mut timeout = 0xFFFFu32;
        while ((regs.ctl.read() & ctl::HXTALSTB) == 0) && timeout > 0 {
            timeout -= 1;
        }

        if (regs.ctl.read() & ctl::HXTALSTB) == 0 {
            // Fallback to IRC8M
            return (self, Clocks::default_irc8m());
        }

        // AHB = SYSCLK / 1, APB2 = AHB / 1, APB1 = AHB / 2 (max APB1 is 54 MHz)
        // ADCPSC = APB2 / 8 (12 MHz at 96 MHz SYSCLK <= 14 MHz max ADC clock)
        regs.cfg0.set_bits(cfg0::AHB_DIV1 | cfg0::APB2_DIV1 | cfg0::APB1_DIV2 | cfg0::ADCPSC_DIV8);

        // PREDV0 = HXTAL (8 MHz) / 2 = 4 MHz
        regs.cfg1.modify(|val| (val & !0x0001_FFFF) | (cfg1::PREDV0SRC_HXTAL | cfg1::PREDV0_DIV2));

        if config.target_sysclk == 96_000_000 {
            // CK_PLL = 4 MHz * 24 = 96 MHz
            // pllmf = 6 (bits 21:18 = 0b0110), pllmf4 = 1 (bit 29), pllsel = 1 (bit 16)
            // USBFS prescaler = Div2 (bits 23:22 = 0b11)
            regs.cfg0.modify(|val| {
                let mask = (0xF << 18) | (1 << 29) | (0x3 << 22);
                (val & !mask) | (1 << 16) | (0x6 << 18) | (1 << 29) | (0x3 << 22)
            });

            // Enable PLL
            regs.ctl.set_bits(ctl::PLLEN);
            while (regs.ctl.read() & ctl::PLLSTB) == 0 {}

            // Switch to PLL
            regs.cfg0.modify(|val| (val & !cfg0::SCS_MASK) | cfg0::SCS_PLL);
            while (regs.cfg0.read() & cfg0::SCSS_MASK) != cfg0::SCSS_PLL {}

            (self, Clocks::default_96mhz())
        } else if config.target_sysclk == 108_000_000 {
            // CK_PLL = 4 MHz * 27 = 108 MHz
            // pllmf = 9 (bits 21:18 = 0b1001), pllmf4 = 1 (bit 29), pllsel = 1 (bit 16)
            regs.cfg0.modify(|val| {
                let mask = (0xF << 18) | (1 << 29);
                (val & !mask) | (1 << 16) | (0x9 << 18) | (1 << 29)
            });

            // Enable PLL
            regs.ctl.set_bits(ctl::PLLEN);
            while (regs.ctl.read() & ctl::PLLSTB) == 0 {}

            // Switch to PLL
            regs.cfg0.modify(|val| (val & !cfg0::SCS_MASK) | cfg0::SCS_PLL);
            while (regs.cfg0.read() & cfg0::SCSS_MASK) != cfg0::SCSS_PLL {}

            (self, Clocks::default_108mhz())
        } else {
            (self, Clocks::default_irc8m())
        }
    }
}
