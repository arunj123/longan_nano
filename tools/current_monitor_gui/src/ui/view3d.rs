use egui::{Color32, Frame, RichText, Sense, Ui};
use crate::graphics3d::{build_longan_nano_board, build_phase_trajectory_ribbon, Renderer3D};
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
    ui.horizontal(|ui| {
        ui.label(RichText::new("3D GRAPHICS VIEWPORT").strong());
        ui.separator();

        ui.selectable_value(&mut state.mode, View3DMode::HardwareDigitalTwin, "📦 3D Digital Twin (Hardware Model)");
        ui.selectable_value(&mut state.mode, View3DMode::PhaseSpaceTrajectory, "📈 3D V-I-t Phase Space Trajectory");

        ui.separator();

        ui.checkbox(&mut state.renderer.auto_rotate, "🔄 Auto-Rotate");
        ui.checkbox(&mut state.renderer.wireframe, "🕸️ Wireframe CAD");

        if ui.button("🎯 Reset Camera").clicked() {
            state.renderer.camera = Default::default();
        }
    });

    ui.add_space(4.0);

    // Canvas container
    Frame::canvas(ui.style())
        .fill(Color32::from_rgb(10, 14, 22))
        .inner_margin(4.0)
        .show(ui, |ui| {
            let available_size = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(available_size, Sense::click_and_drag());

            // Handle interactive mouse orbit, pan, and zoom
            if response.dragged_by(egui::PointerButton::Primary) {
                let delta = response.drag_delta();
                state.renderer.camera.yaw += delta.x * 0.01;
                state.renderer.camera.pitch = (state.renderer.camera.pitch - delta.y * 0.01).clamp(-1.4, 1.4);
            }

            if response.dragged_by(egui::PointerButton::Secondary) {
                let delta = response.drag_delta();
                state.renderer.camera.pan.x -= delta.x * 0.1;
                state.renderer.camera.pan.y += delta.y * 0.1;
            }

            // Mouse wheel zoom
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll.abs() > 0.0 && response.hovered() {
                state.renderer.camera.distance = (state.renderer.camera.distance - scroll * 0.05).clamp(20.0, 300.0);
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
                    // Extract recent trajectory (t, V, I)
                    let points: Vec<(f64, f64, f64)> = history
                        .iter()
                        .rev()
                        .take(80)
                        .map(|p| (p.relative_secs, p.voltage_v(), p.current_ma()))
                        .collect();
                    build_phase_trajectory_ribbon(&points)
                }
            };

            // Render 3D mesh
            let painter = ui.painter_at(rect);
            state.renderer.render(&painter, rect, &mesh);

            // Overlay HUD hints in corner
            painter.text(
                rect.min + egui::vec2(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                "L-Drag: Orbit | R-Drag: Pan | Scroll: Zoom",
                egui::FontId::proportional(11.0),
                Color32::from_rgba_unmultiplied(160, 180, 205, 160),
            );
        });
}
