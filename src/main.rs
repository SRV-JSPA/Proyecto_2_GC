mod framebuffer;
mod ray_intersect;
mod sphere;
mod color;
mod camera;
mod light;
mod material;
mod cube;
use nalgebra_glm::{Vec3, normalize};
use std::time::{Duration, Instant};
use std::f32::consts::PI;
use minifb::{Window, WindowOptions, Key};
use crate::color::Color;
use crate::cube::Cube;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::sphere::Sphere;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;

fn reflector(incidente: &Vec3, normal: &Vec3) -> Vec3 {
    incidente - 2.0 * incidente.dot(normal) * normal
}

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Box<dyn RayIntersect>], luz: &Light, color_fondo: &Color) -> Color {
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
        return color_fondo.clone();
    }

    if intersect.material.albedo == [0.0, 0.0] {
        return intersect.material.diffuse;
    }

    let luz_dir = (luz.position - intersect.point).normalize();
    let vista_dir = (ray_origin - intersect.point).normalize();
    let reflector_dir = reflector(&-luz_dir, &intersect.normal);

    let intensidad_difuminado = intersect.normal.dot(&luz_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * intensidad_difuminado * luz.intensity;

    let specular_intensidad = vista_dir.dot(&reflector_dir).max(0.0).powf(intersect.material.specular);
    let specular = luz.color * intersect.material.albedo[1] * specular_intensidad * luz.intensity;

    diffuse + specular
}

fn transicion_color(inicio: &Color, fin: &Color, t: f32) -> Color {
    let r = (inicio.r() as f32 * (1.0 - t) + fin.r() as f32 * t) as u8;
    let g = (inicio.g() as f32 * (1.0 - t) + fin.g() as f32 * t) as u8;
    let b = (inicio.b() as f32 * (1.0 - t) + fin.b() as f32 * t) as u8;

    Color::new(r, g, b)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, light: &Light, color_fondo: &Color) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, color_fondo);

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
    let intervalo_cambio_color = Duration::from_secs(65); 
    let duracion_recorrido_luz = Duration::from_secs(10); 

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
        3.0,
    );

    let mut esfera_amarilla = Sphere {
        center: luz.position, 
        radius: 1.0, 
        material: sol_material.clone(),
    };

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let rotation_speed = PI / 10.0;
    let velocidad_movimiento = 0.1;

    let color_inicial = Color::new(4, 12, 36);
    let color_final = Color::new(135, 206, 235);
    let mut color_actual = color_inicial.clone();
    let mut siguiente_color = color_final.clone();
    let mut tiempo_inicial = Instant::now();
    let mut progreso_transicion = 0.0;

    let mut tiempo_luz = Instant::now();

    let color_blanco = Color::new(255, 255, 255);
    let color_amarillo = Color::new(255, 234, 100);
    let mut color_esfera_actual = color_blanco.clone();

    let radio = 100.0;
    let duracion_recorrido_luz_secs = duracion_recorrido_luz.as_secs_f32();
    let velocidad_angular = PI / duracion_recorrido_luz_secs;
    let mut angulo = 0.0;

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

        if Instant::now().duration_since(tiempo_luz) >= frame_delay {
            angulo += velocidad_angular * frame_delay.as_secs_f32();

            if angulo >= 2.0 * PI {
                angulo -= 2.0 * PI;
                let progreso_color = 1.0;
                color_esfera_actual = transicion_color(&color_blanco, &color_amarillo, progreso_color);
                siguiente_color = if color_esfera_actual == color_blanco { color_amarillo.clone() } else { color_blanco.clone() };
            } else {
                let tiempo_transcurrido = Instant::now().duration_since(tiempo_inicial);
                let progreso_color = (tiempo_transcurrido.as_secs_f32() / intervalo_cambio_color.as_secs_f32()) % 1.0;
                color_esfera_actual = transicion_color(&color_blanco, &color_amarillo, progreso_color);
            }

            luz.position.x = radio * angulo.cos();
            luz.position.y = 100.0 + radio * angulo.sin();
            luz.position.z = 10.0;

            esfera_amarilla.center = luz.position;
            esfera_amarilla.material.diffuse = color_esfera_actual;

            tiempo_luz = Instant::now();
        }

        if Instant::now().duration_since(tiempo_inicial) >= frame_delay {
            progreso_transicion = (Instant::now().duration_since(tiempo_inicial).as_secs_f32() % intervalo_cambio_color.as_secs_f32()) / intervalo_cambio_color.as_secs_f32();
            color_actual = transicion_color(&color_inicial, &color_final, progreso_transicion);
        }

        let objects: Vec<Box<dyn RayIntersect>> = vec![ 
            Box::new(Cube {
                center: Vec3::new(-1.5, 0.0, -6.0),
                size: 1.0,
                material: rubber,
            }),
            Box::new(esfera_amarilla.clone()),
        ];

        render(&mut framebuffer, &objects, &camera, &luz, &color_actual);

        window.update_with_buffer(&framebuffer.buffer, framebuffer.width, framebuffer.height).unwrap();
    }
}