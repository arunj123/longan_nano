use egui::Color32;
use crate::graphics3d::math3d::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Color32,
    pub normal: Vec3,
}

impl Vertex {
    pub fn new(pos: Vec3, color: Color32, normal: Vec3) -> Self {
        Self { pos, color, normal }
    }
}

#[derive(Debug, Clone)]
pub struct Face {
    pub indices: Vec<usize>,
    pub normal: Vec3,
    pub emissive: bool,
    pub layer: u8, // Layer 0: ground, 1: board, 2: parts, 3: screen bezel, 4: screen decals, 5: LEDs/wires
}

#[derive(Debug, Clone, Default)]
pub struct Mesh3D {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
    pub lines: Vec<(usize, usize, Color32)>, // Wireframe line segments
}

impl Mesh3D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.faces.clear();
        self.lines.clear();
    }

    pub fn add_triangle_layered(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, emissive: bool, layer: u8) {
        let n = (v1.pos - v0.pos).cross(v2.pos - v0.pos).normalized();
        let i0 = self.vertices.len();
        self.vertices.push(v0);
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.faces.push(Face {
            indices: vec![i0, i0 + 1, i0 + 2],
            normal: n,
            emissive,
            layer,
        });
    }

    pub fn add_triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, emissive: bool) {
        self.add_triangle_layered(v0, v1, v2, emissive, 1);
    }

    pub fn add_quad_layered(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, v3: Vertex, emissive: bool, layer: u8) {
        let n = (v1.pos - v0.pos).cross(v2.pos - v0.pos).normalized();
        let i0 = self.vertices.len();
        self.vertices.push(v0);
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v3);
        // Split quad into 2 triangles with consistent winding
        self.faces.push(Face {
            indices: vec![i0, i0 + 1, i0 + 2],
            normal: n,
            emissive,
            layer,
        });
        self.faces.push(Face {
            indices: vec![i0, i0 + 2, i0 + 3],
            normal: n,
            emissive,
            layer,
        });
    }

    pub fn add_quad(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, v3: Vertex, emissive: bool) {
        self.add_quad_layered(v0, v1, v2, v3, emissive, 1);
    }

    pub fn add_line(&mut self, p0: Vec3, p1: Vec3, color: Color32) {
        let i0 = self.vertices.len();
        self.vertices.push(Vertex::new(p0, color, Vec3::UP));
        self.vertices.push(Vertex::new(p1, color, Vec3::UP));
        self.lines.push((i0, i0 + 1, color));
    }

    pub fn add_box_layered(&mut self, center: Vec3, size: Vec3, color: Color32, layer: u8) {
        let hx = size.x * 0.5;
        let hy = size.y * 0.5;
        let hz = size.z * 0.5;

        let c = center;
        let p000 = Vec3::new(c.x - hx, c.y - hy, c.z - hz);
        let p100 = Vec3::new(c.x + hx, c.y - hy, c.z - hz);
        let p110 = Vec3::new(c.x + hx, c.y + hy, c.z - hz);
        let p010 = Vec3::new(c.x - hx, c.y + hy, c.z - hz);
        let p001 = Vec3::new(c.x - hx, c.y - hy, c.z + hz);
        let p101 = Vec3::new(c.x + hx, c.y - hy, c.z + hz);
        let p111 = Vec3::new(c.x + hx, c.y + hy, c.z + hz);
        let p011 = Vec3::new(c.x - hx, c.y + hy, c.z + hz);

        // Front (Z + hz)
        self.add_quad_layered(
            Vertex::new(p001, color, Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(p101, color, Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(p111, color, Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(p011, color, Vec3::new(0.0, 0.0, 1.0)),
            false,
            layer,
        );

        // Back (Z - hz)
        self.add_quad_layered(
            Vertex::new(p100, color, Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(p000, color, Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(p010, color, Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(p110, color, Vec3::new(0.0, 0.0, -1.0)),
            false,
            layer,
        );

        // Top (Y + hy) - normal points UP
        self.add_quad_layered(
            Vertex::new(p010, color, Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(p110, color, Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(p111, color, Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(p011, color, Vec3::new(0.0, 1.0, 0.0)),
            false,
            layer,
        );

        // Bottom (Y - hy) - normal points DOWN
        self.add_quad_layered(
            Vertex::new(p000, color, Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(p001, color, Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(p101, color, Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(p100, color, Vec3::new(0.0, -1.0, 0.0)),
            false,
            layer,
        );

        // Right (X + hx) - normal points RIGHT
        self.add_quad_layered(
            Vertex::new(p100, color, Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(p101, color, Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(p111, color, Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(p110, color, Vec3::new(1.0, 0.0, 0.0)),
            false,
            layer,
        );

        // Left (X - hx) - normal points LEFT
        self.add_quad_layered(
            Vertex::new(p000, color, Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(p010, color, Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(p011, color, Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(p001, color, Vec3::new(-1.0, 0.0, 0.0)),
            false,
            layer,
        );
    }

    pub fn add_box(&mut self, center: Vec3, size: Vec3, color: Color32) {
        self.add_box_layered(center, size, color, 1);
    }
}
