use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec3::Vec3;

use Vec3 as Color;
use Vec3 as Point3;

pub struct Scene {
    pub hittables: Vec<Box<dyn Hittable + Send + Sync + 'static>>,
}

pub fn lerp<T>(a: T, b: T, val: f64) -> T
where
    T: std::ops::Mul<f64, Output = T>,
    T: std::ops::Add<T, Output = T>,
{
    (a * (1.0 - val)) + (b * val)
}

impl Scene {
    pub fn add(&mut self, hittable: Box<dyn Hittable + Send + Sync + 'static>) {
        self.hittables.push(hittable);
    }

    pub fn clear(&mut self) {
        self.hittables.clear();
        self.hittables.shrink_to(10);
    }

    pub fn process_ray(&self, ray: &Ray, max_bounce: u32) -> Color {
        if max_bounce <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let mut closest_rec = HitRecord {
            normal: Vec3::new(0.0, 0.0, 0.0),
            point: Point3::new(0.0, 0.0, 0.0),
            t: f64::INFINITY,
            front_face: false,
            material: None,
        };

        for hittable in self.hittables.iter() {
            let mut temp_rec = HitRecord {
                normal: Vec3::new(0.0, 0.0, 0.0),
                point: Point3::new(0.0, 0.0, 0.0),
                t: f64::INFINITY,
                front_face: false,
                material: None,
            };

            if hittable.hit(ray, 0.001, closest_rec.t, &mut temp_rec) {
                closest_rec = temp_rec;
            }
        }

        let target = closest_rec.point + Vec3::random_in_hemisphere(&closest_rec.normal);
        let unit_y = ray.dir.unit().y;

        match closest_rec.material.is_some() {
            true => {
                let (scattered, attenuation, scatter_ray) = closest_rec
                    .material
                    .as_ref()
                    .unwrap()
                    .scatter(ray, &closest_rec);

                return if scattered {
                    attenuation * self.process_ray(&scatter_ray, max_bounce - 1)
                } else {
                    Color::new(0.0, 0.0, 0.0)
                };
            }

            false => {
                return lerp::<Color>(
                    Color::new(1.0, 1.0, 1.0),
                    Color::new(0.5, 0.7, 0.9),
                    0.5 * (unit_y + 1.0), // map unit_y from [-1.0 -> 1.0] to [0.0 -> 1.0]
                );
            }
        }
    }
}
