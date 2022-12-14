pub mod camera;
pub mod color;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod scene;
pub mod vec3;

use std::rc::Rc;

use crate::material::{Lambertian, Material, Metal};

use crate::vec3::Vec3;
use Vec3 as Point3;
use Vec3 as Color;

use crate::color::write_color;

use crate::ray::Ray;

use crate::camera::Camera;
use crate::hittable::Sphere;
use crate::scene::Scene;

use rand::distributions::{Distribution, Uniform};

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = ((image_width as f64) / aspect_ratio) as i32;

    let camera = Camera::new();

    print!("P3\n{} {}\n255\n", image_width, image_height);

    let mut scene = Scene {
        hittables: Vec::new(),
    };

    let material_ground = Rc::new(Lambertian {
        albedo: Color::new(0.8, 0.8, 0.0),
    });

    let material_center = Rc::new(Lambertian {
        albedo: Color::new(0.7, 0.3, 0.3),
    });
    let material_left = Rc::new(Metal {
        albedo: Color::new(0.8, 0.8, 0.8),
        fuzz: 0.3
    });
    let material_right = Rc::new(Metal {
        albedo: Color::new(0.8, 0.6, 0.2),
        fuzz: 1.0
    });

    scene.add(Box::new(Sphere {
        center: Point3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: Some(material_ground),
    }));

    scene.add(Box::new(Sphere {
        center: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material: Some(material_center),
    }));

    scene.add(Box::new(Sphere {
        center: Point3::new(-1.0, 0.0, -1.0),
        radius: 0.5,
        material: Some(material_right),
    }));

    scene.add(Box::new(Sphere {
        center: Point3::new(1.0, 0.0, -1.0),
        radius: 0.5,
        material: Some(material_left),
    }));

    let sample_count = 64;
    let between = Uniform::from(0.0..1.0);
    let mut rng = rand::thread_rng();

    for j in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {} ", j);
        for i in 0..image_width {
            let mut color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..sample_count {
                let u =
                    ((i as f64) + (between.sample(&mut rng) as f64)) / ((image_width - 1) as f64);
                let v =
                    ((j as f64) + (between.sample(&mut rng) as f64)) / ((image_height - 1) as f64);

                color += scene.process_ray(&mut camera.get_ray(u, v), 8);
            }

            write_color(color, sample_count);
        }
    }
}
