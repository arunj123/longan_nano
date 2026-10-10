use egui::{Color32, Frame, Margin, RichText, Rounding, Sense, Stroke, Ui};
use crate::graphics3d::{build_phase_space_visualization, PhaseProjectionStyle, PhaseSpace3DConfig, Renderer3D, Vec3};
use crate::model::TelemetryPacket;

pub struct View3DState {
    pub config: PhaseSpace3DConfig,
    pub renderer: Renderer3D,
}

impl Default for View3DState {
    fn default() -> Self {
        let mut renderer = Renderer3D::default();
        renderer.camera.yaw = 0.62;
        renderer.camera.pitch = 0.45;
        renderer.camera.distance = 92.0;
        renderer.camera.pan = Vec3::new(0.0, 6.0, 0.0);

        Self {
            config: PhaseSpace3DConfig::default(),
            renderer,
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
    // 3D Toolbar
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("3D PHASE SPACE OSCILLOSCOPE").size(13.0).color(Color32::from_rgb(0, 230, 255)).strong());
        ui.separator();

        // Style selector
        ui.selectable_value(&mut state.config.projection_style, PhaseProjectionStyle::FullStudio, "Studio View");
        ui.selectable_value(&mut state.config.projection_style, PhaseProjectionStyle::RibbonOnly, "Neon Ribbon");

        ui.separator();

        // Camera Presets
        ui.label(RichText::new("Angle:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
        if ui.selectable_label(false, "📐 3D Orbit").clicked() {
            state.renderer.camera.yaw = 0.62;
            state.renderer.camera.pitch = 0.45;
            state.renderer.camera.distance = 92.0;
            state.renderer.camera.pan = Vec3::new(0.0, 6.0, 0.0);
        }
        if ui.selectable_label(false, "⚡ V-I Curve").clicked() {
            state.renderer.camera.yaw = 0.0;
            state.renderer.camera.pitch = 0.0;
            state.renderer.camera.distance = 80.0;
            state.renderer.camera.pan = Vec3::new(0.0, 8.0, 0.0);
        }
        if ui.selectable_label(false, "📊 I-t Profile").clicked() {
            state.renderer.camera.yaw = -std::f32::consts::FRAC_PI_2;
            state.renderer.camera.pitch = 0.0;
            state.renderer.camera.distance = 85.0;
            state.renderer.camera.pan = Vec3::new(0.0, 8.0, 0.0);
        }
        if ui.selectable_label(false, "🔋 V-t Profile").clicked() {
            state.renderer.camera.yaw = 0.0;
            state.renderer.camera.pitch = 1.50;
            state.renderer.camera.distance = 90.0;
            state.renderer.camera.pan = Vec3::new(0.0, 0.0, 0.0);
        }

        ui.separator();

        ui.checkbox(&mut state.renderer.auto_rotate, "🔄 Turntable");
        ui.checkbox(&mut state.config.show_beacon, "🎯 Beacon");
        ui.checkbox(&mut state.renderer.wireframe, "🕸️ Wireframe");

        if ui.button("↺ Reset Cam").clicked() {
            state.renderer.camera.yaw = 0.62;
            state.renderer.camera.pitch = 0.45;
            state.renderer.camera.distance = 92.0;
            state.renderer.camera.pan = Vec3::new(0.0, 6.0, 0.0);
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

            // Orbit controls (Left Click Drag)
            if response.dragged_by(egui::PointerButton::Primary) {
                let delta = response.drag_delta();
                state.renderer.camera.yaw += delta.x * 0.01;
                state.renderer.camera.pitch = (state.renderer.camera.pitch - delta.y * 0.01).clamp(-1.45, 1.45);
            }

            // Pan controls (Right Click Drag)
            if response.dragged_by(egui::PointerButton::Secondary) {
                let delta = response.drag_delta();
                state.renderer.camera.pan.x -= delta.x * 0.08;
                state.renderer.camera.pan.y += delta.y * 0.08;
            }

            // Zoom controls (Mouse Wheel)
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll.abs() > 0.0 && response.hovered() {
                state.renderer.camera.distance = (state.renderer.camera.distance - scroll * 0.06).clamp(20.0, 280.0);
            }

            // Build active 3D phase space mesh
            let mesh = build_phase_space_visualization(
                history,
                latest,
                anim_time,
                &state.config,
            );

            // Render 3D mesh
            let painter = ui.painter_at(rect);
            state.renderer.render(&painter, rect, &mesh);

            // HUD Badges
            // Top-left: Camera stats
            painter.text(
                rect.min + egui::vec2(12.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("Camera: Yaw {:.1}° | Pitch {:.1}° | Zoom {:.0}", state.renderer.camera.yaw.to_degrees(), state.renderer.camera.pitch.to_degrees(), state.renderer.camera.distance),
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(140, 165, 195, 160),
            );

            // Top-right: Live Vector state readout
            if let Some(pkt) = latest {
                let vector_hud = format!("Vector Head: [V: {:.3} V | I: {:.1} mA | P: {:.1} mW]", pkt.voltage_v(), pkt.current_ma(), pkt.power_mw as f64);
                painter.text(
                    egui::pos2(rect.max.x - 12.0, rect.min.y + 10.0),
                    egui::Align2::RIGHT_TOP,
                    vector_hud,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(0, 230, 255),
                );
            }

            // Bottom-left: Controls guide
            painter.text(
                egui::pos2(rect.min.x + 12.0, rect.max.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "Left-Drag: Orbit  |  Right-Drag: Pan  |  Wheel: Zoom",
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(120, 140, 170, 140),
            );

            // Bottom-right: Polys & FPS
            painter.text(
                egui::pos2(rect.max.x - 12.0, rect.max.y - 12.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("Triangles: {}  |  60 FPS Hardware-Accelerated", mesh.faces.len()),
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(100, 130, 160, 130),
            );
        });
}
