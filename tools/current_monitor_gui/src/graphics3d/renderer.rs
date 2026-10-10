use egui::{Color32, Mesh, Painter, Pos2, Rect, Stroke};
use crate::graphics3d::math3d::{Mat4, Vec3};
use crate::graphics3d::mesh::Mesh3D;

pub struct Camera3D {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub pan: Vec3,
    pub fov_deg: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            yaw: 0.58,
            pitch: 0.52,
            distance: 82.0,
            pan: Vec3::new(4.0, 0.0, 6.0),
            fov_deg: 44.0,
        }
    }
}

impl Camera3D {
    pub fn eye_position(&self) -> Vec3 {
        let (s_pitch, c_pitch) = self.pitch.sin_cos();
        let (s_yaw, c_yaw) = self.yaw.sin_cos();

        Vec3::new(
            self.pan.x + self.distance * c_pitch * s_yaw,
            self.pan.y + self.distance * s_pitch,
            self.pan.z + self.distance * c_pitch * c_yaw,
        )
    }

    pub fn view_matrix(&self) -> Mat4 {
        let eye = self.eye_position();
        let target = self.pan;
        Mat4::look_at(eye, target, Vec3::UP)
    }

    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        let fov_rad = self.fov_deg.to_radians();
        Mat4::perspective(fov_rad, aspect, 0.1, 1000.0)
    }
}

pub struct Renderer3D {
    pub camera: Camera3D,
    pub key_light_dir: Vec3,
    pub fill_light_dir: Vec3,
    pub ambient: f32,
    pub wireframe: bool,
    pub auto_rotate: bool,
}

impl Default for Renderer3D {
    fn default() -> Self {
        Self {
            camera: Camera3D::default(),
            key_light_dir: Vec3::new(0.45, 0.85, 0.50).normalized(),
            fill_light_dir: Vec3::new(-0.55, 0.35, -0.45).normalized(),
            ambient: 0.40,
            wireframe: false,
            auto_rotate: false,
        }
    }
}

struct ProjectedTriangle {
    screen_pts: [Pos2; 3],
    depth: f32,
    layer: u8,
    color: Color32,
}

impl Renderer3D {
    pub fn update_animation(&mut self, dt_sec: f32) {
        if self.auto_rotate {
            self.camera.yaw += 0.35 * dt_sec;
            if self.camera.yaw > std::f32::consts::TAU {
                self.camera.yaw -= std::f32::consts::TAU;
            }
        }
    }

    pub fn render(&self, painter: &Painter, rect: Rect, mesh: &Mesh3D) {
        let aspect = rect.width() / rect.height().max(1.0);
        let view = self.camera.view_matrix();
        let proj = self.camera.proj_matrix(aspect);
        let mvp = proj.mul(&view);
        let eye = self.camera.eye_position();

        // Project vertices
        let mut proj_verts: Vec<Option<(Pos2, f32)>> = Vec::with_capacity(mesh.vertices.len());
        for v in &mesh.vertices {
            let (clip, w) = mvp.transform_point(v.pos);
            if w > 0.05 {
                let ndc_x = clip.x / w;
                let ndc_y = clip.y / w;
                let ndc_z = clip.z / w;

                let scr_x = rect.left() + (ndc_x + 1.0) * 0.5 * rect.width();
                let scr_y = rect.top() + (1.0 - ndc_y) * 0.5 * rect.height();
                proj_verts.push(Some((Pos2::new(scr_x, scr_y), ndc_z)));
            } else {
                proj_verts.push(None);
            }
        }

        // Build list of drawable triangles
        let mut draw_tris: Vec<ProjectedTriangle> = Vec::with_capacity(mesh.faces.len() * 2);

        for face in &mesh.faces {
            if face.indices.len() < 3 {
                continue;
            }
            let i0 = face.indices[0];
            let i1 = face.indices[1];
            let i2 = face.indices[2];

            if let (Some((p0, z0)), Some((p1, z1)), Some((p2, z2))) = (proj_verts[i0], proj_verts[i1], proj_verts[i2]) {
                // Backface culling: signed 2D screen area
                let area = (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x);
                if area <= 0.0 && !face.emissive {
                    continue;
                }

                // Shading calculation
                let base_col = mesh.vertices[i0].color;
                let final_col = if face.emissive {
                    base_col
                } else {
                    let diff_key = face.normal.dot(self.key_light_dir).max(0.0);
                    let diff_fill = face.normal.dot(self.fill_light_dir).max(0.0) * 0.35;
                    let intensity = (self.ambient + (1.0 - self.ambient) * (diff_key + diff_fill)).min(1.0);

                    // Specular highlight
                    let view_dir = (eye - mesh.vertices[i0].pos).normalized();
                    let half_vec = (self.key_light_dir + view_dir).normalized();
                    let spec = face.normal.dot(half_vec).max(0.0).powi(16) * 0.25;

                    let r = ((base_col.r() as f32 * intensity + spec * 255.0).min(255.0)) as u8;
                    let g = ((base_col.g() as f32 * intensity + spec * 255.0).min(255.0)) as u8;
                    let b = ((base_col.b() as f32 * intensity + spec * 255.0).min(255.0)) as u8;

                    Color32::from_rgba_premultiplied(r, g, b, base_col.a())
                };

                let avg_depth = (z0 + z1 + z2) / 3.0;
                draw_tris.push(ProjectedTriangle {
                    screen_pts: [p0, p1, p2],
                    depth: avg_depth,
                    layer: face.layer,
                    color: final_col,
                });
            }
        }

        // Layer-Aware Sorting:
        // Lower layer (e.g. PCB) drawn FIRST, higher layer (e.g. Screen content, LEDs) drawn AFTER.
        // Within the same layer, sort from farthest depth to nearest depth.
        draw_tris.sort_by(|a, b| {
            a.layer.cmp(&b.layer).then_with(|| {
                b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        // Submit to egui mesh
        let mut egui_mesh = Mesh::default();
        let mut idx = 0u32;

        for tri in draw_tris {
            egui_mesh.vertices.push(egui::epaint::Vertex {
                pos: tri.screen_pts[0],
                uv: Pos2::ZERO,
                color: tri.color,
            });
            egui_mesh.vertices.push(egui::epaint::Vertex {
                pos: tri.screen_pts[1],
                uv: Pos2::ZERO,
                color: tri.color,
            });
            egui_mesh.vertices.push(egui::epaint::Vertex {
                pos: tri.screen_pts[2],
                uv: Pos2::ZERO,
                color: tri.color,
            });

            egui_mesh.indices.extend_from_slice(&[idx, idx + 1, idx + 2]);
            idx += 3;
        }

        painter.add(egui::Shape::mesh(egui_mesh));

        // Wireframe edges overlay
        if self.wireframe {
            let line_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 210, 255, 140));
            for &(i0, i1, _) in &mesh.lines {
                if let (Some((p0, _)), Some((p1, _))) = (proj_verts[i0], proj_verts[i1]) {
                    painter.line_segment([p0, p1], line_stroke);
                }
            }
        }
    }
}
