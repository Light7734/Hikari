use bytemuck::{Pod, Zeroable};
use display_json::DebugAsJsonPretty;
use rand::distributions::{Distribution, Uniform};
use serde::Serialize;
use std::ops;

#[derive(Copy, Clone, PartialEq, Zeroable, Pod, Default, Serialize, DebugAsJsonPretty)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn length(&self) -> f32 {
        return self.length_squared().sqrt();
    }

    pub fn length_squared(&self) -> f32 {
        return (self.x * self.x) + (self.y * self.y) + (self.z * self.z);
    }

    pub fn dot(&self, rhs: &Vec3) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn unit(&self) -> Vec3 {
        *self / self.length()
    }

    pub fn random_in_bounds(uniform: &Uniform<f32>) -> Vec3 {
        Vec3 {
            x: uniform.sample(&mut rand::thread_rng()),
            y: uniform.sample(&mut rand::thread_rng()),
            z: uniform.sample(&mut rand::thread_rng()),
        }
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        let uniform_sampler = Uniform::from(0.0..1.0);
        loop {
            let p = Vec3::random_in_bounds(&uniform_sampler);

            if p.length_squared() >= 1.0 {
                continue;
            }

            return p;
        }
    }

    pub fn random_in_hemisphere(normal: &Vec3) -> Vec3 {
        let in_unit_sphere = Vec3::random_in_unit_sphere();

        if in_unit_sphere.dot(normal) > 0.0 {
            in_unit_sphere
        } else {
            -in_unit_sphere
        }
    }

    pub fn random_in_unit_disk() -> Vec3 {
        let uniform_sampler = Uniform::from(-1.0..1.0);
        loop {
            let p = Vec3::new(
                uniform_sampler.sample(&mut rand::thread_rng()),
                uniform_sampler.sample(&mut rand::thread_rng()),
                0.0,
            );

            if p.length_squared() >= 1.0 {
                continue;
            } else {
                return p;
            };
        }
    }

    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        *self - *normal * (self.dot(normal) * 2.0)
    }

    pub fn is_near_zero(&self) -> bool {
        f32::abs(self.x) < 1e-8 && f32::abs(self.y) < 1e-8 && f32::abs(self.z) < 1e-8
    }

    pub fn refract(&self, normal: &Vec3, etai_over_etat: f32) -> Vec3 {
        let cos_theta = f32::min((-(*self)).dot(normal), 1.0);

        let r_out_perp = etai_over_etat * (*self + *normal * cos_theta);
        let r_out_parallel = *normal * -f32::sqrt(f32::abs(1.0 - r_out_perp.length_squared()));

        return r_out_perp + r_out_parallel;
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        (1.0 / rhs) * self
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
