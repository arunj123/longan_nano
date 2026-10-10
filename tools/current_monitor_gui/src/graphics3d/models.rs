use egui::Color32;
use crate::graphics3d::math3d::Vec3;
use crate::graphics3d::mesh::{Mesh3D, Vertex};
use crate::model::TelemetryPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseProjectionStyle {
    FullStudio,    // Ribbon + Floor Shadow + Wall V-I Curve + Beacon
    RibbonOnly,    // Minimal clean neon trace
    ProjectionsOnly, // Orthogonal 2D projections
}

pub struct PhaseSpace3DConfig {
    pub projection_style: PhaseProjectionStyle,
    pub show_grid: bool,
    pub show_axes: bool,
    pub show_beacon: bool,
    pub ribbon_width: f32,
    pub sample_count: usize,
}

impl Default for PhaseSpace3DConfig {
    fn default() -> Self {
        Self {
            projection_style: PhaseProjectionStyle::FullStudio,
            show_grid: true,
            show_axes: true,
            show_beacon: true,
            ribbon_width: 1.2,
            sample_count: 150,
        }
    }
}

pub fn build_phase_space_visualization(
    history: &[TelemetryPacket],
    _latest: Option<&TelemetryPacket>,
    anim_time: f64,
    config: &PhaseSpace3DConfig,
) -> Mesh3D {
    let mut mesh = Mesh3D::new();

    let grid_y = -14.0;
    let wall_x = -34.0;

    // ==========================================
    // 1. Studio Grid Floor & Back Wall
    // ==========================================
    if config.show_grid {
        let grid_size: f32 = 45.0;
        let grid_col = Color32::from_rgba_unmultiplied(25, 38, 55, 65);
        let grid_accent = Color32::from_rgba_unmultiplied(40, 65, 95, 90);

        let mut x: f32 = -grid_size;
        while x <= grid_size {
            let col = if (x.abs() < 0.1) || ((x + grid_size) as i32 % 20 == 0) {
                grid_accent
            } else {
                grid_col
            };
            mesh.add_line(Vec3::new(x, grid_y, -grid_size), Vec3::new(x, grid_y, grid_size), col);
            x += 5.0;
        }

        let mut z: f32 = -grid_size;
        while z <= grid_size {
            let col = if (z.abs() < 0.1) || ((z + grid_size) as i32 % 20 == 0) {
                grid_accent
            } else {
                grid_col
            };
            mesh.add_line(Vec3::new(-grid_size, grid_y, z), Vec3::new(grid_size, grid_y, z), col);
            z += 5.0;
        }

        // Back wall grid (V-I projection plane at X = wall_x)
        if config.projection_style != PhaseProjectionStyle::RibbonOnly {
            let wall_col = Color32::from_rgba_unmultiplied(20, 32, 48, 50);
            let mut y = grid_y;
            while y <= 36.0 {
                mesh.add_line(Vec3::new(wall_x, y, -25.0), Vec3::new(wall_x, y, 25.0), wall_col);
                y += 10.0;
            }
            let mut wz = -25.0;
            while wz <= 25.0 {
                mesh.add_line(Vec3::new(wall_x, grid_y, wz), Vec3::new(wall_x, 36.0, wz), wall_col);
                wz += 10.0;
            }
        }
    }

    // ==========================================
    // 2. 3D Coordinate Axes & Measurement Cage
    // ==========================================
    if config.show_axes {
        // X Axis: Time (Red / Orange)
        mesh.add_line(Vec3::new(-32.0, grid_y, 0.0), Vec3::new(32.0, grid_y, 0.0), Color32::from_rgb(245, 75, 75));
        // X Axis Arrow
        mesh.add_line(Vec3::new(32.0, grid_y, 0.0), Vec3::new(30.0, grid_y, -1.2), Color32::from_rgb(245, 75, 75));
        mesh.add_line(Vec3::new(32.0, grid_y, 0.0), Vec3::new(30.0, grid_y, 1.2), Color32::from_rgb(245, 75, 75));

        // Y Axis: Current (Green / Mint)
        mesh.add_line(Vec3::new(0.0, grid_y, 0.0), Vec3::new(0.0, 36.0, 0.0), Color32::from_rgb(45, 230, 115));
        // Y Axis Arrow
        mesh.add_line(Vec3::new(0.0, 36.0, 0.0), Vec3::new(-1.2, 34.0, 0.0), Color32::from_rgb(45, 230, 115));
        mesh.add_line(Vec3::new(0.0, 36.0, 0.0), Vec3::new(1.2, 34.0, 0.0), Color32::from_rgb(45, 230, 115));

        // Z Axis: Voltage (Cyan / Blue)
        mesh.add_line(Vec3::new(0.0, grid_y, -24.0), Vec3::new(0.0, grid_y, 24.0), Color32::from_rgb(0, 210, 255));
        // Z Axis Arrow
        mesh.add_line(Vec3::new(0.0, grid_y, 24.0), Vec3::new(-1.2, grid_y, 22.0), Color32::from_rgb(0, 210, 255));
        mesh.add_line(Vec3::new(0.0, grid_y, 24.0), Vec3::new(1.2, grid_y, 22.0), Color32::from_rgb(0, 210, 255));
    }

    if history.len() < 2 {
        return mesh;
    }

    // Extract recent points
    let pts_slice: Vec<&TelemetryPacket> = history
        .iter()
        .rev()
        .take(config.sample_count)
        .collect();
    let n = pts_slice.len();

    // Map each sample to a 3D point
    let map_sample = |idx: usize, p: &TelemetryPacket| -> (Vec3, Color32) {
        // Normalized time position (-30.0 at oldest sample, +30.0 at newest)
        // Since pts_slice is reversed (idx 0 is newest), reverse the X mapping:
        let norm_t = 1.0 - (idx as f32 / (n - 1).max(1) as f32);
        let x = (norm_t * 60.0) - 30.0;

        // Current mapping (Y): 0 mA at -11.0, 200 mA at +10.0, 500 mA at +28.0
        let c = p.current_ma();
        let y = ((c as f32 * 0.12) - 11.0).clamp(-13.0, 35.0);

        // Voltage mapping (Z): centered around nominal 3.3V
        let v = p.voltage_v();
        let z = (((v as f32) - 3.30) * 22.0).clamp(-22.0, 22.0);

        // Severity-based neon color
        let col = if c > 450.0 {
            Color32::from_rgb(255, 45, 45) // Critical Overload Red
        } else if c > 120.0 {
            Color32::from_rgb(250, 160, 30) // High Load Amber
        } else if c > 25.0 {
            Color32::from_rgb(45, 225, 120) // Active Load Mint
        } else {
            Color32::from_rgb(0, 225, 255)  // Quiescent Cyan
        };

        (Vec3::new(x, y, z), col)
    };

    let mapped: Vec<(Vec3, Color32)> = pts_slice
        .iter()
        .enumerate()
        .map(|(i, &p)| map_sample(i, p))
        .collect();

    // ==========================================
    // 3. Neon 3D Extruded Ribbon
    // ==========================================
    let hw = config.ribbon_width * 0.5;

    for i in 0..(mapped.len() - 1) {
        // pts_slice[0] is newest, so mapped[0] is at x=+30, mapped[n-1] is at x=-30
        let (p_new, col_new) = mapped[i];
        let (p_old, col_old) = mapped[i + 1];

        // Normal/offset vector perpendicular to segment in XZ
        let dir = (p_new - p_old).normalized();
        let mut normal_xz = Vec3::new(-dir.z, 0.0, dir.x);
        if normal_xz.length_sq() < 0.001 {
            normal_xz = Vec3::new(0.0, 0.0, 1.0);
        } else {
            normal_xz = normal_xz.normalized();
        }
        let offset = normal_xz * hw;

        // Top surface quad (emissive)
        mesh.add_quad_layered(
            Vertex::new(p_old - offset, col_old, Vec3::UP),
            Vertex::new(p_old + offset, col_old, Vec3::UP),
            Vertex::new(p_new + offset, col_new, Vec3::UP),
            Vertex::new(p_new - offset, col_new, Vec3::UP),
            true,
            2,
        );

        // Underside quad (for inverted camera view)
        mesh.add_quad_layered(
            Vertex::new(p_new - offset, col_new, -Vec3::UP),
            Vertex::new(p_new + offset, col_new, -Vec3::UP),
            Vertex::new(p_old + offset, col_old, -Vec3::UP),
            Vertex::new(p_old - offset, col_old, -Vec3::UP),
            true,
            2,
        );

        // Floor Shadow Projection (Time vs Voltage depth plane)
        if config.projection_style == PhaseProjectionStyle::FullStudio {
            let s_old = Vec3::new(p_old.x, grid_y + 0.12, p_old.z);
            let s_new = Vec3::new(p_new.x, grid_y + 0.12, p_new.z);
            let shadow_col = Color32::from_rgba_unmultiplied(15, 30, 50, 75);
            let s_offset = Vec3::new(0.0, 0.0, hw * 0.7);

            mesh.add_quad_layered(
                Vertex::new(s_old - s_offset, shadow_col, Vec3::UP),
                Vertex::new(s_old + s_offset, shadow_col, Vec3::UP),
                Vertex::new(s_new + s_offset, shadow_col, Vec3::UP),
                Vertex::new(s_new - s_offset, shadow_col, Vec3::UP),
                false,
                1,
            );

            // Back Wall Projection (V-I characteristic load curve on X = wall_x plane)
            let w_old = Vec3::new(wall_x + 0.12, p_old.y, p_old.z);
            let w_new = Vec3::new(wall_x + 0.12, p_new.y, p_new.z);
            let wall_shadow = Color32::from_rgba_unmultiplied(20, 50, 80, 85);
            mesh.add_line(w_old, w_new, wall_shadow);
        }
    }

    // ==========================================
    // 4. Live Vector Target Probe Beacon
    // ==========================================
    if config.show_beacon && !mapped.is_empty() {
        let (head_pos, head_col) = mapped[0];

        // Pulsating probe radius
        let pulse = ((anim_time * 7.0).sin() as f32 * 0.3) + 1.2;

        // Glowing 3D Diamond / Octahedron at the latest telemetry head
        let p_top = head_pos + Vec3::new(0.0, pulse * 1.3, 0.0);
        let p_bot = head_pos - Vec3::new(0.0, pulse * 1.3, 0.0);
        let p_r = head_pos + Vec3::new(pulse, 0.0, 0.0);
        let p_l = head_pos - Vec3::new(pulse, 0.0, 0.0);
        let p_f = head_pos + Vec3::new(0.0, 0.0, pulse);
        let p_b = head_pos - Vec3::new(0.0, 0.0, pulse);

        let beacon_col = Color32::from_rgb(255, 255, 255);
        mesh.add_triangle_layered(Vertex::new(p_top, beacon_col, Vec3::UP), Vertex::new(p_r, head_col, Vec3::UP), Vertex::new(p_f, head_col, Vec3::UP), true, 4);
        mesh.add_triangle_layered(Vertex::new(p_top, beacon_col, Vec3::UP), Vertex::new(p_f, head_col, Vec3::UP), Vertex::new(p_l, head_col, Vec3::UP), true, 4);
        mesh.add_triangle_layered(Vertex::new(p_top, beacon_col, Vec3::UP), Vertex::new(p_l, head_col, Vec3::UP), Vertex::new(p_b, head_col, Vec3::UP), true, 4);
        mesh.add_triangle_layered(Vertex::new(p_top, beacon_col, Vec3::UP), Vertex::new(p_b, head_col, Vec3::UP), Vertex::new(p_r, head_col, Vec3::UP), true, 4);

        mesh.add_triangle_layered(Vertex::new(p_bot, head_col, -Vec3::UP), Vertex::new(p_f, head_col, -Vec3::UP), Vertex::new(p_r, head_col, -Vec3::UP), true, 4);
        mesh.add_triangle_layered(Vertex::new(p_bot, head_col, -Vec3::UP), Vertex::new(p_l, head_col, -Vec3::UP), Vertex::new(p_f, head_col, -Vec3::UP), true, 4);
        mesh.add_triangle_layered(Vertex::new(p_bot, head_col, -Vec3::UP), Vertex::new(p_b, head_col, -Vec3::UP), Vertex::new(p_l, head_col, -Vec3::UP), true, 4);
        mesh.add_triangle_layered(Vertex::new(p_bot, head_col, -Vec3::UP), Vertex::new(p_r, head_col, -Vec3::UP), Vertex::new(p_b, head_col, -Vec3::UP), true, 4);

        // Vertical laser drop-line to floor grid
        let drop_floor = Vec3::new(head_pos.x, grid_y + 0.1, head_pos.z);
        let drop_col = Color32::from_rgba_unmultiplied(head_col.r(), head_col.g(), head_col.b(), 120);
        mesh.add_line(head_pos, drop_floor, drop_col);

        // Concentric target reticle on floor
        let ring_col = Color32::from_rgba_unmultiplied(head_col.r(), head_col.g(), head_col.b(), 160);
        mesh.add_box_layered(drop_floor, Vec3::new(2.4, 0.05, 2.4), ring_col, 1);

        // Horizontal laser line to back wall V-I curve
        let wall_point = Vec3::new(wall_x + 0.1, head_pos.y, head_pos.z);
        mesh.add_line(head_pos, wall_point, Color32::from_rgba_unmultiplied(80, 160, 240, 100));
    }

    mesh
}
