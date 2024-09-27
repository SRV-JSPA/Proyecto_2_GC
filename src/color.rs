use std::ops::{Add, AddAssign, Mul};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Color { r, g, b }
    }

    pub fn to_hex(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn lerp(&self, otro: &Color, t: f32) -> Color {
        let r = (self.r as f32 * (1.0 - t) + otro.r as f32 * t) as u8;
        let g = (self.g as f32 * (1.0 - t) + otro.g as f32 * t) as u8;
        let b = (self.b as f32 * (1.0 - t) + otro.b as f32 * t) as u8;

        Color::new(r, g, b)
    }

    pub fn r(&self) -> u8 { self.r }
    pub fn g(&self) -> u8 { self.g }
    pub fn b(&self) -> u8 { self.b }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, other: Color) {
        self.r = self.r.saturating_add(other.r);
        self.g = self.g.saturating_add(other.g);
        self.b = self.b.saturating_add(other.b);
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: (self.r as f32 * scalar).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * scalar).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * scalar).clamp(0.0, 255.0) as u8,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color(r: {}, g: {}, b: {})", self.r, self.g, self.b)
    }
}