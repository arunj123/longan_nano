pub mod theme;
pub mod screen_hero;
pub mod screen_graph;
pub mod screen_stats;

pub use screen_hero::HeroScreen;
pub use screen_graph::GraphScreen;
pub use screen_stats::{SdStatus, StatsScreen};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ScreenMode {
    Hero,  // Large 28px numeric display with load gauge bar and metric tiles
    Graph, // Real-time oscilloscope with 137-sample history replotting
    Stats, // Comprehensive session dashboard and DSP analytics
}
