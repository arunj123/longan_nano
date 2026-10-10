use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_sq().sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len > 1e-6 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        m: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    pub fn mul(&self, rhs: &Self) -> Self {
        let mut out = [0.0; 16];
        for r in 0..4 {
            for c in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[r * 4 + k] * rhs.m[k * 4 + c];
                }
                out[r * 4 + c] = sum;
            }
        }
        Self { m: out }
    }

    pub fn transform_point(&self, p: Vec3) -> (Vec3, f32) {
        let x = self.m[0] * p.x + self.m[1] * p.y + self.m[2] * p.z + self.m[3];
        let y = self.m[4] * p.x + self.m[5] * p.y + self.m[6] * p.z + self.m[7];
        let z = self.m[8] * p.x + self.m[9] * p.y + self.m[10] * p.z + self.m[11];
        let w = self.m[12] * p.x + self.m[13] * p.y + self.m[14] * p.z + self.m[15];
        (Vec3::new(x, y, z), w)
    }

    pub fn translation(tx: f32, ty: f32, tz: f32) -> Self {
        let mut res = Self::IDENTITY;
        res.m[3] = tx;
        res.m[7] = ty;
        res.m[11] = tz;
        res
    }

    pub fn rotation_x(rad: f32) -> Self {
        let (s, c) = rad.sin_cos();
        let mut res = Self::IDENTITY;
        res.m[5] = c;
        res.m[6] = -s;
        res.m[9] = s;
        res.m[10] = c;
        res
    }

    pub fn rotation_y(rad: f32) -> Self {
        let (s, c) = rad.sin_cos();
        let mut res = Self::IDENTITY;
        res.m[0] = c;
        res.m[2] = s;
        res.m[8] = -s;
        res.m[10] = c;
        res
    }

    pub fn rotation_z(rad: f32) -> Self {
        let (s, c) = rad.sin_cos();
        let mut res = Self::IDENTITY;
        res.m[0] = c;
        res.m[1] = -s;
        res.m[4] = s;
        res.m[5] = c;
        res
    }

    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        let mut res = Self::IDENTITY;
        res.m[0] = sx;
        res.m[5] = sy;
        res.m[10] = sz;
        res
    }

    pub fn perspective(fov_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let tan_half_fov = (fov_rad / 2.0).tan();
        let mut res = Self { m: [0.0; 16] };
        res.m[0] = 1.0 / (aspect * tan_half_fov);
        res.m[5] = 1.0 / tan_half_fov;
        res.m[10] = -(far + near) / (far - near);
        res.m[11] = -(2.0 * far * near) / (far - near);
        res.m[14] = -1.0;
        res
    }

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalized();
        let s = f.cross(up).normalized();
        let u = s.cross(f);

        let mut res = Self::IDENTITY;
        res.m[0] = s.x;
        res.m[1] = s.y;
        res.m[2] = s.z;
        res.m[3] = -s.dot(eye);

        res.m[4] = u.x;
        res.m[5] = u.y;
        res.m[6] = u.z;
        res.m[7] = -u.dot(eye);

        res.m[8] = -f.x;
        res.m[9] = -f.y;
        res.m[10] = -f.z;
        res.m[11] = f.dot(eye);
        res
    }
}
