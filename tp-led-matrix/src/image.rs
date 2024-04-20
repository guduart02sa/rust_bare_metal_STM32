#[repr(C)]
#[derive(Copy, Clone, Default)]

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

use crate::gamma;

impl Color {
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    pub fn gamma_correct(&self) -> Color {
        Color {
            r: gamma::gamma_correct(self.r),
            g: gamma::gamma_correct(self.g),
            b: gamma::gamma_correct(self.b),
        }
    }
}

use micromath::F32Ext;
impl core::ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, arg: f32) -> Color {
        Color {
            r: (self.r as f32 * arg).clamp(0.0, 255.0).round() as u8,
            g: (self.g as f32 * arg).clamp(0.0, 255.0).round() as u8,
            b: (self.b as f32 * arg).clamp(0.0, 255.0).round() as u8,
        }
    }
}

use core::ops::Mul;
impl core::ops::Div<f32> for Color {
    type Output = Color;

    fn div(self, arg: f32) -> Color {
        self.mul(1.0 / arg)
    }
}

/* ❎ Create a public structure image::Image containing a unique unnamed field consisting of an array of 64 Color. Structure with unnamed fields are declared as follow, and fields are access like tuple fields (.0 to access the first field, .1 to access the second field, …): */
#[repr(transparent)]
pub struct Image([Color; 64]);

impl Image {
    pub const fn new_solid(color: Color) -> Self {
        Image([color; 64])
    }
    pub fn row(&self, row: usize) -> &[Color] {
        &self.0[row * 8..(row + 1) * 8]
    }
    pub fn gradient(color: Color) -> Self {
        let mut new_image = Image::default();
        for row in 0..8 {
            for col in 0..8 {
                new_image[(row, col)] = color / (1.0 + (row * row + col) as f32);
            }
        }
        new_image
    }
}

impl Default for Image {
    fn default() -> Self {
        Image::new_solid(Color::default())
    }
}

impl core::ops::Index<(usize, usize)> for Image {
    type Output = Color;

    fn index(&self, (row, column): (usize, usize)) -> &Color {
        &self.0[row * 8 + column]
    }
}

impl core::ops::IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, (row, column): (usize, usize)) -> &mut Color {
        &mut self.0[row * 8 + column]
    }
}

impl AsRef<[u8; 192]> for Image {
    fn as_ref(&self) -> &[u8; 192] {
        unsafe { core::mem::transmute(self) }
    }
}

impl AsMut<[u8; 192]> for Image {
    fn as_mut(&mut self) -> &mut [u8; 192] {
        unsafe { core::mem::transmute(self) }
    }
}
