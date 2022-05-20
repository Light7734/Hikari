use crate::vec3::Vec3;

use Vec3 as Color;

pub fn write_color(color: Color) {
    print!(
        "{} {} {}\n",
        (255.999 * color.x) as i32,
        (255.999 * color.y) as i32,
        (255.999 * color.z) as i32,
    );
}
