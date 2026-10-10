use egui::{Color32, RichText, Ui};
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
        // App Title & Logo
        ui.heading(RichText::new("⚡ LONGAN NANO CURRENT MONITOR").color(Color32::from_rgb(0, 230, 255)).strong());
        ui.separator();

        // Connection Status Badge
        if connected {
            ui.label(RichText::new("● CONNECTED").color(Color32::from_rgb(50, 230, 100)).strong());
        } else {
            ui.label(RichText::new("○ DISCONNECTED").color(Color32::from_rgb(220, 70, 70)).strong());
        }

        ui.label(RichText::new(format!("({})", status_text)).color(Color32::from_rgb(160, 175, 195)).small());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Transport Selector
            let mut selected_idx = match current_transport {
                TransportKind::LocalHid => 0,
                TransportKind::TcpBridge(s) if s.contains("127.0.0.1") => 1,
                TransportKind::TcpBridge(s) if s.contains("192.168.0.63") => 2,
                TransportKind::Simulation => 3,
                _ => 1,
            };

            let prev_idx = selected_idx;
            egui::ComboBox::from_id_source("transport_selector")
                .selected_text(match selected_idx {
                    0 => "Direct USB HID (VID:0x28E9, PID:0x1234)",
                    1 => "Local TCP Bridge (127.0.0.1:5055)",
                    2 => "Remote Testbed (192.168.0.63:5055)",
                    _ => "Simulation Mode (Synthetic)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected_idx, 0, "Direct USB HID (Local Windows/Linux)");
                    ui.selectable_value(&mut selected_idx, 1, "Local TCP Bridge (127.0.0.1:5055)");
                    ui.selectable_value(&mut selected_idx, 2, "Remote Testbed (192.168.0.63:5055)");
                    ui.selectable_value(&mut selected_idx, 3, "Simulation Mode (Synthetic Telemetry)");
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

            // Record CSV Button
            let rec_text = if is_recording {
                format!("⏹ Stop Rec ({} pkts)", recording_samples)
            } else {
                "⏺ Record CSV".to_string()
            };
            let rec_color = if is_recording { Color32::from_rgb(255, 80, 80) } else { Color32::from_rgb(200, 200, 200) };
            if ui.button(RichText::new(rec_text).color(rec_color)).clicked() {
                *on_toggle_recording = true;
            }

            // Sync Epoch Button
            if ui.button(RichText::new("⏱ Sync RTC Epoch").color(Color32::from_rgb(230, 200, 70))).clicked() {
                let epoch = Utc::now().timestamp() as u32;
                *on_send_cmd = Some(HidCommand::SetEpoch(epoch));
            }

            // Zero-Tare Button
            if ui.button(RichText::new("⌖ Zero-Tare").color(Color32::from_rgb(80, 220, 255)).strong()).clicked() {
                *on_send_cmd = Some(HidCommand::TareZero);
            }
        });
    });
}
