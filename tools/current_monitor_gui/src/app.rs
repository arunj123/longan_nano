use std::time::{Duration, Instant};
use egui::{CentralPanel, Color32, Frame, Margin, RichText, Rounding, SidePanel, Stroke, TopBottomPanel, Visuals};

use crate::model::{HidCommand, LiveStats, TelemetryPacket};
use crate::transport::{TransportKind, TransportManager};
use crate::ui::{
    render_controls_panel, render_header, render_metrics_cards, render_recording_panel,
    render_waveform_plots, render_3d_viewport, CsvRecorder, HardwareControlState, PlotOptions,
    View3DState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainViewTab {
    SplitView,
    Dashboard2D,
    Viewport3D,
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
    show_side_panel: bool,
    start_instant: Instant,
    last_frame_instant: Instant,
    theme_initialized: bool,
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
            show_side_panel: false,
            start_instant: Instant::now(),
            last_frame_instant: Instant::now(),
            theme_initialized: false,
        }
    }

    fn init_theme(&mut self, ctx: &egui::Context) {
        if self.theme_initialized {
            return;
        }
        let mut visuals = Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(11, 15, 23);
        visuals.window_fill = Color32::from_rgb(14, 19, 30);
        visuals.extreme_bg_color = Color32::from_rgb(8, 11, 18);
        visuals.override_text_color = Some(Color32::from_rgb(220, 230, 245));
        visuals.selection.bg_fill = Color32::from_rgb(0, 160, 220);
        visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0, 240, 255));
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(15, 21, 32);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(28, 38, 55));
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 28, 42);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(28, 40, 60);
        visuals.widgets.hovered.rounding = Rounding::same(6.0);
        visuals.widgets.active.bg_fill = Color32::from_rgb(0, 140, 190);
        visuals.widgets.active.rounding = Rounding::same(6.0);
        ctx.set_visuals(visuals);
        self.theme_initialized = true;
    }
}

impl eframe::App for CurrentMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.init_theme(ctx);

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
        TopBottomPanel::top("top_header_panel")
            .frame(
                Frame::none()
                    .fill(Color32::from_rgb(11, 15, 23))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(24, 34, 50)))
                    .inner_margin(Margin::symmetric(14.0, 8.0)),
            )
            .show(ctx, |ui| {
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

        // 5. Collapsible Side Panel (Controls & Recording)
        if self.show_side_panel {
            SidePanel::right("controls_side_panel")
                .resizable(true)
                .default_width(300.0)
                .frame(
                    Frame::none()
                        .fill(Color32::from_rgb(12, 17, 26))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(26, 36, 52)))
                        .inner_margin(Margin::same(12.0)),
                )
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        render_controls_panel(ui, &mut self.control_state, &mut send_cmd);
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);
                        render_recording_panel(ui, &mut self.csv_recorder, &self.history);
                    });
                });
        }

        // 6. Central Panel
        CentralPanel::default().show(ctx, |ui| {
            let latest = self.history.last();

            // Hero Metric Cards
            render_metrics_cards(ui, latest, &self.stats);
            ui.add_space(8.0);

            // View Mode Tab Bar + Side Panel Toggle
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, MainViewTab::SplitView, "🎛️ Split View (Scope + 3D Phase Space)");
                ui.selectable_value(&mut self.active_tab, MainViewTab::Dashboard2D, "📊 Oscilloscope Fullscreen");
                ui.selectable_value(&mut self.active_tab, MainViewTab::Viewport3D, "📈 3D Phase Space Fullscreen");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let side_btn_text = if self.show_side_panel { "⚙️ Hide Controls" } else { "⚙️ Show Controls" };
                    let side_col = if self.show_side_panel { Color32::from_rgb(0, 230, 255) } else { Color32::from_rgb(160, 180, 205) };
                    if ui.button(RichText::new(side_btn_text).color(side_col).size(11.0).strong()).clicked() {
                        self.show_side_panel = !self.show_side_panel;
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            // Main Content Area
            match self.active_tab {
                MainViewTab::Dashboard2D => {
                    render_waveform_plots(ui, &self.history, &mut self.plot_options);
                }

                MainViewTab::Viewport3D => {
                    render_3d_viewport(
                        ui,
                        &mut self.view_3d,
                        latest,
                        &self.history,
                        anim_time,
                    );
                }

                MainViewTab::SplitView => {
                    ui.columns(2, |cols| {
                        // Left Column (52%): Oscilloscope Plot
                        render_waveform_plots(&mut cols[0], &self.history, &mut self.plot_options);

                        // Right Column (48%): 3D Graphics Viewport
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

        // 7. Dispatch any outbound hardware commands
        if let Some(cmd) = send_cmd {
            if matches!(cmd, HidCommand::TareZero) {
                self.stats.reset();
            }
            self.transport.send_command(cmd);
        }

        // 8. Request smooth 60 FPS repaints
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
