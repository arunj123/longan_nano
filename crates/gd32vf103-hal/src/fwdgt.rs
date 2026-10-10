use gd32vf103_pac::fwdgt::{ctl, psc, stat, Fwdgt as PacFwdgt};

pub struct Fwdgt {
    fwdgt: PacFwdgt,
}

impl Fwdgt {
    /// Initializes and starts the Free Watchdog Timer (FWDGT) clocked by internal IRC40K (~40 kHz).
    ///
    /// `timeout_ms` specifies the watchdog period in milliseconds (e.g. 2000 ms).
    /// Uses prescaler DIV64 (~625 Hz count rate, 1.6 ms per tick).
    pub fn start(fwdgt: PacFwdgt, timeout_ms: u16) -> Self {
        let regs = fwdgt.regs();

        // 1. Enable write access to FWDGT_PSC and FWDGT_RLD
        regs.ctl.write(ctl::CMD_WRITE_ENABLE);

        // 2. Wait until register update flags are cleared
        let mut timeout = 10_000;
        while (regs.stat.read() & (stat::PUD | stat::RUD)) != 0 && timeout > 0 {
            timeout -= 1;
        }

        // 3. Configure prescaler: DIV64 (40000 Hz / 64 = 625 Hz)
        regs.psc.write(psc::DIV64);

        // 4. Configure reload counter (timeout_ms * 625 / 1000)
        // For 2000 ms: 2000 * 625 / 1000 = 1250 ticks
        let ticks = ((timeout_ms as u32 * 625) / 1000).min(0x0FFF);
        regs.rld.write(ticks);

        // 5. Reload counter
        regs.ctl.write(ctl::CMD_RELOAD);

        // 6. Enable FWDGT counter
        regs.ctl.write(ctl::CMD_ENABLE);

        Self { fwdgt }
    }

    /// Reloads (kicks/feeds) the watchdog counter to prevent MCU reset.
    #[inline(always)]
    pub fn feed(&mut self) {
        self.fwdgt.regs().ctl.write(ctl::CMD_RELOAD);
    }
}
