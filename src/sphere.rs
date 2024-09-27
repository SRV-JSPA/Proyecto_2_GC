use nalgebra_glm::{Vec3, dot};
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;
use std::any::Any;

#[derive(Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let oc = ray_origin - self.center;

        let a = dot(ray_direction, ray_direction);
        let b = 2.0 * dot(&oc, ray_direction);
        let c = dot(&oc, &oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant > 0.0 {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 {
                let point = ray_origin + ray_direction * t;
                let normal = (point - self.center).normalize();
                let distance = t;

                return Intersect::new(point, normal, distance, self.material.clone(), 0.0, 0.0); 
            }
        }

        Intersect::empty()
    }

    fn get_uv(&self, point: &Vec3) -> (f32, f32) {

        let r = (point - self.center).normalize();


        let theta = r.z.atan2(r.x);


        let phi = r.y.asin();

        let u = 0.5 + theta / (2.0 * std::f32::consts::PI);
        let v = 0.5 - phi / std::f32::consts::PI;

        (u, v)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}