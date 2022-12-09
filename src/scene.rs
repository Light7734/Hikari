use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec3::Vec3;

use Vec3 as Color;
use Vec3 as Point3;

pub struct Scene {
    pub hittables: Vec<Box<dyn Hittable>>,
}

pub fn lerp<T>(a: T, b: T, val: f64) -> T
where
    T: std::ops::Mul<f64, Output = T>,
    T: std::ops::Add<T, Output = T>,
{
    (a * (1.0 - val)) + (b * val)
}

impl Scene {
    pub fn add(&mut self, hittable: Box<dyn Hittable>) {
        self.hittables.push(hittable);
    }

    pub fn clear(&mut self) {
        self.hittables.clear();
        self.hittables.shrink_to(10);
    }

    pub fn process_ray(&self, ray: &Ray, t_min: f64, t_max: f64) -> Color {
        let mut closest_rec = HitRecord {
            normal: Vec3::new(0.0, 0.0, 0.0),
            point: Point3::new(0.0, 0.0, 0.0),
            t: t_max,
            front_face: false,
        };

        let mut temp_rec: HitRecord = closest_rec.clone();
        for hittable in self.hittables.iter() {
            if hittable.hit(ray, t_min, closest_rec.t, &mut temp_rec) {
                closest_rec = temp_rec;
            }
        }

        match closest_rec.normal != Vec3::new(0.0, 0.0, 0.0) {
            true => return 0.5 * (closest_rec.normal + Color::new(1.0, 1.0, 1.0)),

            false => {
                let unit_y = ray.dir.unit().y;

                lerp::<Color>(
                    Color::new(1.0, 1.0, 1.0),
                    Color::new(0.5, 0.7, 0.9),
                    0.5 * (unit_y + 1.0), // map unit_y from [-1.0 -> 1.0] to [0.0 -> 1.0]
                )
            }
        }
    }
}
