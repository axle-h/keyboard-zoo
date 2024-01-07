use sdl2::pixels::Color;
use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleColor {
    red: f64,
    green: f64,
    blue: f64,
}

impl ParticleColor {
    pub const WHITE: ParticleColor = ParticleColor::rgb(1.0, 1.0, 1.0);
    pub const BLACK: ParticleColor = ParticleColor::rgb(0.0, 0.0, 0.0);
    pub const ZERO: ParticleColor = ParticleColor::rgb(0.0, 0.0, 0.0);

    pub const fn rgb(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
    }

    pub fn from_sdl(color: Color) -> Self {
        fn to_ratio(value: u8) -> f64 {
            value as f64 / 255.0
        }
        ParticleColor::rgb(to_ratio(color.r), to_ratio(color.g), to_ratio(color.b))
    }

    pub fn to_sdl(self, alpha: f64) -> Color {
        Color::RGBA(
            to_byte(self.red),
            to_byte(self.green),
            to_byte(self.blue),
            to_byte(alpha),
        )
    }
}
fn to_byte(value: f64) -> u8 {
    (255.0 * value.max(0.0).min(1.0)).round() as u8
}

impl From<(f64, f64, f64)> for ParticleColor {
    fn from((r, g, b): (f64, f64, f64)) -> Self {
        ParticleColor::rgb(r, g, b)
    }
}

impl From<ParticleColor> for (f64, f64, f64) {
    fn from(val: ParticleColor) -> Self {
        (val.red, val.green, val.blue)
    }
}

impl From<ParticleColor> for (u8, u8, u8) {
    fn from(val: ParticleColor) -> Self {
        (to_byte(val.red), to_byte(val.green), to_byte(val.blue))
    }
}

impl From<Color> for ParticleColor {
    fn from(value: Color) -> Self {
        ParticleColor::from_sdl(value)
    }
}

impl Add for ParticleColor {
    type Output = ParticleColor;

    fn add(self, rhs: Self) -> Self::Output {
        Self::rgb(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        )
    }
}

impl Mul for ParticleColor {
    type Output = ParticleColor;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::rgb(
            self.red * rhs.red,
            self.green * rhs.green,
            self.blue * rhs.blue,
        )
    }
}
