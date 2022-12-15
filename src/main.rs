pub mod camera;
pub mod color;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod scene;
pub mod vec3;

use std::io::prelude::*;

use pbr::MultiBar;
use std::fs::File;
use std::path::Path;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

use std::rc::Rc;

use crate::material::{Dielectric, Lambertian, Material, Metal};

use crate::vec3::Vec3;
use Vec3 as Point3;
use Vec3 as Color;

use crate::color::write_color;

use crate::ray::Ray;

use crate::camera::Camera;
use crate::hittable::{Hittable, Sphere};
use crate::scene::Scene;

use rand::distributions::{Distribution, Uniform};

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 800;
    let image_height = (image_width as f64 / aspect_ratio) as i32;

    let sample_count = 100;
    let scale = 1.0 / sample_count as f64;

    let between = Uniform::from(0.0..1.0);

    let num_threads = 18;

    let row_per_thread = f64::ceil(image_height as f64 / (num_threads as f64)) as i32;

    let camera = Arc::new(Camera::new());

    // materials
    let material_ground = Arc::new(Lambertian {
        albedo: Color::new(0.8, 0.8, 0.0),
    });

    let material_center = Arc::new(Dielectric { ir: 1.5 });

    let material_left = Arc::new(Dielectric { ir: 1.5 });

    let material_right = Arc::new(Metal {
        albedo: Color::new(0.8, 0.6, 0.2),
        fuzz: 1.0,
    });

    let sphere_ground = Box::new(Sphere {
        center: Point3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: Some(material_ground),
    }) as Box<dyn Hittable + Send + Sync + 'static>;

    // scene
    let scene = Arc::new(RwLock::new(Scene {
        hittables: Vec::new(),
    }));

    let mut mutable_scene = scene.write().unwrap();
    mutable_scene.add(sphere_ground);

    mutable_scene.add(Box::new(Sphere {
        center: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material: Some(material_center),
    }));

    mutable_scene.add(Box::new(Sphere {
        center: Point3::new(-1.0, 0.0, -1.0),
        radius: 0.5,
        material: Some(material_left.clone()),
    }));

    mutable_scene.add(Box::new(Sphere {
        center: Point3::new(-1.0, 0.0, -1.0),
        radius: -0.4,
        material: Some(material_left),
    }));

    mutable_scene.add(Box::new(Sphere {
        center: Point3::new(1.0, 0.0, -1.0),
        radius: 0.5,
        material: Some(material_right),
    }));
    drop(mutable_scene);

    let max_bounces = 50;
    let mut mb = MultiBar::new();

    let (tx, rx) = mpsc::channel();
    for t in 0..num_threads {
        let t_scoped = tx.clone();

        let start = t * row_per_thread;

        let s_scene = Arc::clone(&scene);
        let s_camera = Arc::clone(&camera);

        let mut bar = mb.create_bar(row_per_thread as u64);
        bar.show_message = true;
        bar.show_speed = false;
        bar.show_percent = false;

        bar.message(&format!(
            "[{}] Rows {} -> {} ==> ",
            t,
            start,
            start + row_per_thread
        ));

        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut results: Vec<String> = Vec::new();

            for j in (start..start + row_per_thread).rev() {
                for i in 0..image_width {
                    let mut color = Color::new(0.0, 0.0, 0.0);
                    for _ in 0..sample_count {
                        let u = ((i as f64) + (between.sample(&mut rng) as f64))
                            / ((image_width - 1) as f64);
                        let v = ((j as f64) + (between.sample(&mut rng) as f64))
                            / ((image_height - 1) as f64);

                        color += s_scene
                            .read()
                            .unwrap()
                            .process_ray(&mut s_camera.get_ray(u, v), 50);
                    }

                    results.push(format!(
                        "{} {} {}\n",
                        (256.0 * f64::clamp(f64::sqrt(color.x * scale), 0.0, 0.99)) as i32,
                        (256.0 * f64::clamp(f64::sqrt(color.y * scale), 0.0, 0.99)) as i32,
                        (256.0 * f64::clamp(f64::sqrt(color.z * scale), 0.0, 0.99)) as i32,
                    ));
                }
                bar.inc();
            }

            match t_scoped.send((t, results)) {
                Err(why) => panic!("{}", why),
                Ok(_) => {}
            };

            bar.finish();
            drop(t_scoped);
        });
    }
    drop(tx);

    mb.listen();
    let mut row_packs: Vec<(i32, Vec<String>)> = Vec::new();
    for recieved in rx {
        row_packs.push(recieved);
    }

    row_packs.sort_by(|(a, _), (c, _)| c.cmp(a));

    let path = Path::new("a.bpp");
    let mut file = match File::create(&path) {
        Err(why) => panic!("Failed to create {}: {}", path.display(), why),
        Ok(file) => file,
    };

    match file.write_all(format!("P3\n{} {}\n255\n", image_width, image_height).as_bytes()) {
        Err(why) => panic!("Failed to write to {}: {}", path.display(), why),
        Ok(_) => {}
    }

    for (_index, pixel_packs) in row_packs {
        for pixels in pixel_packs {
            match file.write_all(pixels.as_bytes()) {
                Err(why) => panic!("Failed to write to {}: {}", path.display(), why),
                Ok(_) => {}
            }
        }
    }
}
