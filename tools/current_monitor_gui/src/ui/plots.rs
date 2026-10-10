use egui::{Color32, RichText, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use crate::model::TelemetryPacket;

pub struct PlotOptions {
    pub time_window_secs: f64,
    pub is_frozen: bool,
    pub show_voltage: bool,
    pub show_current: bool,
    pub show_power: bool,
    pub autoscale_y: bool,
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            time_window_secs: 15.0,
            is_frozen: false,
            show_voltage: true,
            show_current: true,
            show_power: true,
            autoscale_y: true,
        }
    }
}

pub fn render_waveform_plots(
    ui: &mut Ui,
    history: &[TelemetryPacket],
    options: &mut PlotOptions,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("WAVEFORM OSCILLOSCOPE").strong());
        ui.separator();

        // Run / Freeze Toggle
        let freeze_text = if options.is_frozen { "▶ Resume Sweep" } else { "⏸ Freeze Sweep" };
        let freeze_col = if options.is_frozen { Color32::from_rgb(255, 180, 50) } else { Color32::from_rgb(180, 190, 205) };
        if ui.button(RichText::new(freeze_text).color(freeze_col)).clicked() {
            options.is_frozen = !options.is_frozen;
        }

        ui.separator();

        // Time Window Buttons
        ui.label("Window:");
        for &w in &[5.0, 10.0, 15.0, 30.0, 60.0, 120.0] {
            let label = format!("{}s", w as i32);
            if ui.selectable_label((options.time_window_secs - w).abs() < 0.1, label).clicked() {
                options.time_window_secs = w;
            }
        }

        ui.separator();

        // Channel Visibility Toggles
        ui.checkbox(&mut options.show_current, RichText::new("Current (mA)").color(Color32::from_rgb(0, 240, 255)));
        ui.checkbox(&mut options.show_voltage, RichText::new("Voltage (V)").color(Color32::from_rgb(46, 160, 67)));
        ui.checkbox(&mut options.show_power, RichText::new("Power (mW)").color(Color32::from_rgb(227, 179, 65)));
    });

    if history.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("Awaiting telemetry points for plot rendering...");
        });
        return;
    }

    let latest_t = history.last().map(|p| p.relative_secs).unwrap_or(0.0);
    let min_t = (latest_t - options.time_window_secs).max(0.0);

    // Filter points in window
    let mut current_pts = Vec::new();
    let mut voltage_pts = Vec::new();
    let mut power_pts = Vec::new();

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

    let plot = Plot::new("oscilloscope_plot")
        .legend(Legend::default().text_style(egui::TextStyle::Small))
        .show_x(true)
        .show_y(true)
        .allow_zoom(true)
        .allow_drag(true)
        .x_axis_label("Time (s)")
        .y_axis_label("Value")
        .include_x(min_t)
        .include_x(latest_t.max(min_t + 1.0));

    plot.show(ui, |plot_ui| {
        if options.show_current {
            plot_ui.line(
                Line::new(PlotPoints::new(current_pts))
                    .color(Color32::from_rgb(0, 240, 255))
                    .width(1.8)
                    .name("Current (mA)"),
            );
        }

        if options.show_voltage {
            plot_ui.line(
                Line::new(PlotPoints::new(voltage_pts))
                    .color(Color32::from_rgb(46, 160, 67))
                    .width(1.8)
                    .name("Voltage (V)"),
            );
        }

        if options.show_power {
            plot_ui.line(
                Line::new(PlotPoints::new(power_pts))
                    .color(Color32::from_rgb(227, 179, 65))
                    .width(1.8)
                    .name("Power (mW)"),
            );
        }
    });
}
