pub mod hittable;
pub mod scene;
pub mod vec3;

use crate::vec3::Vec3;
use Vec3 as Point3;
use Vec3 as Color;

pub mod color;
use crate::color::write_color;

pub mod ray;
use crate::ray::Ray;

use crate::hittable::{Hittable, Sphere};
use crate::scene::Scene;

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = ((image_width as f64) / aspect_ratio) as i32;

    // Camera
    let viewport_height = 2.0;
    let viewport_width = viewport_height * aspect_ratio;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - (horizontal / 2.0) - (vertical / 2.0) - Vec3::new(0.0, 0.0, focal_length);

    print!("P3\n{} {}\n255\n", image_width, image_height);

    let mut scene = Scene {
        hittables: Vec::new(),
    };

    scene.add(Box::new(Sphere {
        center: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    }));

    scene.add(Box::new(Sphere {
        center: Point3::new(0.0, -100.5, -1.0),
        radius: 100.0,
    }));

    for j in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {} ", image_height - j);
        for i in 0..image_width {
            let u = (i as f64) / ((image_width - 1) as f64);
            let v = (j as f64) / ((image_height - 1) as f64);
            let ray = Ray::new(
                origin,
                (u * horizontal) + (v * vertical) + lower_left_corner - origin,
            );

            write_color(scene.process_ray(&ray, 0.0, f64::INFINITY));
        }
    }
}
