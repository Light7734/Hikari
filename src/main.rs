pub mod color;
pub mod vec3;
use crate::color::write_color;
use crate::vec3::Vec3;

use Vec3 as Point3;
use Vec3 as Color;

fn main() {
    let image_width = 256;
    let image_height = 256;

    print!("P3\n{} {}\n255\n", image_width, image_height);

    for j in 0..image_height {
        eprint!("\rScanlines remaining: {} ", image_height - j);
        for i in 0..image_width {
            let r = (i as f64) / ((image_width - 1) as f64);
            let g = (j as f64) / ((image_height - 1) as f64);
            let b = 0.25;

            let color: Color = Color::new(
                (i as f64) / (image_height) as f64,
                (j as f64) / (image_width) as f64,
                0.25,
            );

            write_color(color);
        }
    }
}
