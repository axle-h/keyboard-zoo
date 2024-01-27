use box2d_rs::b2_math::B2vec2;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use crate::assets::geometry::{SpritePoint, SpriteTriangle};
use crate::config::PhysicsConfig;
use crate::game::polygon::Triangle;

#[derive(Debug, Clone)]
pub struct PhysicsScale {
    config: PhysicsConfig,
    width: u32,
    height: u32
}

impl PhysicsScale {
    pub fn new(width: u32, height: u32, config: PhysicsConfig) -> Self {
        Self { width, height, config }
    }

    pub fn b2d_to_sdl(&self, value: f32) -> i32 {
        (value * self.config.pixels_per_meter).round() as i32
    }

    pub fn point_to_b2d_vec2(&self, point: SpritePoint) -> B2vec2 {
        B2vec2::new(point.x() as f32 / self.config.pixels_per_meter, point.y() as f32 / self.config.pixels_per_meter)
    }

    pub fn b2d_vec2_to_sdl(&self, value: B2vec2) -> Point {
        Point::new(
            self.b2d_to_sdl(value.x),
            self.b2d_to_sdl(value.y)
        )
    }

    pub fn b2d_size(&self) -> (f32, f32) {
        (self.width as f32 / self.config.pixels_per_meter, self.height as f32 / self.config.pixels_per_meter)
    }

    pub fn triangle_to_polygon(&self, triangle: &SpriteTriangle) -> Triangle {
        let points = triangle.points()
            .map(|p| Point::new(
                (p.x() as f32 * self.config.polygon_scale).round() as i32,
                (p.y() as f32 * self.config.polygon_scale).round() as i32)
            );
        let [r, g, b] = triangle.color();
        Triangle::new(points, Color::RGB(r, g, b))
    }
}