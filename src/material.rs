use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec3::Vec3;

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
