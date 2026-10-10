use egui::{Color32, RichText, Slider, Ui};
use crate::model::{BatteryProfile, HidCommand, OnDeviceScreen};

pub struct HardwareControlState {
    pub selected_battery: BatteryProfile,
    pub current_limit_ma: u16,
    pub selected_screen: OnDeviceScreen,
}

impl Default for HardwareControlState {
    fn default() -> Self {
        Self {
            selected_battery: BatteryProfile::None,
            current_limit_ma: 2000,
            selected_screen: OnDeviceScreen::Hero,
        }
    }
}

pub fn render_controls_panel(
    ui: &mut Ui,
    state: &mut HardwareControlState,
    on_send_cmd: &mut Option<HidCommand>,
) {
    ui.heading(RichText::new("HARDWARE CONTROLS").size(15.0).strong());
    ui.add_space(4.0);

    // 1. Battery Profile Selection
    ui.group(|ui| {
        ui.label(RichText::new("🔋 Battery Profile (SOC Tracking)").strong());
        let prev_prof = state.selected_battery;
        egui::ComboBox::from_id_source("battery_profile_combo")
            .selected_text(state.selected_battery.name())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.selected_battery, BatteryProfile::None, BatteryProfile::None.name());
                ui.selectable_value(&mut state.selected_battery, BatteryProfile::Lipo500, BatteryProfile::Lipo500.name());
                ui.selectable_value(&mut state.selected_battery, BatteryProfile::Lipo1200, BatteryProfile::Lipo1200.name());
                ui.selectable_value(&mut state.selected_battery, BatteryProfile::LiIon2500, BatteryProfile::LiIon2500.name());
                ui.selectable_value(&mut state.selected_battery, BatteryProfile::Alkaline1000, BatteryProfile::Alkaline1000.name());
            });

        if state.selected_battery != prev_prof {
            *on_send_cmd = Some(HidCommand::SetBatteryProfile(state.selected_battery));
        }
    });

    ui.add_space(6.0);

    // 2. Overcurrent Alert Limit Slider
    ui.group(|ui| {
        ui.label(RichText::new("⚠️ Overcurrent Alert Threshold").strong());
        let prev_limit = state.current_limit_ma;
        ui.add(Slider::new(&mut state.current_limit_ma, 50..=3200).suffix(" mA"));

        ui.horizontal(|ui| {
            ui.label("Presets:");
            for &lim in &[250, 500, 1000, 2000, 3000] {
                if ui.button(format!("{}mA", lim)).clicked() {
                    state.current_limit_ma = lim;
                }
            }
        });

        if state.current_limit_ma != prev_limit {
            *on_send_cmd = Some(HidCommand::SetCurrentLimit(state.current_limit_ma));
        }
    });

    ui.add_space(6.0);

    // 3. On-Device Display Screen Mode Switcher
    ui.group(|ui| {
        ui.label(RichText::new("🖥️ Longan Nano Screen Mode").strong());
        ui.horizontal_wrapped(|ui| {
            if ui.button("Hero").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::Hero));
            }
            if ui.button("Graph").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::Graph));
            }
            if ui.button("Stats").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::Stats));
            }
            if ui.button("Histogram").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::Histogram));
            }
            if ui.button("BigDigit").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::BigDigit));
            }
            if ui.button(RichText::new("Cycle ⏭").color(Color32::from_rgb(0, 220, 255))).clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::Cycle));
            }
        });
    });

    ui.add_space(6.0);

    // 4. MicroSD Storage Remote Operations
    ui.group(|ui| {
        ui.label(RichText::new("💾 MicroSD Storage Operations").strong());
        ui.horizontal(|ui| {
            if ui.button("Flush Buffer").clicked() {
                *on_send_cmd = Some(HidCommand::FlushSd);
            }
            if ui.button("Rotate Log").clicked() {
                *on_send_cmd = Some(HidCommand::RotateLog);
            }
            if ui.button("Dump Summary").clicked() {
                *on_send_cmd = Some(HidCommand::RequestSummary);
            }
        });
    });
}
