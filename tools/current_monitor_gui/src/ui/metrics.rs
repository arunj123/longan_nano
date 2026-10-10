use egui::{Color32, Frame, RichText, Ui};
use crate::model::{LiveStats, TelemetryPacket};

pub fn render_metrics_cards(ui: &mut Ui, latest: Option<&TelemetryPacket>, stats: &LiveStats) {
    ui.horizontal(|ui| {
        // 1. Hero Current Metric Card
        Frame::canvas(ui.style()).fill(Color32::from_rgb(15, 23, 35)).inner_margin(12.0).show(ui, |ui| {
            ui.set_min_width(220.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("CURRENT").color(Color32::from_rgb(140, 165, 195)).small());
                let (cur_str, cur_unit, cur_col) = if let Some(p) = latest {
                    let c = p.current_ma();
                    let col = if p.is_alert() {
                        Color32::from_rgb(255, 60, 60)
                    } else if c.abs() > 500.0 {
                        Color32::from_rgb(245, 160, 40)
                    } else if c.abs() > 50.0 {
                        Color32::from_rgb(50, 230, 120)
                    } else {
                        Color32::from_rgb(80, 220, 255)
                    };

                    if c.abs() >= 1000.0 {
                        (format!("{:.3}", c / 1000.0), "A", col)
                    } else {
                        (format!("{:.1}", c), "mA", col)
                    }
                } else {
                    ("---.-".to_string(), "mA", Color32::from_rgb(100, 110, 130))
                };

                ui.horizontal(|ui| {
                    ui.label(RichText::new(cur_str).size(34.0).color(cur_col).strong());
                    ui.label(RichText::new(cur_unit).size(16.0).color(cur_col));
                });

                let min_c = if stats.c_min_ma.is_finite() { stats.c_min_ma } else { 0.0 };
                let max_c = if stats.c_max_ma.is_finite() { stats.c_max_ma } else { 0.0 };
                ui.label(
                    RichText::new(format!("Min: {:.1} | Max: {:.1} mA", min_c, max_c))
                        .color(Color32::from_rgb(120, 140, 165))
                        .small(),
                );
            });
        });

        // 2. Bus Voltage Metric Card
        Frame::canvas(ui.style()).fill(Color32::from_rgb(15, 23, 35)).inner_margin(12.0).show(ui, |ui| {
            ui.set_min_width(180.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("VOLTAGE").color(Color32::from_rgb(140, 165, 195)).small());
                let (v_str, v_col) = if let Some(p) = latest {
                    (format!("{:.3} V", p.voltage_v()), Color32::from_rgb(50, 220, 120))
                } else {
                    ("---.--- V".to_string(), Color32::from_rgb(100, 110, 130))
                };

                ui.label(RichText::new(v_str).size(26.0).color(v_col).strong());
                let min_v = if stats.v_min_mv < u16::MAX { stats.v_min_mv as f64 / 1000.0 } else { 0.0 };
                let max_v = stats.v_max_mv as f64 / 1000.0;
                ui.label(
                    RichText::new(format!("Min: {:.3} | Max: {:.3} V", min_v, max_v))
                        .color(Color32::from_rgb(120, 140, 165))
                        .small(),
                );
            });
        });

        // 3. Power Metric Card
        Frame::canvas(ui.style()).fill(Color32::from_rgb(15, 23, 35)).inner_margin(12.0).show(ui, |ui| {
            ui.set_min_width(180.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("POWER").color(Color32::from_rgb(140, 165, 195)).small());
                let (p_str, p_col) = if let Some(p) = latest {
                    let mw = p.power_mw as f64;
                    if mw >= 1000.0 {
                        (format!("{:.3} W", mw / 1000.0), Color32::from_rgb(235, 180, 55))
                    } else {
                        (format!("{:.1} mW", mw), Color32::from_rgb(235, 180, 55))
                    }
                } else {
                    ("---.- mW".to_string(), Color32::from_rgb(100, 110, 130))
                };

                ui.label(RichText::new(p_str).size(26.0).color(p_col).strong());
                let peak_mw = stats.p_max_mw as f64;
                ui.label(
                    RichText::new(format!("Peak: {:.1} mW", peak_mw))
                        .color(Color32::from_rgb(120, 140, 165))
                        .small(),
                );
            });
        });

        // 4. Impedance & Accumulator Card
        Frame::canvas(ui.style()).fill(Color32::from_rgb(15, 23, 35)).inner_margin(12.0).show(ui, |ui| {
            ui.set_min_width(220.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("IMPEDANCE & ACCUMULATORS").color(Color32::from_rgb(140, 165, 195)).small());
                let r_str = if let Some(p) = latest {
                    if let Some(r) = p.resistance_ohms() {
                        if r < 1000.0 {
                            format!("{:.1} Ω", r)
                        } else if r < 1_000_000.0 {
                            format!("{:.2} kΩ", r / 1000.0)
                        } else {
                            format!("{:.2} MΩ", r / 1_000_000.0)
                        }
                    } else {
                        "---.- Ω".to_string()
                    }
                } else {
                    "---.- Ω".to_string()
                };

                ui.label(RichText::new(format!("R: {}", r_str)).size(18.0).color(Color32::from_rgb(200, 220, 240)).strong());
                ui.label(
                    RichText::new(format!("Energy: {:.2} mWh | Charge: {:.2} mAh", stats.energy_mwh, stats.charge_mah))
                        .color(Color32::from_rgb(140, 175, 215))
                        .small(),
                );
            });
        });

        // 5. Diagnostics & Status Badges
        Frame::canvas(ui.style()).fill(Color32::from_rgb(15, 23, 35)).inner_margin(12.0).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("HARDWARE STATUS").color(Color32::from_rgb(140, 165, 195)).small());
                ui.horizontal_wrapped(|ui| {
                    if let Some(p) = latest {
                        let badge = |ui: &mut Ui, label: &str, active: bool, color: Color32| {
                            let text_color = if active { color } else { Color32::from_rgb(60, 75, 95) };
                            ui.label(RichText::new(format!("[{}]", label)).color(text_color).strong());
                        };

                        badge(ui, "ONLINE", p.is_online(), Color32::from_rgb(50, 220, 120));
                        badge(ui, "REV", p.is_reverse(), Color32::from_rgb(240, 180, 40));
                        badge(ui, "OVF", p.is_overflow(), Color32::from_rgb(255, 60, 60));
                        badge(ui, "SD-LOG", p.is_sd_logging(), Color32::from_rgb(80, 200, 255));
                        badge(ui, "ALERT", p.is_alert(), Color32::from_rgb(255, 40, 40));
                        ui.label(RichText::new(format!("Seq #{}", p.seq)).color(Color32::from_rgb(120, 140, 165)).small());
                    } else {
                        ui.label(RichText::new("Awaiting telemetry stream...").color(Color32::from_rgb(120, 140, 165)).small());
                    }
                });
            });
        });
    });
}
