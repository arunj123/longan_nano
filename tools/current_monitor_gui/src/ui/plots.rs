use egui::{Color32, RichText, Stroke, Ui};
use egui_plot::{Legend, Line, Plot, PlotBounds, PlotPoints};
use crate::model::TelemetryPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotLayoutMode {
    StackedMultiChannel, // 3 synchronized stacked sub-scopes (V, I, P)
    CurrentFocus,        // Large dedicated current scope
    CombinedOverlay,     // All on one graph
}

pub struct PlotOptions {
    pub layout_mode: PlotLayoutMode,
    pub time_window_secs: f64,
    pub is_frozen: bool,
    pub show_voltage: bool,
    pub show_current: bool,
    pub show_power: bool,
    pub show_fill: bool,
    pub reset_view_trigger: bool,
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            layout_mode: PlotLayoutMode::StackedMultiChannel,
            time_window_secs: 15.0,
            is_frozen: false,
            show_voltage: true,
            show_current: true,
            show_power: true,
            show_fill: true,
            reset_view_trigger: false,
        }
    }
}

pub fn render_waveform_plots(
    ui: &mut Ui,
    history: &[TelemetryPacket],
    options: &mut PlotOptions,
) {
    // Toolbar
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("WAVEFORM OSCILLOSCOPE").size(13.0).color(Color32::from_rgb(0, 230, 255)).strong());
        ui.separator();

        // Run / Freeze Toggle Button
        let (freeze_text, freeze_bg, freeze_fg) = if options.is_frozen {
            ("▶ RESUME", Color32::from_rgb(180, 110, 20), Color32::from_rgb(255, 240, 200))
        } else {
            ("⏸ FREEZE", Color32::from_rgb(30, 42, 60), Color32::from_rgb(180, 210, 240))
        };
        if ui.add(egui::Button::new(RichText::new(freeze_text).size(11.0).color(freeze_fg).strong()).fill(freeze_bg)).clicked() {
            options.is_frozen = !options.is_frozen;
        }

        if ui.button(RichText::new("↺ Reset Bounds").size(11.0)).clicked() {
            options.reset_view_trigger = true;
        }

        ui.separator();

        // Layout Mode Selector
        ui.selectable_value(&mut options.layout_mode, PlotLayoutMode::StackedMultiChannel, "Stacked 3-CH");
        ui.selectable_value(&mut options.layout_mode, PlotLayoutMode::CurrentFocus, "Current Focus");
        ui.selectable_value(&mut options.layout_mode, PlotLayoutMode::CombinedOverlay, "Overlay");

        ui.separator();

        // Time Window Buttons
        ui.label(RichText::new("Span:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
        for &w in &[5.0, 10.0, 15.0, 30.0, 60.0, 120.0] {
            let label = format!("{}s", w as i32);
            if ui.selectable_label((options.time_window_secs - w).abs() < 0.1, label).clicked() {
                options.time_window_secs = w;
                options.reset_view_trigger = true;
            }
        }

        ui.separator();
        ui.checkbox(&mut options.show_fill, "Fill Area");
    });

    ui.add_space(4.0);

    if history.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Awaiting telemetry stream for oscilloscope...").color(Color32::from_rgb(120, 140, 165)));
        });
        return;
    }

    let latest_t = history.last().map(|p| p.relative_secs).unwrap_or(0.0);
    let min_t = (latest_t - options.time_window_secs).max(0.0);
    let max_t = latest_t.max(min_t + 1.0);

    // Extract points in window
    let mut current_pts = Vec::with_capacity(1200);
    let mut voltage_pts = Vec::with_capacity(1200);
    let mut power_pts = Vec::with_capacity(1200);

    for p in history.iter().rev() {
        if p.relative_secs < min_t {
            break;
        }
        current_pts.push([p.relative_secs, p.current_ma()]);
        voltage_pts.push([p.relative_secs, p.voltage_v()]);
        power_pts.push([p.relative_secs, p.power_mw as f64]);
    }

    current_pts.reverse();
    voltage_pts.reverse();
    power_pts.reverse();

    // Compute live stats for current window
    let compute_stats = |pts: &[[f64; 2]]| -> (f64, f64, f64, f64) {
        if pts.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut sum = 0.0;
        for &p in pts {
            let v = p[1];
            if v < min { min = v; }
            if v > max { max = v; }
            sum += v;
        }
        let avg = sum / pts.len() as f64;
        let pk_pk = (max - min).max(0.0);
        (min, max, avg, pk_pk)
    };

    let (cur_min, cur_max, cur_avg, cur_pk) = compute_stats(&current_pts);
    let (volt_min, volt_max, volt_avg, volt_pk) = compute_stats(&voltage_pts);
    let (pwr_min, pwr_max, pwr_avg, pwr_pk) = compute_stats(&power_pts);

    // Dynamic auto-centering Y bounds
    let (y_c_min, y_c_max) = {
        let span = (cur_max - cur_min).max(20.0);
        let top = (cur_max + span * 0.15).max(10.0);
        let bot = (cur_min - span * 0.10).min(-2.0);
        (bot, top)
    };

    let (y_v_min, y_v_max) = {
        let span = (volt_max - volt_min).max(0.12);
        let mid = volt_avg;
        (mid - span * 0.65, mid + span * 0.65)
    };

    let (y_p_min, y_p_max) = {
        let span = (pwr_max - pwr_min).max(50.0);
        let top = (pwr_max + span * 0.15).max(25.0);
        let bot = (pwr_min - span * 0.10).min(-5.0);
        (bot, top)
    };

    let cur_col = Color32::from_rgb(0, 240, 255);
    let volt_col = Color32::from_rgb(46, 215, 110);
    let pwr_col = Color32::from_rgb(245, 185, 45);

    let should_force_bounds = !options.is_frozen || options.reset_view_trigger;
    options.reset_view_trigger = false;

    match options.layout_mode {
        PlotLayoutMode::StackedMultiChannel => {
            let avail_h = ui.available_height().max(240.0);
            let strip_h = (avail_h - 24.0) / 3.0;

            // 1. Current Channel Plot (mA)
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("CH1: CURRENT (mA)").color(cur_col).size(11.0).strong());
                    if let Some(last) = current_pts.last() {
                        ui.label(RichText::new(format!("Live: {:.1} mA", last[1])).color(cur_col).size(11.0).strong());
                    }
                    ui.label(RichText::new(format!("| Min: {:.1} | Max: {:.1} | Avg: {:.1} | Pk-Pk: {:.1} mA", cur_min, cur_max, cur_avg, cur_pk)).color(Color32::from_rgb(130, 155, 185)).size(10.0));
                });
                Plot::new("scope_current_strip")
                    .height(strip_h)
                    .link_axis("time_sync_axis", true, false)
                    .show_x(false)
                    .show_y(true)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .show(ui, |plot_ui| {
                        if should_force_bounds {
                            plot_ui.set_plot_bounds(PlotBounds::from_min_max([min_t, y_c_min], [max_t, y_c_max]));
                        }
                        let mut line = Line::new(PlotPoints::new(current_pts.clone()))
                            .color(cur_col)
                            .width(2.0)
                            .name("Current (mA)");
                        if options.show_fill {
                            line = line.fill(0.0);
                        }
                        plot_ui.line(line);
                    });
            });

            // 2. Bus Voltage Channel Plot (V)
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("CH2: VOLTAGE (V)").color(volt_col).size(11.0).strong());
                    if let Some(last) = voltage_pts.last() {
                        ui.label(RichText::new(format!("Live: {:.3} V", last[1])).color(volt_col).size(11.0).strong());
                    }
                    ui.label(RichText::new(format!("| Min: {:.3} | Max: {:.3} | Avg: {:.3} | Ripple: {:.1} mV", volt_min, volt_max, volt_avg, volt_pk * 1000.0)).color(Color32::from_rgb(130, 155, 185)).size(10.0));
                });
                Plot::new("scope_voltage_strip")
                    .height(strip_h)
                    .link_axis("time_sync_axis", true, false)
                    .show_x(false)
                    .show_y(true)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .show(ui, |plot_ui| {
                        if should_force_bounds {
                            plot_ui.set_plot_bounds(PlotBounds::from_min_max([min_t, y_v_min], [max_t, y_v_max]));
                        }
                        let line = Line::new(PlotPoints::new(voltage_pts.clone()))
                            .color(volt_col)
                            .width(2.0)
                            .name("Voltage (V)");
                        plot_ui.line(line);
                    });
            });

            // 3. Power Channel Plot (mW)
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("CH3: POWER (mW)").color(pwr_col).size(11.0).strong());
                    if let Some(last) = power_pts.last() {
                        ui.label(RichText::new(format!("Live: {:.1} mW", last[1])).color(pwr_col).size(11.0).strong());
                    }
                    ui.label(RichText::new(format!("| Min: {:.1} | Max: {:.1} | Avg: {:.1} | Pk-Pk: {:.1} mW", pwr_min, pwr_max, pwr_avg, pwr_pk)).color(Color32::from_rgb(130, 155, 185)).size(10.0));
                });
                Plot::new("scope_power_strip")
                    .height(strip_h)
                    .link_axis("time_sync_axis", true, false)
                    .show_x(true)
                    .show_y(true)
                    .x_axis_label("Elapsed Time (s)")
                    .allow_zoom(true)
                    .allow_drag(true)
                    .show(ui, |plot_ui| {
                        if should_force_bounds {
                            plot_ui.set_plot_bounds(PlotBounds::from_min_max([min_t, y_p_min], [max_t, y_p_max]));
                        }
                        let mut line = Line::new(PlotPoints::new(power_pts.clone()))
                            .color(pwr_col)
                            .width(2.0)
                            .name("Power (mW)");
                        if options.show_fill {
                            line = line.fill(0.0);
                        }
                        plot_ui.line(line);
                    });
            });
        }

        PlotLayoutMode::CurrentFocus => {
            let avail_h = ui.available_height().max(180.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("PRIMARY CURRENT SCOPE (mA)").color(cur_col).size(11.0).strong());
                if let Some(last) = current_pts.last() {
                    ui.label(RichText::new(format!("Live: {:.1} mA", last[1])).color(cur_col).size(11.0).strong());
                }
                ui.label(RichText::new(format!("| Min: {:.1} | Max: {:.1} | Avg: {:.1} | Pk-Pk: {:.1} mA", cur_min, cur_max, cur_avg, cur_pk)).color(Color32::from_rgb(130, 155, 185)).size(10.0));
            });
            Plot::new("scope_current_focus")
                .height(avail_h - 22.0)
                .show_x(true)
                .show_y(true)
                .x_axis_label("Elapsed Time (s)")
                .y_axis_label("Current (mA)")
                .allow_zoom(true)
                .allow_drag(true)
                .show(ui, |plot_ui| {
                    if should_force_bounds {
                        plot_ui.set_plot_bounds(PlotBounds::from_min_max([min_t, y_c_min], [max_t, y_c_max]));
                    }
                    let mut line = Line::new(PlotPoints::new(current_pts))
                        .color(cur_col)
                        .width(2.2)
                        .name("Current (mA)");
                    if options.show_fill {
                        line = line.fill(0.0);
                    }
                    plot_ui.line(line);
                });
        }

        PlotLayoutMode::CombinedOverlay => {
            let avail_h = ui.available_height().max(180.0);
            Plot::new("scope_combined_overlay")
                .height(avail_h)
                .legend(Legend::default().text_style(egui::TextStyle::Small))
                .show_x(true)
                .show_y(true)
                .x_axis_label("Elapsed Time (s)")
                .y_axis_label("Magnitude")
                .allow_zoom(true)
                .allow_drag(true)
                .show(ui, |plot_ui| {
                    if should_force_bounds {
                        let top = y_c_max.max(y_p_max).max(50.0);
                        plot_ui.set_plot_bounds(PlotBounds::from_min_max([min_t, -5.0], [max_t, top]));
                    }
                    if options.show_current {
                        let mut line = Line::new(PlotPoints::new(current_pts)).color(cur_col).width(2.0).name("Current (mA)");
                        if options.show_fill {
                            line = line.fill(0.0);
                        }
                        plot_ui.line(line);
                    }
                    if options.show_voltage {
                        plot_ui.line(Line::new(PlotPoints::new(voltage_pts)).color(volt_col).width(2.0).name("Voltage (V)"));
                    }
                    if options.show_power {
                        plot_ui.line(Line::new(PlotPoints::new(power_pts)).color(pwr_col).width(2.0).name("Power (mW)"));
                    }
                });
        }
    }
}
