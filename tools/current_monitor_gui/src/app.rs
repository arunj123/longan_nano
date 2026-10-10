use std::time::{Duration, Instant};
use egui::{CentralPanel, TopBottomPanel};

use crate::model::{HidCommand, LiveStats, TelemetryPacket};
use crate::transport::{TransportKind, TransportManager};
use crate::ui::{
    render_controls_panel, render_header, render_metrics_cards, render_recording_panel,
    render_waveform_plots, render_3d_viewport, CsvRecorder, HardwareControlState, PlotOptions,
    View3DState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainViewTab {
    Dashboard2D,
    Viewport3D,
    SplitView,
}

pub struct CurrentMonitorApp {
    transport: TransportManager,
    history: Vec<TelemetryPacket>,
    max_history_len: usize,
    stats: LiveStats,
    plot_options: PlotOptions,
    control_state: HardwareControlState,
    csv_recorder: CsvRecorder,
    view_3d: View3DState,
    active_tab: MainViewTab,
    start_instant: Instant,
    last_frame_instant: Instant,
}

impl CurrentMonitorApp {
    pub fn new(initial_transport: TransportKind) -> Self {
        Self {
            transport: TransportManager::new(initial_transport),
            history: Vec::with_capacity(5000),
            max_history_len: 10_000,
            stats: LiveStats::new(),
            plot_options: PlotOptions::default(),
            control_state: HardwareControlState::default(),
            csv_recorder: CsvRecorder::default(),
            view_3d: View3DState::default(),
            active_tab: MainViewTab::SplitView,
            start_instant: Instant::now(),
            last_frame_instant: Instant::now(),
        }
    }
}

impl eframe::App for CurrentMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_instant).as_secs_f32();
        self.last_frame_instant = now;
        let anim_time = now.duration_since(self.start_instant).as_secs_f64();

        // 1. Ingest all pending telemetry packets from transport thread
        let new_packets = self.transport.poll_all_packets();
        for pkt in &new_packets {
            self.stats.update(pkt);
            self.csv_recorder.write_packet(pkt);

            if !self.plot_options.is_frozen {
                self.history.push(*pkt);
                if self.history.len() > self.max_history_len {
                    self.history.drain(0..100);
                }
            }
        }

        // 2. Update 3D animation
        self.view_3d.renderer.update_animation(dt);

        let mut switch_transport = None;
        let mut send_cmd = None;
        let mut toggle_recording = false;

        // 3. Top Header Bar
        TopBottomPanel::top("top_header_panel").show(ctx, |ui| {
            render_header(
                ui,
                self.transport.is_connected(),
                &self.transport.status_text(),
                &self.transport.kind,
                &mut switch_transport,
                &mut send_cmd,
                self.csv_recorder.is_recording,
                self.csv_recorder.samples_written,
                &mut toggle_recording,
            );
        });

        // 4. Handle Transport Switch or Recording Toggle
        if let Some(new_kind) = switch_transport {
            self.transport = TransportManager::new(new_kind);
        }

        if toggle_recording {
            if self.csv_recorder.is_recording {
                self.csv_recorder.stop();
            } else {
                let _ = self.csv_recorder.start();
            }
        }

        // 5. Central Panel
        CentralPanel::default().show(ctx, |ui| {
            let latest = self.history.last();

            // Hero Metric Cards
            render_metrics_cards(ui, latest, &self.stats);
            ui.add_space(6.0);

            // View Mode Tab Bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, MainViewTab::SplitView, "🎛️ Split View (2D + 3D)");
                ui.selectable_value(&mut self.active_tab, MainViewTab::Dashboard2D, "📊 Oscilloscope & Plots (2D)");
                ui.selectable_value(&mut self.active_tab, MainViewTab::Viewport3D, "📦 3D Graphics Viewport (3D)");
            });
            ui.separator();

            // Main Content Area
            match self.active_tab {
                MainViewTab::Dashboard2D => {
                    ui.columns(2, |cols| {
                        // Left Column (70%): Plots
                        render_waveform_plots(&mut cols[0], &self.history, &mut self.plot_options);

                        // Right Column (30%): Controls & Recording
                        cols[1].vertical(|ui| {
                            render_controls_panel(ui, &mut self.control_state, &mut send_cmd);
                            ui.separator();
                            render_recording_panel(ui, &mut self.csv_recorder, &self.history);
                        });
                    });
                }

                MainViewTab::Viewport3D => {
                    ui.columns(2, |cols| {
                        // Left Column (70%): 3D Viewport
                        render_3d_viewport(
                            &mut cols[0],
                            &mut self.view_3d,
                            latest,
                            &self.history,
                            anim_time,
                        );

                        // Right Column (30%): Controls & Recording
                        cols[1].vertical(|ui| {
                            render_controls_panel(ui, &mut self.control_state, &mut send_cmd);
                            ui.separator();
                            render_recording_panel(ui, &mut self.csv_recorder, &self.history);
                        });
                    });
                }

                MainViewTab::SplitView => {
                    ui.columns(2, |cols| {
                        // Left Column: 2D Waveform Plot
                        render_waveform_plots(&mut cols[0], &self.history, &mut self.plot_options);

                        // Right Column: Interactive 3D Viewport
                        render_3d_viewport(
                            &mut cols[1],
                            &mut self.view_3d,
                            latest,
                            &self.history,
                            anim_time,
                        );
                    });
                }
            }
        });

        // 6. Dispatch any outbound hardware commands
        if let Some(cmd) = send_cmd {
            if matches!(cmd, HidCommand::TareZero) {
                self.stats.reset();
            }
            self.transport.send_command(cmd);
        }

        // 7. Request smooth 60 FPS repaints
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
