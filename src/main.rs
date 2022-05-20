pub mod vec3;
use crate::vec3::Vec3;
use Vec3 as Point3;
use Vec3 as Color;

pub mod color;
use crate::color::write_color;

pub mod ray;
use crate::ray::Ray;

pub fn hit_sphere(center: &Point3, radius: f64, ray: &Ray) -> bool {
    let oc = ray.orig - *center;
    let a = vec3::dot(&ray.dir, &ray.dir);
    let b = 2.0 * vec3::dot(&oc, &ray.dir);
    let c = vec3::dot(&oc, &oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    discriminant > 0.0
}

// Raytrace !
pub fn ray_color(ray: &Ray) -> Color {
    if hit_sphere(&Point3::new(0.0, 0.0, -1.0), 0.5, ray) {
        Color::new(1.0, 0.0, 0.0)
    } else {
        let unit_dir: Vec3 = vec3::unit_vec(ray.dir); // turn ray direction(ending point) into unit vector, making Y axis to be in range[-1.0 -> 1.0]
        let lerp_val = 0.5 * (unit_dir.y + 1.0); // turn lerp_value(Y Direction) into unit length [0 -> 1]

        // lerp (linear-interpolate) between colorA -> colorB
        // finalColor = ((1.0 - lerp_val) * colorA) + ((lerp_val) * colorB)
        ((1.0 - lerp_val) * Color::new(1.0, 1.0, 1.0)) + (lerp_val * Color::new(0.5, 0.7, 0.9))
    }
}

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

    for j in 0..image_height {
        eprint!("\rScanlines remaining: {} ", image_height - j);
        for i in 0..image_width {
            let u = (i as f64) / ((image_width - 1) as f64);
            let v = (j as f64) / ((image_height - 1) as f64);
            let ray = Ray::new(
                origin,
                (u * horizontal) + (v * vertical) + lower_left_corner - origin,
            );

            write_color(ray_color(&ray));
        }
    }
}
