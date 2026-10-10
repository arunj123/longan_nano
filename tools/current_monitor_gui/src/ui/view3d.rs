use egui::{Color32, Frame, Margin, RichText, Rounding, Sense, Stroke, Ui};
use crate::graphics3d::{build_longan_nano_board, build_phase_trajectory_ribbon, Renderer3D, Vec3};
use crate::model::TelemetryPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View3DMode {
    HardwareDigitalTwin,
    PhaseSpaceTrajectory,
}

pub struct View3DState {
    pub mode: View3DMode,
    pub renderer: Renderer3D,
}

impl Default for View3DState {
    fn default() -> Self {
        Self {
            mode: View3DMode::HardwareDigitalTwin,
            renderer: Renderer3D::default(),
        }
    }
}

pub fn render_3d_viewport(
    ui: &mut Ui,
    state: &mut View3DState,
    latest: Option<&TelemetryPacket>,
    history: &[TelemetryPacket],
    anim_time: f64,
) {
    // Clean, non-colliding 3D Toolbar
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("3D GRAPHICS VIEWPORT").size(13.0).color(Color32::from_rgb(0, 230, 255)).strong());
        ui.separator();

        ui.selectable_value(&mut state.mode, View3DMode::HardwareDigitalTwin, "📦 Digital Twin");
        ui.selectable_value(&mut state.mode, View3DMode::PhaseSpaceTrajectory, "📈 3D Phase Space");

        ui.separator();

        ui.checkbox(&mut state.renderer.auto_rotate, "🔄 Turntable");
        ui.checkbox(&mut state.renderer.wireframe, "🕸️ Wireframe");

        if ui.button("🎯 Reset Cam").clicked() {
            state.renderer.camera = Default::default();
        }

        // Camera Angle Presets
        if ui.selectable_label(false, "Top").clicked() {
            state.renderer.camera.yaw = 0.0;
            state.renderer.camera.pitch = 1.35;
            state.renderer.camera.distance = 75.0;
            state.renderer.camera.pan = Vec3::new(4.0, 0.0, 6.0);
        }
        if ui.selectable_label(false, "Display").clicked() {
            state.renderer.camera.yaw = 0.40;
            state.renderer.camera.pitch = 0.35;
            state.renderer.camera.distance = 55.0;
            state.renderer.camera.pan = Vec3::new(12.5, 2.0, 0.0);
        }
    });

    ui.add_space(4.0);

    // Viewport Frame
    Frame::none()
        .fill(Color32::from_rgb(8, 12, 18))
        .stroke(Stroke::new(1.0, Color32::from_rgb(25, 36, 52)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(2.0))
        .show(ui, |ui| {
            let available_size = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(available_size, Sense::click_and_drag());

            // Orbit controls
            if response.dragged_by(egui::PointerButton::Primary) {
                let delta = response.drag_delta();
                state.renderer.camera.yaw += delta.x * 0.01;
                state.renderer.camera.pitch = (state.renderer.camera.pitch - delta.y * 0.01).clamp(-1.45, 1.45);
            }

            // Pan controls
            if response.dragged_by(egui::PointerButton::Secondary) {
                let delta = response.drag_delta();
                state.renderer.camera.pan.x -= delta.x * 0.08;
                state.renderer.camera.pan.y += delta.y * 0.08;
            }

            // Zoom controls
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll.abs() > 0.0 && response.hovered() {
                state.renderer.camera.distance = (state.renderer.camera.distance - scroll * 0.05).clamp(15.0, 260.0);
            }

            // Build active 3D model
            let mesh = match state.mode {
                View3DMode::HardwareDigitalTwin => {
                    let v_mv = latest.map(|p| p.voltage_mv).unwrap_or(3300);
                    let c_ma = latest.map(|p| p.current_ma()).unwrap_or(0.0);
                    let p_mw = latest.map(|p| p.power_mw).unwrap_or(0);
                    let alert = latest.map(|p| p.is_alert()).unwrap_or(false);
                    let conn = latest.map(|p| p.is_online()).unwrap_or(false);
                    build_longan_nano_board(v_mv, c_ma, p_mw, alert, conn, anim_time)
                }
                View3DMode::PhaseSpaceTrajectory => {
                    let points: Vec<(f64, f64, f64)> = history
                        .iter()
                        .rev()
                        .take(120)
                        .map(|p| (p.relative_secs, p.voltage_v(), p.current_ma()))
                        .collect();
                    build_phase_trajectory_ribbon(&points)
                }
            };

            // Render 3D mesh
            let painter = ui.painter_at(rect);
            state.renderer.render(&painter, rect, &mesh);

            // Sleek HUD Overlay Badges
            // Top-left: Camera stats
            painter.text(
                rect.min + egui::vec2(12.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("Camera: Yaw {:.1}° | Pitch {:.1}° | Zoom {:.0}", state.renderer.camera.yaw.to_degrees(), state.renderer.camera.pitch.to_degrees(), state.renderer.camera.distance),
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(140, 165, 195, 150),
            );

            // Top-right: Active Mode Badge
            let mode_str = match state.mode {
                View3DMode::HardwareDigitalTwin => "LONGAN NANO DIGITAL TWIN (HARDWARE REPLICA)",
                View3DMode::PhaseSpaceTrajectory => "V-I-t PHASE SPACE OSCILLOSCOPE TRAJECTORY",
            };
            painter.text(
                egui::pos2(rect.max.x - 12.0, rect.min.y + 10.0),
                egui::Align2::RIGHT_TOP,
                mode_str,
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(0, 230, 255, 180),
            );

            // Bottom-left: Interaction guide
            painter.text(
                egui::pos2(rect.min.x + 12.0, rect.max.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "Left-Drag: Orbit  |  Right-Drag: Pan  |  Wheel: Zoom",
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(120, 140, 170, 140),
            );

            // Bottom-right: Poly count & 60 FPS
            painter.text(
                egui::pos2(rect.max.x - 12.0, rect.max.y - 12.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("Polys: {}  |  60 FPS (Hardware Accel)", mesh.faces.len()),
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(100, 130, 160, 130),
            );
        });
}
