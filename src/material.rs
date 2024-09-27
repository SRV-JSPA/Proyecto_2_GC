use crate::color::Color;
use image::{RgbaImage, Rgba};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 2],
    pub textura: Option<Arc<RgbaImage>>, 
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 2], textura: Option<Arc<RgbaImage>>) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            textura,
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0],
            textura: None,
        }
    }


    pub fn get_diffuse_color(&self, u: f32, v: f32) -> Color {
        if let Some(textura) = &self.textura {

            let u = u.clamp(0.0, 1.0);
            let v = v.clamp(0.0, 1.0);


            let x = (u * (textura.width() as f32 - 1.0)).floor() as u32;
            let y = (v * (textura.height() as f32 - 1.0)).floor() as u32;


            let x = x.min(textura.width() as u32 - 1);
            let y = y.min(textura.height() as u32 - 1);


            let pixel = textura.get_pixel(x, y);

            return Color::new(pixel[0], pixel[1], pixel[2]);
        } 

        self.diffuse
    }

    
    
}