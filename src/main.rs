mod framebuffer;
mod ray_intersect;
mod sphere;
mod color;
mod camera;
mod light;
mod material;
mod cube;
mod texturas;
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
use crate::texturas::TextureManager;
use image::open;

fn reflector(incidente: &Vec3, normal: &Vec3) -> Vec3 {
    incidente - 2.0 * incidente.dot(normal) * normal
}

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Box<dyn RayIntersect>], luz: &Light, color_fondo: &Color) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting && tmp.distance < zbuffer {
            zbuffer = tmp.distance;
            intersect = tmp;
        }
    }

    if !intersect.is_intersecting {
        return color_fondo.clone();  
    }

    let mut color = intersect.material.diffuse.clone();  

    if let Some(ref textura) = intersect.material.textura {
        color = intersect.material.get_diffuse_color(intersect.u, intersect.v);
    }

    if let Some(emissive_color) = intersect.material.emisividad_color {
        color += emissive_color;
    }

    let light_dir = (luz.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflector(&-light_dir, &intersect.normal);

    let diff = intersect.normal.dot(&light_dir).max(0.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diff * luz.intensity;

    let spec = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = luz.color * intersect.material.albedo[1] * spec * luz.intensity;

    color += diffuse + specular;

    color
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
    let intervalo_cambio_color = Duration::from_secs(67); 
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
        None,
        None,
    );

    let ivory = Material::new(
        Color::new(100, 100, 80),
        50.0,
        [0.6, 0.3],
        None,
        None
    );

    let mut manejador_textura = TextureManager::new();
    let imagen_tierra = image::open("images/tierra.png").unwrap().into_rgba8();
    manejador_textura.cargar_textura("tierra", imagen_tierra);
    let textura_tierra = manejador_textura.get_textura("tierra");

    let imagen_hoja = image::open("images/hoja_arbol.jpg").unwrap().into_rgba8();
    manejador_textura.cargar_textura("hoja", imagen_hoja);
    let textura_hoja = manejador_textura.get_textura("hoja");

    let imagen_tierra_grama = image::open("images/tierra2.png").unwrap().into_rgba8();
    manejador_textura.cargar_textura("tierra_grama", imagen_tierra_grama);
    let textura_tierra_grama = manejador_textura.get_textura("tierra_grama");

    let imagen_lava = image::open("images/lava.jpg").unwrap().into_rgba8();
    manejador_textura.cargar_textura("lava", imagen_lava);
    let textura_lava = manejador_textura.get_textura("lava");

    let imagen_piedra = image::open("images/piedra.webp").unwrap().into_rgba8();
    manejador_textura.cargar_textura("piedra", imagen_piedra);
    let textura_piedra = manejador_textura.get_textura("piedra");

    let imagen_agua = image::open("images/agua.jpg").unwrap().into_rgba8();
    manejador_textura.cargar_textura("agua", imagen_agua);
    let textura_agua = manejador_textura.get_textura("agua");

    let imagen_grama = image::open("images/grama.png").unwrap().into_rgba8();
    manejador_textura.cargar_textura("grama", imagen_grama);
    let textura_grama = manejador_textura.get_textura("grama");

    let imagen_madera = image::open("images/madera.jpg").unwrap().into_rgba8();
    manejador_textura.cargar_textura("madera", imagen_madera);
    let textura_madera = manejador_textura.get_textura("madera");

    let tierra = Material::new(
        Color::new(101, 67, 33),  
        0.1, 
        [0.9, 0.05],
        textura_tierra.clone(),
        None
    );    

    let hojas = Material::new(
        Color::new(34, 139, 34),  
        0.2,
        [0.4, 0.1],  
        textura_hoja.clone(),
        None
    );

    let madera = Material::new(
        Color::new(139, 69, 19),  
        0.2,  
        [0.5, 0.1],  
        textura_madera,
        None
    );
    

    let tierra_grama = Material::new(
        Color::new(101, 67, 33),  
        0.1, 
        [0.9, 0.05],
        textura_tierra_grama.clone(),  
        None
    );

    let grama = Material::new(
        Color::new(50, 205, 50), 
        0.2,  
        [0.05, 0.1], 
        textura_grama.clone(),
        None
    );
    

    let lava = Material::new(
        Color::new(34, 139, 34),  
        1.0,
        [0.0, 0.0],  
        textura_lava.clone(),
        Some(Color::new(255, 69, 0)),
    );

    let piedra = Material::new(
        Color::new(112, 112, 112),  
        0.15,  
        [0.75, 0.05],  
        textura_piedra.clone(),
        None
    );    
    
    let agua = Material::new(
        Color::new(64, 164, 223),  
        0.9,  
        [0.1, 0.5],  
        textura_agua.clone(),
        None
    );    

    let sol_material = Material::new(
        Color::new(255, 234, 100), 
        1.0,
        [0.0, 0.0],
        None,
        None
    );

    let mut luz = Light::new(
        Vec3::new(100.0, 100.0, 10.0),
        Color::new(255, 255, 255),
        3.5,
        3.0,
    );

    let mut esfera_amarilla = Sphere {
        center: luz.position, 
        radius: 1.0, 
        material: sol_material.clone(),
    };

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 10.0, 0.0),
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
        let tiempo = tiempo_inicial.elapsed().as_secs_f32();
        let mut objects: Vec<Box<dyn RayIntersect>> = vec![ 
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -6.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -7.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -7.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -7.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -8.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -8.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -8.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -9.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -9.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -9.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -10.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -10.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -11.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -11.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -11.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 5.0, -11.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -10.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 5.0, -10.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 5.0, -9.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -9.0),
                size: 1.0,
                materials: [agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone(), agua.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -6.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -6.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -6.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -6.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -6.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -6.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 5.0, -7.0),
                size: 1.0,
                materials: [piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 5.0, -7.0),
                size: 1.0,
                materials: [piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 6.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 6.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -8.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 5.0, -8.0),
                size: 1.0,
                materials: [piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 5.0, -8.0),
                size: 1.0,
                materials: [piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 6.0, -8.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 6.0, -8.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 6.0, -8.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 5.0, -8.0),
                size: 1.0,
                materials: [piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone(), piedra.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 5.0, -9.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 5.0, -9.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 5.0, -10.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 5.0, -10.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 5.0, -11.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 5.0, -11.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(6.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(5.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(4.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -11.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(3.5, 5.0, -10.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -11.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -10.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -9.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -8.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(2.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -11.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -10.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -9.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -8.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(1.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -13.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -12.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -11.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -10.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -9.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -8.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(0.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 6.0, -7.0),
                size: 1.0,
                materials: [tierra_grama.clone(), tierra_grama.clone(), tierra.clone(), grama.clone(), tierra_grama.clone(), tierra_grama.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 5.0, -7.0),
                size: 1.0,
                materials: [tierra.clone(), tierra.clone(), tierra.clone(), tierra.clone(), tierra.clone(), tierra.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 7.0, -8.0),
                size: 1.0,
                materials: [madera.clone(), madera.clone(), madera.clone(), madera.clone(), madera.clone(), madera.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 8.0, -8.0),
                size: 1.0,
                materials: [madera.clone(), madera.clone(), madera.clone(), madera.clone(), madera.clone(), madera.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 9.0, -8.0),
                size: 1.0,
                materials: [madera.clone(), madera.clone(), madera.clone(), madera.clone(), madera.clone(), madera.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 10.0, -8.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 11.0, -8.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 12.0, -8.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 10.0, -8.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 11.0, -8.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 10.0, -8.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 11.0, -8.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 10.0, -7.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 11.0, -7.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 10.0, -7.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 11.0, -7.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 10.0, -7.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(9.5, 10.0, -9.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 10.0, -9.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(8.5, 11.0, -9.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(Cube {
                center: Vec3::new(7.5, 10.0, -9.0),
                size: 1.0,
                materials: [hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone(), hojas.clone()],
            }),
            Box::new(esfera_amarilla.clone()),
        ];

        for (i, object) in objects.iter_mut().enumerate() {
            if let Some(cube) = object.as_any_mut().downcast_mut::<Cube>() {
                if cube.materials.iter().all(|m| m == &agua) {
                    let desfase = i as f32 * 0.2; 
                    cube.center.y += (tiempo * 1.0 + desfase).sin() * 0.2;
                }
            }
        }        

        render(&mut framebuffer, &objects, &camera, &luz, &color_actual);

        window.update_with_buffer(&framebuffer.buffer, framebuffer.width, framebuffer.height).unwrap();
    }
}