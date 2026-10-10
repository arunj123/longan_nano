pub mod theme;
pub mod screen_hero;
pub mod screen_graph;
pub mod screen_stats;
pub mod screen_histogram;
pub mod screen_big_digit;

pub use screen_hero::HeroScreen;
pub use screen_graph::{GraphScreen, TriggerMode, TriggerState};
pub use screen_stats::{SdStatus, StatsScreen};
pub use screen_histogram::HistogramScreen;
pub use screen_big_digit::BigDigitScreen;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScreenMode {
    Hero,      // Large 28px numeric display with load gauge bar and metric tiles
    Graph,     // Real-time oscilloscope with 137-sample history replotting
    Stats,     // Comprehensive session dashboard and DSP analytics
    Histogram, // 7-bin logarithmic current distribution profile
    BigDigit,  // Full-screen high-visibility bench meter mode
}
