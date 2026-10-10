pub mod header;
pub mod metrics;
pub mod plots;
pub mod controls;
pub mod recording;
pub mod view3d;

pub use header::render_header;
pub use metrics::render_metrics_cards;
pub use plots::{render_waveform_plots, PlotOptions};
pub use controls::{render_controls_panel, HardwareControlState};
pub use recording::{render_recording_panel, CsvRecorder};
pub use view3d::{render_3d_viewport, View3DState};
