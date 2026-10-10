use egui::{Color32, Frame, Margin, RichText, Rounding, Stroke, Ui};
use chrono::Utc;
use crate::model::HidCommand;
use crate::transport::TransportKind;

pub fn render_header(
    ui: &mut Ui,
    connected: bool,
    status_text: &str,
    current_transport: &TransportKind,
    on_switch_transport: &mut Option<TransportKind>,
    on_send_cmd: &mut Option<HidCommand>,
    is_recording: bool,
    recording_samples: u64,
    on_toggle_recording: &mut bool,
) {
    ui.horizontal(|ui| {
        // App Title & Brand
        ui.label(RichText::new("⚡ LONGAN NANO").color(Color32::from_rgb(0, 235, 255)).size(15.0).strong());
        ui.label(RichText::new("CURRENT MONITOR").color(Color32::from_rgb(220, 230, 245)).size(13.0).strong());

        // Build Badge
        Frame::none()
            .fill(Color32::from_rgb(22, 32, 48))
            .stroke(Stroke::new(1.0, Color32::from_rgb(45, 65, 95)))
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(6.0, 2.0))
            .show(ui, |ui| {
                ui.label(RichText::new("BUILD 00CE").color(Color32::from_rgb(0, 220, 255)).size(10.0).strong());
            });

        ui.separator();

        // Connection Status Pill
        let (conn_dot, conn_text, conn_bg, conn_border) = if connected {
            (
                Color32::from_rgb(45, 230, 110),
                "CONNECTED",
                Color32::from_rgba_unmultiplied(45, 230, 110, 25),
                Color32::from_rgb(45, 230, 110),
            )
        } else {
            (
                Color32::from_rgb(240, 70, 70),
                "DISCONNECTED",
                Color32::from_rgba_unmultiplied(240, 70, 70, 25),
                Color32::from_rgb(240, 70, 70),
            )
        };

        Frame::none()
            .fill(conn_bg)
            .stroke(Stroke::new(1.0, conn_border))
            .rounding(Rounding::same(12.0))
            .inner_margin(Margin::symmetric(8.0, 3.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(conn_dot).size(10.0));
                    ui.label(RichText::new(conn_text).color(Color32::from_rgb(240, 245, 255)).size(11.0).strong());
                });
            });

        ui.label(RichText::new(format!("({})", status_text)).color(Color32::from_rgb(130, 150, 175)).size(11.0));

        // Right-aligned toolbar
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 1. Transport Selector Dropdown
            let mut selected_idx = match current_transport {
                TransportKind::LocalHid => 0,
                TransportKind::TcpBridge(s) if s.contains("127.0.0.1") => 1,
                TransportKind::TcpBridge(s) if s.contains("192.168.0.63") => 2,
                TransportKind::Simulation => 3,
                _ => 1,
            };

            let prev_idx = selected_idx;
            egui::ComboBox::from_id_source("top_transport_combo")
                .selected_text(match selected_idx {
                    0 => "🔌 Direct USB HID (Local)",
                    1 => "🌐 Local Bridge (127.0.0.1)",
                    2 => "🛰️ Remote Testbed (192.168.0.63)",
                    _ => "🎮 Simulation Demo",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected_idx, 0, "🔌 Direct USB HID (Local Windows/Linux)");
                    ui.selectable_value(&mut selected_idx, 1, "🌐 Local TCP Bridge (127.0.0.1:5055)");
                    ui.selectable_value(&mut selected_idx, 2, "🛰️ Remote Testbed (192.168.0.63:5055)");
                    ui.selectable_value(&mut selected_idx, 3, "🎮 Simulation Demo (Synthetic)");
                });

            if selected_idx != prev_idx {
                let new_kind = match selected_idx {
                    0 => TransportKind::LocalHid,
                    1 => TransportKind::TcpBridge("127.0.0.1:5055".to_string()),
                    2 => TransportKind::TcpBridge("192.168.0.63:5055".to_string()),
                    _ => TransportKind::Simulation,
                };
                *on_switch_transport = Some(new_kind);
            }

            ui.separator();

            // 2. Record CSV Button
            let (rec_label, rec_bg, rec_fg) = if is_recording {
                (
                    format!("⏹ STOP REC ({} pkts)", recording_samples),
                    Color32::from_rgb(180, 30, 30),
                    Color32::from_rgb(255, 235, 235),
                )
            } else {
                (
                    "⏺ RECORD CSV".to_string(),
                    Color32::from_rgb(28, 38, 54),
                    Color32::from_rgb(200, 215, 235),
                )
            };
            if ui.add(egui::Button::new(RichText::new(rec_label).size(11.0).color(rec_fg).strong()).fill(rec_bg)).clicked() {
                *on_toggle_recording = true;
            }

            // 3. Sync RTC Epoch Button
            if ui.add(egui::Button::new(RichText::new("⏱ SYNC RTC").size(11.0).color(Color32::from_rgb(245, 200, 60)).strong()).fill(Color32::from_rgb(28, 38, 54))).clicked() {
                let epoch = Utc::now().timestamp() as u32;
                *on_send_cmd = Some(HidCommand::SetEpoch(epoch));
            }

            // 4. Zero-Tare Button
            if ui.add(egui::Button::new(RichText::new("⌖ ZERO-TARE").size(11.0).color(Color32::from_rgb(0, 235, 255)).strong()).fill(Color32::from_rgb(18, 42, 60))).clicked() {
                *on_send_cmd = Some(HidCommand::TareZero);
            }
        });
    });
}
