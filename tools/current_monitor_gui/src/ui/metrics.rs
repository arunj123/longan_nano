use egui::{Color32, Frame, Margin, RichText, Rounding, Stroke, Ui};
use crate::model::{LiveStats, TelemetryPacket};

pub fn render_metrics_cards(ui: &mut Ui, latest: Option<&TelemetryPacket>, stats: &LiveStats) {
    let card_frame = |border_color: Color32| {
        Frame::none()
            .fill(Color32::from_rgb(16, 22, 34))
            .stroke(Stroke::new(1.0, border_color))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
    };

    ui.horizontal(|ui| {
        // ==========================================
        // 1. HERO CURRENT CARD
        // ==========================================
        let cur_val = latest.map(|p| p.current_ma()).unwrap_or(0.0);
        let is_alert = latest.map(|p| p.is_alert()).unwrap_or(false);

        let (cur_str, cur_unit, cur_col) = if latest.is_some() {
            let col = if is_alert {
                Color32::from_rgb(255, 65, 65)
            } else if cur_val.abs() > 500.0 {
                Color32::from_rgb(245, 160, 40)
            } else if cur_val.abs() > 50.0 {
                Color32::from_rgb(45, 225, 130)
            } else {
                Color32::from_rgb(0, 235, 255)
            };

            if cur_val.abs() >= 1000.0 {
                (format!("{:.3}", cur_val / 1000.0), "A", col)
            } else {
                (format!("{:.1}", cur_val), "mA", col)
            }
        } else {
            ("---.-".to_string(), "mA", Color32::from_rgb(90, 105, 125))
        };

        card_frame(Color32::from_rgba_unmultiplied(0, 235, 255, 70)).show(ui, |ui| {
            ui.set_width(225.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("CURRENT").color(Color32::from_rgb(130, 155, 185)).size(11.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("[ {} ]", cur_unit))
                                .color(cur_col)
                                .size(11.0)
                                .strong(),
                        );
                    });
                });

                ui.add_space(2.0);
                ui.label(RichText::new(cur_str).size(36.0).color(cur_col).strong());

                // Mini horizontal load bar gauge
                let fill = (cur_val.abs() as f32 / 1000.0).clamp(0.02, 1.0);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(220.0, 4.0), egui::Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, Rounding::same(2.0), Color32::from_rgb(25, 35, 50));
                let filled_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * fill, rect.height()));
                painter.rect_filled(filled_rect, Rounding::same(2.0), cur_col);

                ui.add_space(4.0);
                let min_c = if stats.c_min_ma.is_finite() { stats.c_min_ma } else { 0.0 };
                let max_c = if stats.c_max_ma.is_finite() { stats.c_max_ma } else { 0.0 };
                let avg_c = stats.avg_current_ma();
                ui.label(
                    RichText::new(format!("Min: {:.1} | Avg: {:.1} | Max: {:.1} mA", min_c, avg_c, max_c))
                        .color(Color32::from_rgb(120, 140, 165))
                        .size(10.0),
                );
            });
        });

        // ==========================================
        // 2. BUS VOLTAGE CARD
        // ==========================================
        let v_val = latest.map(|p| p.voltage_v()).unwrap_or(0.0);
        let v_col = Color32::from_rgb(46, 210, 110);
        card_frame(Color32::from_rgba_unmultiplied(46, 210, 110, 70)).show(ui, |ui| {
            ui.set_width(190.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("BUS VOLTAGE").color(Color32::from_rgb(130, 155, 185)).size(11.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("[ V ]").color(v_col).size(11.0).strong());
                    });
                });

                ui.add_space(2.0);
                let v_str = if latest.is_some() {
                    format!("{:.3} V", v_val)
                } else {
                    "---.--- V".to_string()
                };
                ui.label(RichText::new(v_str).size(28.0).color(v_col).strong());

                ui.add_space(4.0);
                let min_v = if stats.v_min_mv < u16::MAX { stats.v_min_mv as f64 / 1000.0 } else { 0.0 };
                let max_v = stats.v_max_mv as f64 / 1000.0;
                let delta_mv = ((max_v - min_v) * 1000.0).max(0.0);
                ui.label(
                    RichText::new(format!("Min: {:.3} | Max: {:.3} V (Δ: {:.0}mV)", min_v, max_v, delta_mv))
                        .color(Color32::from_rgb(120, 140, 165))
                        .size(10.0),
                );
            });
        });

        // ==========================================
        // 3. POWER CARD
        // ==========================================
        let p_mw = latest.map(|p| p.power_mw as f64).unwrap_or(0.0);
        let p_col = Color32::from_rgb(245, 180, 50);
        card_frame(Color32::from_rgba_unmultiplied(245, 180, 50, 70)).show(ui, |ui| {
            ui.set_width(190.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("POWER").color(Color32::from_rgb(130, 155, 185)).size(11.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let unit = if p_mw >= 1000.0 { "[ W ]" } else { "[ mW ]" };
                        ui.label(RichText::new(unit).color(p_col).size(11.0).strong());
                    });
                });

                ui.add_space(2.0);
                let p_str = if latest.is_some() {
                    if p_mw >= 1000.0 {
                        format!("{:.3} W", p_mw / 1000.0)
                    } else {
                        format!("{:.1} mW", p_mw)
                    }
                } else {
                    "---.- mW".to_string()
                };
                ui.label(RichText::new(p_str).size(28.0).color(p_col).strong());

                ui.add_space(4.0);
                let peak_mw = stats.p_max_mw as f64;
                let avg_mw = stats.avg_power_mw();
                ui.label(
                    RichText::new(format!("Avg: {:.1} | Peak: {:.1} mW", avg_mw, peak_mw))
                        .color(Color32::from_rgb(120, 140, 165))
                        .size(10.0),
                );
            });
        });

        // ==========================================
        // 4. LOAD IMPEDANCE & ACCUMULATORS
        // ==========================================
        let imp_col = Color32::from_rgb(120, 180, 255);
        card_frame(Color32::from_rgba_unmultiplied(120, 180, 255, 70)).show(ui, |ui| {
            ui.set_width(225.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("IMPEDANCE & ACCUMULATOR").color(Color32::from_rgb(130, 155, 185)).size(11.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("⚡").color(imp_col).size(11.0));
                    });
                });

                ui.add_space(2.0);
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
                ui.label(RichText::new(format!("R: {}", r_str)).size(22.0).color(imp_col).strong());

                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("Energy: {:.2} mWh  |  Charge: {:.2} mAh", stats.energy_mwh, stats.charge_mah))
                        .color(Color32::from_rgb(160, 185, 215))
                        .size(11.0),
                );
            });
        });

        // ==========================================
        // 5. HARDWARE STATUS & TELEMETRY
        // ==========================================
        let stat_col = Color32::from_rgb(45, 210, 130);
        card_frame(Color32::from_rgba_unmultiplied(stat_col.r(), stat_col.g(), stat_col.b(), 60)).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("HARDWARE TELEMETRY").color(Color32::from_rgb(130, 155, 185)).size(11.0).strong());
                    if let Some(p) = latest {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("Seq #{}", p.seq)).color(Color32::from_rgb(130, 155, 185)).size(11.0));
                        });
                    }
                });

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if let Some(p) = latest {
                        let render_pill = |ui: &mut Ui, label: &str, active: bool, active_col: Color32| {
                            let (bg, dot, text) = if active {
                                (Color32::from_rgba_unmultiplied(active_col.r(), active_col.g(), active_col.b(), 35), active_col, Color32::from_rgb(230, 240, 255))
                            } else {
                                (Color32::from_rgb(20, 28, 40), Color32::from_rgb(50, 65, 85), Color32::from_rgb(90, 110, 135))
                            };
                            Frame::none()
                                .fill(bg)
                                .stroke(Stroke::new(1.0, dot))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(6.0, 3.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("●").color(dot).size(9.0));
                                        ui.label(RichText::new(label).color(text).size(10.0).strong());
                                    });
                                });
                        };

                        render_pill(ui, "ONLINE", p.is_online(), Color32::from_rgb(45, 220, 120));
                        render_pill(ui, if p.is_reverse() { "REV" } else { "FWD" }, p.is_reverse(), Color32::from_rgb(240, 175, 45));
                        render_pill(ui, "OVERFLOW", p.is_overflow(), Color32::from_rgb(255, 60, 60));
                        render_pill(ui, "SD-LOG", p.is_sd_logging(), Color32::from_rgb(0, 220, 255));
                        render_pill(ui, "ALERT", p.is_alert(), Color32::from_rgb(255, 50, 50));
                    } else {
                        ui.label(RichText::new("Awaiting telemetry stream...").color(Color32::from_rgb(110, 130, 155)).size(11.0));
                    }
                });
            });
        });
    });
}
