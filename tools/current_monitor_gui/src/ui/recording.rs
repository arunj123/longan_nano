use std::fs::File;
use std::io::Write;
use chrono::Utc;
use egui::{Color32, RichText, Ui};
use crate::model::TelemetryPacket;

pub struct CsvRecorder {
    pub is_recording: bool,
    pub filename: String,
    pub samples_written: u64,
    file_writer: Option<File>,
}

impl Default for CsvRecorder {
    fn default() -> Self {
        let default_name = format!("session_{}.csv", Utc::now().format("%Y%m%d_%H%M%S"));
        Self {
            is_recording: false,
            filename: default_name,
            samples_written: 0,
            file_writer: None,
        }
    }
}

impl CsvRecorder {
    pub fn start(&mut self) -> Result<(), String> {
        let mut file = File::create(&self.filename).map_err(|e| format!("Failed to create file: {}", e))?;
        writeln!(
            file,
            "Timestamp_UTC,Relative_s,Voltage_mV,Current_mA,Power_mW,Flags,Sequence"
        )
        .map_err(|e| format!("Write header failed: {}", e))?;

        self.file_writer = Some(file);
        self.is_recording = true;
        self.samples_written = 0;
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut f) = self.file_writer.take() {
            let _ = f.flush();
        }
        self.is_recording = false;
    }

    pub fn write_packet(&mut self, pkt: &TelemetryPacket) {
        if !self.is_recording {
            return;
        }

        if let Some(f) = &mut self.file_writer {
            let now_str = Utc::now().to_rfc3339();
            let _ = writeln!(
                f,
                "{},{:.3},{},{:.1},{},0x{:02X},{}",
                now_str,
                pkt.relative_secs,
                pkt.voltage_mv,
                pkt.current_ma(),
                pkt.power_mw,
                pkt.flags,
                pkt.seq
            );
            self.samples_written += 1;
        }
    }

    pub fn export_buffer(&self, history: &[TelemetryPacket], target_path: &str) -> Result<usize, String> {
        let mut file = File::create(target_path).map_err(|e| format!("Failed to create export file: {}", e))?;
        writeln!(
            file,
            "Timestamp_UTC,Relative_s,Voltage_mV,Current_mA,Power_mW,Flags,Sequence"
        )
        .map_err(|e| format!("Write header failed: {}", e))?;

        for p in history {
            let _ = writeln!(
                file,
                "{},{:.3},{},{:.1},{},0x{:02X},{}",
                Utc::now().to_rfc3339(),
                p.relative_secs,
                p.voltage_mv,
                p.current_ma(),
                p.power_mw,
                p.flags,
                p.seq
            );
        }
        let _ = file.flush();
        Ok(history.len())
    }
}

pub fn render_recording_panel(
    ui: &mut Ui,
    recorder: &mut CsvRecorder,
    history: &[TelemetryPacket],
) {
    ui.heading(RichText::new("DATA LOGGING & EXPORT").size(15.0).strong());
    ui.add_space(4.0);

    ui.group(|ui| {
        ui.label(RichText::new("📁 CSV File Recording").strong());
        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut recorder.filename);
        });

        ui.horizontal(|ui| {
            if !recorder.is_recording {
                if ui.button(RichText::new("⏺ Start Recording").color(Color32::from_rgb(0, 220, 255)).strong()).clicked() {
                    let _ = recorder.start();
                }
            } else {
                if ui.button(RichText::new("⏹ Stop Recording").color(Color32::from_rgb(255, 70, 70)).strong()).clicked() {
                    recorder.stop();
                }
                ui.label(
                    RichText::new(format!("Writing: {} samples", recorder.samples_written))
                        .color(Color32::from_rgb(50, 230, 100)),
                );
            }
        });
    });

    ui.add_space(6.0);

    ui.group(|ui| {
        ui.label(RichText::new("💾 Buffer Snapshot").strong());
        ui.label(format!("Active buffer: {} samples", history.len()));
        if ui.button("Export Buffer to CSV").clicked() {
            let snap_name = format!("snapshot_{}.csv", Utc::now().format("%Y%m%d_%H%M%S"));
            let _ = recorder.export_buffer(history, &snap_name);
        }
    });
}
