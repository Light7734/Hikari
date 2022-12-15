use crate::ray::Ray;
use crate::vec3::Vec3;

use Vec3 as Point3;

pub struct Camera {
    pub origin: Point3,
    pub lower_left_corner: Point3,
    pub horizontal: Vec3,
    pub vertical: Vec3,

    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub lens_radius: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Camera {
        let theta = f64::to_radians(vfov);
        let h = f64::tan(theta / 2.0);

        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (lookfrom - lookat).unit();
        let u = vup.cross(&w).unit();
        let v = w.cross(&u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;

        return Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin - (horizontal / 2.0) - (vertical / 2.0) - (focus_dist * w),
            u,
            v,
            w,
            lens_radius: aperture / 2.0,
        };
    }
    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        Ray {
            orig: self.origin + offset,
            dir: self.lower_left_corner + (u * self.horizontal) + (v * self.vertical) - self.origin - offset,
        }
    }
}
