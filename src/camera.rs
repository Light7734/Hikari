use crate::ray::Ray;
use crate::vec3::Vec3;

use Vec3 as Point3;

pub struct Camera {
    pub origin: Point3,
    pub lower_left_corner: Point3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
}

impl Camera {
    pub fn new() -> Camera {
        let aspect_ratio = 16.0 / 9.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * aspect_ratio;
        let focal_length = 1.0;

        let origin = Point3::new(0.0, 0.0, 0.0);
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);

        return Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: -(horizontal / 2.0)
                - (vertical / 2.0)
                - Vec3::new(0.0, 0.0, focal_length),
        };
    }
    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray {
            orig: self.origin,
            dir: self.lower_left_corner + (u * self.horizontal) + (v * self.vertical) - self.origin,
        }
    }
}
