use egui::{Color32, Frame, Margin, RichText, Rounding, Slider, Stroke, Ui};
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
    ui.label(RichText::new("⚙️ HARDWARE CONTROLS").size(13.0).color(Color32::from_rgb(0, 230, 255)).strong());
    ui.add_space(6.0);

    let card_frame = Frame::none()
        .fill(Color32::from_rgb(15, 22, 34))
        .stroke(Stroke::new(1.0, Color32::from_rgb(30, 42, 60)))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0));

    // 1. Battery Profile Selection
    card_frame.show(ui, |ui| {
        ui.label(RichText::new("🔋 Battery Profile (SOC Tracking)").color(Color32::from_rgb(220, 235, 255)).strong());
        ui.add_space(2.0);
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
    card_frame.show(ui, |ui| {
        ui.label(RichText::new("⚠️ Overcurrent Alert Threshold").color(Color32::from_rgb(245, 185, 45)).strong());
        ui.add_space(2.0);
        let prev_limit = state.current_limit_ma;
        ui.add(Slider::new(&mut state.current_limit_ma, 50..=3200).suffix(" mA"));

        ui.horizontal(|ui| {
            ui.label(RichText::new("Presets:").color(Color32::from_rgb(130, 150, 175)).size(11.0));
            for &lim in &[250, 500, 1000, 2000, 3000] {
                if ui.button(RichText::new(format!("{}mA", lim)).size(10.0)).clicked() {
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
    card_frame.show(ui, |ui| {
        ui.label(RichText::new("🖥️ Longan Nano Screen Switcher").color(Color32::from_rgb(0, 220, 255)).strong());
        ui.add_space(4.0);
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
            if ui.button("Histo").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::Histogram));
            }
            if ui.button("BigDigit").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::BigDigit));
            }
            if ui.button("⏭ Cycle").clicked() {
                *on_send_cmd = Some(HidCommand::SetMode(OnDeviceScreen::Cycle));
            }
        });
    });

    ui.add_space(6.0);

    // 4. MicroSD Storage Remote Operations
    card_frame.show(ui, |ui| {
        ui.label(RichText::new("💾 MicroSD Operations").color(Color32::from_rgb(120, 180, 255)).strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button("Flush Buffer").clicked() {
                *on_send_cmd = Some(HidCommand::FlushSd);
            }
            if ui.button("Rotate File").clicked() {
                *on_send_cmd = Some(HidCommand::RotateLog);
            }
            if ui.button("UART Stats").clicked() {
                *on_send_cmd = Some(HidCommand::RequestSummary);
            }
        });
    });
}
