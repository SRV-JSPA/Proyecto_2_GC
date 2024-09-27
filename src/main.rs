
mod framebuffer;
mod ray_intersect;
mod sphere;
mod color;
mod camera;
mod light;
mod material;
mod cube;


use minifb::{ Window, WindowOptions, Key };
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;

use crate::color::Color;
use crate::cube::Cube;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::sphere::Sphere;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Box<dyn RayIntersect>], light: &Light) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting && tmp.distance < zbuffer {
            zbuffer = intersect.distance;
            intersect = tmp;
        }
    }

    if !intersect.is_intersecting {
        return Color::new(4, 12, 36);
    }

    if intersect.material.albedo == [0.0, 0.0] {
        return intersect.material.diffuse; 
    }
    
    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);


    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light.intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light.intensity;

    diffuse + specular
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "DIORAMA",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    let rubber = Material::new(
        Color::new(80, 0, 0),
        1.0,
        [0.9, 0.1],
    );

    let ivory = Material::new(
        Color::new(100, 100, 80),
        50.0,
        [0.6, 0.3],
    );

    let sol_material = Material::new(
        Color::new(255, 234, 100),
        1.0,
        [0.0, 0.0],
    );

    let mut luz = Light::new(
        Vec3::new(100.0, 100.0, 10.0),
        Color::new(255, 255, 255),
        5.0,
        3.0 
    );    

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(Cube {
            center: Vec3::new(-1.5, 0.0, -6.0),
            size: 1.0,
            material: rubber,
        }),
        Box::new(Sphere {
            center: luz.position,  
            radius: luz.radius,    
            material: sol_material,
        }),
    ];

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let rotation_speed = PI / 10.0;
    let velocidad_movimiento = 0.1;

    while window.is_open() && !window.is_key_down(Key::Escape) {

        if window.is_key_down(Key::W) {
            camera.mover_enfrente(velocidad_movimiento);
        }
        if window.is_key_down(Key::S) {
            camera.mover_atras(velocidad_movimiento);
        }
        if window.is_key_down(Key::A) {
            camera.mover_izq(velocidad_movimiento);
        }
        if window.is_key_down(Key::D) {
            camera.mover_der(velocidad_movimiento);
        }

        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        render(&mut framebuffer, &objects, &camera, &luz);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

