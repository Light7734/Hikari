use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec3::Vec3;
use rand::distributions::{Distribution, Uniform};

use Vec3 as Color;

pub trait Material {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> (bool, Color, Ray);
}

pub struct Lambertian {
    pub albedo: Color,
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

pub struct Dielectric {
    // index of refraction
    pub ir: f64,
}

impl Dielectric {
    pub fn reflectance(&self, cosine: f64, ref_idx: f64) -> f64 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        return r0 + (1.0 - r0) * f64::powf(1.0 - cosine, 5.0);
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> (bool, Color, Ray) {
        let mut scatter_dir = hit_record.normal + Vec3::random_in_unit_sphere().unit();

        if scatter_dir.is_near_zero() {
            scatter_dir = hit_record.normal;
        };

        (
            true,
            self.albedo,
            Ray {
                orig: hit_record.point,
                dir: scatter_dir,
            },
        )
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> (bool, Color, Ray) {
        let reflected = ray.dir.unit().reflect(&hit_record.normal);

        (
            reflected.dot(&hit_record.normal) > 0.0,
            self.albedo,
            Ray {
                orig: hit_record.point,
                dir: reflected + (self.fuzz * Vec3::random_in_unit_sphere()),
            },
        )
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> (bool, Color, Ray) {
        let uniform_sampler = Uniform::from(0.0..1.0);

        let refraction_ratio: f64 = if hit_record.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_dir = ray.dir.unit();

        let cos_theta = f64::min((-unit_dir).dot(&hit_record.normal), 1.0);
        let sin_theta = f64::sqrt(1.0 - (cos_theta * cos_theta));

        let direction = if refraction_ratio * sin_theta > 1.0
            || self.reflectance(cos_theta, refraction_ratio)
                > uniform_sampler.sample(&mut rand::thread_rng())
        {
            unit_dir.reflect(&hit_record.normal)
        } else {
            unit_dir.refract(&hit_record.normal, refraction_ratio)
        };

        (
            true,
            Color::new(1.0, 1.0, 1.0),
            Ray {
                orig: hit_record.point,
                dir: direction,
            },
        )
    }
}
