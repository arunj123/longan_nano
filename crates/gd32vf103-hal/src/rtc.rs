use gd32vf103_pac as pac;
use gd32vf103_pac::rcu::{apb1en, bdctl, rstsck};
use gd32vf103_pac::rtc::ctl;
use gd32vf103_pac::pmu::ctl as pmu_ctl;

/// Human-readable date and time representation (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    pub const fn zero() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

/// Converts Unix epoch seconds (since 1970-01-01 00:00:00 UTC) to `DateTime`
/// using Howard Hinnant's branch-free civil calendar algorithm.
pub fn epoch_to_datetime(epoch: u32) -> DateTime {
    let second = (epoch % 60) as u8;
    let epoch_min = epoch / 60;
    let minute = (epoch_min % 60) as u8;
    let epoch_hr = epoch_min / 60;
    let hour = (epoch_hr % 24) as u8;
    let mut days = (epoch_hr / 24) as i32;

    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i32) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
    let year = (if month <= 2 { y + 1 } else { y }) as u16;

    DateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    }
}

pub struct Rtc {
    rtc: pac::Rtc,
    #[allow(dead_code)]
    bkp: pac::Bkp,
    pmu: pac::Pmu,
    synced: bool,
}

impl Rtc {
    /// Initializes GD32VF103 Real-Time Clock with IRC40K oscillator and 1 Hz prescaler.
    /// Preserves existing counter if backup register indicates prior initialization.
    pub fn init(rtc: pac::Rtc, bkp: pac::Bkp, pmu: pac::Pmu, rcu: &pac::Rcu) -> Self {
        // 1. Enable PMU and BKP interface clocks
        rcu.regs().apb1en.modify(|r| r | apb1en::PMUEN | apb1en::BKPIEN);

        // 2. Enable write access to Backup Domain and RTC registers
        pmu.regs().ctl.modify(|r| r | pmu_ctl::BKPWEN);

        // Check if previously configured via BKP_DATA0 magic signature
        let is_configured = (bkp.regs().data0.read() & 0xFFFF) == 0xA5A5;

        if !is_configured {
            // Enable internal 40 kHz RC oscillator
            rcu.regs().rstsck.modify(|r| r | rstsck::IRC40KEN);
            let mut timeout = 100_000;
            while (rcu.regs().rstsck.read() & rstsck::IRC40KSTB) == 0 && timeout > 0 {
                timeout -= 1;
            }

            // Configure BDCTL: Select IRC40K as RTC clock source and enable RTC
            rcu.regs().bdctl.modify(|r| {
                (r & !bdctl::RTCSRC_MASK) | bdctl::RTCSRC_IRC40K | bdctl::RTCEN
            });

            // Wait for last write operation to complete
            Self::wait_lwoff(&rtc);

            // Wait for registers synchronization flag
            Self::wait_rsynf(&rtc);

            // Enter RTC configuration mode
            Self::enter_config(&rtc);

            // Set prescaler for 40 kHz -> 1 Hz (PSC = 39,999 = 0x9C3F)
            rtc.regs().psch.write(0);
            rtc.regs().pscl.write(39_999);

            // Exit configuration mode
            Self::exit_config(&rtc);

            // Write magic signature to BKP_DATA0 to preserve settings across soft resets
            bkp.regs().data0.write(0xA5A5);
        } else {
            // Wait for registers synchronization
            Self::wait_rsynf(&rtc);
        }

        let mut instance = Self {
            rtc,
            bkp,
            pmu,
            synced: false,
        };

        if instance.get_epoch() > 1_700_000_000 {
            instance.synced = true;
        }

        instance
    }

    #[inline(always)]
    fn wait_lwoff(rtc: &pac::Rtc) {
        let mut timeout = 200_000;
        while (rtc.regs().ctl.read() & ctl::LWOFF) == 0 && timeout > 0 {
            timeout -= 1;
        }
    }

    #[inline(always)]
    fn wait_rsynf(rtc: &pac::Rtc) {
        Self::wait_lwoff(rtc);
        // Clear RSYNF bit
        rtc.regs().ctl.modify(|r| r & !ctl::RSYNF);
        let mut timeout = 200_000;
        while (rtc.regs().ctl.read() & ctl::RSYNF) == 0 && timeout > 0 {
            timeout -= 1;
        }
    }

    #[inline(always)]
    fn enter_config(rtc: &pac::Rtc) {
        Self::wait_lwoff(rtc);
        rtc.regs().ctl.modify(|r| r | ctl::CMF);
    }

    #[inline(always)]
    fn exit_config(rtc: &pac::Rtc) {
        rtc.regs().ctl.modify(|r| r & !ctl::CMF);
        Self::wait_lwoff(rtc);
    }

    /// Sets 32-bit Unix epoch seconds counter.
    pub fn set_epoch(&mut self, epoch: u32) {
        // Ensure backup domain write access is enabled
        self.pmu.regs().ctl.modify(|r| r | pmu_ctl::BKPWEN);

        Self::enter_config(&self.rtc);
        self.rtc.regs().cnth.write((epoch >> 16) & 0xFFFF);
        self.rtc.regs().cntl.write(epoch & 0xFFFF);
        Self::exit_config(&self.rtc);

        self.synced = true;
    }

    /// Reads current 32-bit Unix epoch seconds counter.
    /// Uses glitch-free double-read to guarantee atomic consistency during roll-over.
    pub fn get_epoch(&self) -> u32 {
        for _ in 0..10 {
            let h1 = self.rtc.regs().cnth.read() & 0xFFFF;
            let l1 = self.rtc.regs().cntl.read() & 0xFFFF;
            let h2 = self.rtc.regs().cnth.read() & 0xFFFF;
            if h1 == h2 {
                return (h1 << 16) | l1;
            }
        }
        (self.rtc.regs().cnth.read() << 16) | (self.rtc.regs().cntl.read() & 0xFFFF)
    }

    /// Returns current date and time in UTC.
    pub fn get_datetime(&self) -> DateTime {
        epoch_to_datetime(self.get_epoch())
    }

    /// Returns true if RTC has been synchronized with host time.
    #[inline(always)]
    pub fn is_synced(&self) -> bool {
        self.synced || self.get_epoch() > 1_700_000_000
    }
}
