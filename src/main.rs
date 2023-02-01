pub mod raytracer;
pub mod vec3;

use crate::raytracer::*;
use crate::vec3::Vec3;
use rand::distributions::{Distribution, Uniform};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn rand() -> f32 {
    Uniform::from(0.0..1.0).sample(&mut rand::thread_rng())
}

fn main() {
    // Image
    let aspect_ratio: f32 = 3.0 / 2.0;
    let image_width = (720.0 * aspect_ratio) as u32;
    let image_height = (image_width as f32 / aspect_ratio) as u32;

    let lookfrom = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let vfov: f32 = 20.0;
    let aperture: f32 = 0.1;

    let theta = f32::to_radians(vfov);
    let h = f32::tan(theta / 2.0);
    let viewport_height = 2.0 * h;
    let viewport_width = aspect_ratio * viewport_height;

    let w = (lookfrom - lookat).unit();
    let u = vup.cross(&w).unit();
    let v = w.cross(&u);

    let focus_dist = 10.0;

    let horizontal = focus_dist * u * viewport_width;
    let vertical = focus_dist * v * viewport_height;

    let camera = Camera {
        origin: lookfrom,
        horizontal,
        vertical,
        lower_left_corner: lookfrom - (horizontal / 2.0) - (vertical / 2.0) - focus_dist * w,
        up: vup,
        u,
        v,
        w,
        lens_radius: aperture / 2.0,
        ..Default::default()
    };

    let mut spheres = [
        Sphere {
            center: Vec3::new(0.0, -1000.0, -1.0),
            radius: 1000.0,
            mat_type: 0,
            albedo: Vec3::new(0.5, 0.5, 0.5),
            ..Default::default()
        },
        Sphere {
            center: Vec3::new(0.0, 1.0, 0.0),
            radius: 1.0,
            mat_type: 2,
            fuzz_or_ir: 1.5,
            ..Default::default()
        },
        Sphere {
            center: Vec3::new(-4.0, 1.0, 0.0),
            radius: 1.0,
            mat_type: 0,
            albedo: Vec3::new(0.4, 0.2, 0.1),
            ..Default::default()
        },
        Sphere {
            center: Vec3::new(4.0, 1.0, 0.0),
            radius: 1.0,
            fuzz_or_ir: 0.0,
            mat_type: 1,
            albedo: Vec3::new(0.7, 0.6, 0.5),
            ..Default::default()
        },
    ]
    .to_vec();

    for a in -11..11 {
        for b in -11..11 {
            let mat = rand();
            let center = Vec3::new(a as f32 + 0.9 * rand(), 0.2, b as f32 + 0.9 * rand());

            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if mat < 0.8 {
                    spheres.push(Sphere {
                        center,
                        radius: 0.2,
                        mat_type: 0,
                        albedo: Vec3::new(rand(), rand(), rand()),
                        ..Default::default()
                    })
                } else if mat < 0.95 {
                    spheres.push(Sphere {
                        center,
                        radius: 0.2,
                        mat_type: 1,
                        albedo: Vec3::new(
                            0.5 + rand() / 2.0,
                            0.5 + rand() / 2.0,
                            0.5 + rand() / 2.0,
                        ),
                        fuzz_or_ir: rand() / 2.0,
                        ..Default::default()
                    })
                } else {
                    spheres.push(Sphere {
                        center,
                        radius: 0.2,
                        mat_type: 2,
                        fuzz_or_ir: 1.5,
                        ..Default::default()
                    })
                }
            }
        }
    }

    let config = Config {
        num_spheres: spheres.len() as u32,
        sample_count: 32,
        max_bounces: 4,
        width: image_width,
        height: image_height,

        camera,
        ..Default::default()
    };

    let mut raytracer = Raytracer::new(config, spheres);
    let output = raytracer.raytrace();

    let mut out_string: String = String::new();
    out_string.push_str(&format!("P3\n{} {}\n255\n", image_width, image_height));
    for i in output.iter() {
        out_string.push_str(i.to_string().as_str());
        out_string.push(' ');
    }

    let path = Path::new("a.bpp");
    match File::create(&path) {
        Err(why) => panic!("Failed to create {}: {}", path.display(), why),
        Ok(mut file) => match file.write_all(out_string.as_bytes()) {
            Err(why) => panic!("Failed to write to {}: {}", path.display(), why),
            Ok(_) => {}
        },
    };
}
