pub mod theme;
pub mod screen_hero;
pub mod screen_graph;
pub mod screen_stats;
pub mod screen_histogram;
pub mod screen_big_digit;
pub mod screen_usbdrive;

pub use screen_hero::HeroScreen;
pub use screen_graph::{GraphScreen, TriggerMode, TriggerState};
pub use screen_stats::{SdStatus, StatsScreen};
pub use screen_histogram::HistogramScreen;
pub use screen_big_digit::BigDigitScreen;
pub use screen_usbdrive::{UsbDriveScreen, UsbDriveState};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScreenMode {
    Hero,      // Large 28px numeric display with load gauge bar and metric tiles
    Graph,     // Real-time oscilloscope with 137-sample history replotting
    Stats,     // Comprehensive session dashboard and DSP analytics
    Histogram, // 7-bin logarithmic current distribution profile
    BigDigit,  // Full-screen high-visibility bench meter mode
    UsbDrive,  // Dedicated USB Mass Storage Class drive mode
}

#[allow(dead_code)]
impl ScreenMode {
    /// Total number of distinct operational display screens
    pub const COUNT: usize = 6;

    /// Complete ordered array of all screen modes
    pub const ALL: [ScreenMode; Self::COUNT] = [
        ScreenMode::Hero,
        ScreenMode::Graph,
        ScreenMode::Stats,
        ScreenMode::Histogram,
        ScreenMode::BigDigit,
        ScreenMode::UsbDrive,
    ];

    /// Zero-based numerical index corresponding to NVRAM BKP and command dispatch
    #[inline]
    pub const fn index(self) -> usize {
        match self {
            ScreenMode::Hero => 0,
            ScreenMode::Graph => 1,
            ScreenMode::Stats => 2,
            ScreenMode::Histogram => 3,
            ScreenMode::BigDigit => 4,
            ScreenMode::UsbDrive => 5,
        }
    }

    /// Construct a ScreenMode variant from an integer index (0..5)
    #[inline]
    pub const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(ScreenMode::Hero),
            1 => Some(ScreenMode::Graph),
            2 => Some(ScreenMode::Stats),
            3 => Some(ScreenMode::Histogram),
            4 => Some(ScreenMode::BigDigit),
            5 => Some(ScreenMode::UsbDrive),
            _ => None,
        }
    }

    /// Advance to the next screen mode (wraps from UsbDrive to Hero)
    #[inline]
    pub const fn next(self) -> Self {
        match self {
            ScreenMode::Hero => ScreenMode::Graph,
            ScreenMode::Graph => ScreenMode::Stats,
            ScreenMode::Stats => ScreenMode::Histogram,
            ScreenMode::Histogram => ScreenMode::BigDigit,
            ScreenMode::BigDigit => ScreenMode::UsbDrive,
            ScreenMode::UsbDrive => ScreenMode::Hero,
        }
    }

    /// Regress to the previous screen mode (wraps from Hero to UsbDrive)
    #[inline]
    pub const fn prev(self) -> Self {
        match self {
            ScreenMode::Hero => ScreenMode::UsbDrive,
            ScreenMode::Graph => ScreenMode::Hero,
            ScreenMode::Stats => ScreenMode::Graph,
            ScreenMode::Histogram => ScreenMode::Stats,
            ScreenMode::BigDigit => ScreenMode::Histogram,
            ScreenMode::UsbDrive => ScreenMode::BigDigit,
        }
    }

    /// Full human-readable display title for top status banner
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            ScreenMode::Hero => "HERO",
            ScreenMode::Graph => "OSCILLOSCOPE",
            ScreenMode::Stats => "STATISTICS",
            ScreenMode::Histogram => "HISTOGRAM",
            ScreenMode::BigDigit => "BIG DIGIT",
            ScreenMode::UsbDrive => "USB DRIVE",
        }
    }

    /// Short compact identifier used in serial dumps, CLI, and telemetry
    #[inline]
    pub const fn short_name(self) -> &'static str {
        match self {
            ScreenMode::Hero => "HERO",
            ScreenMode::Graph => "GRAPH",
            ScreenMode::Stats => "STATS",
            ScreenMode::Histogram => "HISTO",
            ScreenMode::BigDigit => "BIG",
            ScreenMode::UsbDrive => "USB_DRIVE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_mode_constants_and_round_trip() {
        assert_eq!(ScreenMode::COUNT, 6);
        assert_eq!(ScreenMode::ALL.len(), ScreenMode::COUNT);

        for (i, &mode) in ScreenMode::ALL.iter().enumerate() {
            assert_eq!(mode.index(), i);
            assert_eq!(ScreenMode::from_index(i), Some(mode));
        }

        assert_eq!(ScreenMode::from_index(6), None);
        assert_eq!(ScreenMode::from_index(99), None);
    }

    #[test]
    fn test_screen_mode_next_prev_cycling() {
        for (i, &mode) in ScreenMode::ALL.iter().enumerate() {
            let next_expected = ScreenMode::ALL[(i + 1) % ScreenMode::COUNT];
            let prev_expected = ScreenMode::ALL[(i + ScreenMode::COUNT - 1) % ScreenMode::COUNT];
            assert_eq!(mode.next(), next_expected);
            assert_eq!(mode.prev(), prev_expected);
        }
    }

    #[test]
    fn test_screen_mode_names() {
        assert_eq!(ScreenMode::Hero.name(), "HERO");
        assert_eq!(ScreenMode::Graph.name(), "OSCILLOSCOPE");
        assert_eq!(ScreenMode::Stats.name(), "STATISTICS");
        assert_eq!(ScreenMode::Histogram.name(), "HISTOGRAM");
        assert_eq!(ScreenMode::BigDigit.name(), "BIG DIGIT");
        assert_eq!(ScreenMode::UsbDrive.name(), "USB DRIVE");

        assert_eq!(ScreenMode::Hero.short_name(), "HERO");
        assert_eq!(ScreenMode::Graph.short_name(), "GRAPH");
        assert_eq!(ScreenMode::Stats.short_name(), "STATS");
        assert_eq!(ScreenMode::Histogram.short_name(), "HISTO");
        assert_eq!(ScreenMode::BigDigit.short_name(), "BIG");
        assert_eq!(ScreenMode::UsbDrive.short_name(), "USB_DRIVE");
    }
}
