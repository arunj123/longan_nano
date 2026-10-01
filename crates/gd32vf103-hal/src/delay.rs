use gd32vf103_pac::mtime::Mtime;
use crate::rcu::Clocks;
use embedded_hal::delay::DelayNs;

pub struct Delay {
    mtime: Mtime,
    clocks: Clocks,
}

impl Delay {
    pub fn new(mtime: Mtime, clocks: Clocks) -> Self {
        Self { mtime, clocks }
    }

    #[inline(always)]
    pub fn get_raw_ticks(&self) -> u64 {
        self.mtime.read_ticks()
    }

    #[inline(always)]
    pub fn uptime_ms(&self) -> u32 {
        let ticks_per_ms = (self.clocks.mtime_freq / 1000) as u64;
        (self.get_raw_ticks() / ticks_per_ms) as u32
    }
}

impl DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        let ticks = ((ns as u64) * (self.clocks.mtime_freq as u64)) / 1_000_000_000;
        let start = self.mtime.read_ticks();
        while self.mtime.read_ticks().wrapping_sub(start) < ticks {}
    }

    fn delay_us(&mut self, us: u32) {
        let ticks = ((us as u64) * (self.clocks.mtime_freq as u64)) / 1_000_000;
        let start = self.mtime.read_ticks();
        while self.mtime.read_ticks().wrapping_sub(start) < ticks {}
    }

    fn delay_ms(&mut self, ms: u32) {
        let ticks = ((ms as u64) * (self.clocks.mtime_freq as u64)) / 1_000;
        let start = self.mtime.read_ticks();
        while self.mtime.read_ticks().wrapping_sub(start) < ticks {}
    }
}
