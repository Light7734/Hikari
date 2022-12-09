use crate::vec3::Vec3;

use Vec3 as Color;

pub fn write_color(color: Color, sample_count: i32) {
    let scale = 1.0 / sample_count as f64;

    print!(
        "{} {} {}\n",
        (256.0 * f64::clamp(color.x * scale, 0.0, 0.99)) as i32,
        (256.0 * f64::clamp(color.y * scale, 0.0, 0.99)) as i32,
        (256.0 * f64::clamp(color.z * scale, 0.0, 0.99)) as i32,
    );
}
