use nalgebra_glm::Vec3;
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};

pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        Cube { center, size, material }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let half_size = self.size / 2.0;
        
        let min = self.center - Vec3::new(half_size, half_size, half_size);
        let max = self.center + Vec3::new(half_size, half_size, half_size);

        let inv_dir = Vec3::new(1.0 / ray_direction.x, 1.0 / ray_direction.y, 1.0 / ray_direction.z);
        let t_min = (min - ray_origin).component_mul(&inv_dir);
        let t_max = (max - ray_origin).component_mul(&inv_dir);

        let t1 = t_min.x.min(t_max.x).max(t_min.y.min(t_max.y)).max(t_min.z.min(t_max.z));
        let t2 = t_min.x.max(t_max.x).min(t_min.y.max(t_max.y)).min(t_min.z.max(t_max.z));

        if t1 > t2 || t2 < 0.0 {
            return Intersect::empty();
        }

        let t_hit = if t1 < 0.0 { t2 } else { t1 };

        let hit_point = ray_origin + ray_direction * t_hit;

        let mut normal = Vec3::new(0.0, 0.0, 0.0);
        for i in 0..3 {
            if (hit_point[i] - min[i]).abs() < 0.001 {
                normal[i] = -1.0;
            } else if (hit_point[i] - max[i]).abs() < 0.001 {
                normal[i] = 1.0;
            }
        }

        Intersect::new(
            hit_point,
            normal,
            t_hit,
            self.material,
        )
    }
}