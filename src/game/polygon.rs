use box2d_rs::b2_math::B2vec2;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use crate::assets::geometry::{SpritePoint, SpriteTriangle};

pub trait PolygonArea {
    fn contains_point(&self, p: Point) -> bool;
    fn aabb(&self) -> Rect;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Triangle {
    points: [Point; 3],
    color: Color
}

impl Triangle {
    pub fn new(points: [Point; 3], color: Color) -> Self {
        Self { points, color }
    }

    pub fn centroid(&self) -> Point {
        let [p1, p2, p3] = self.points;
        let centroid_x = (p1.x() as f64 + p2.x() as f64 + p3.x() as f64) / 3.0;
        let centroid_y = (p1.y() as f64 + p2.y() as f64 + p3.y() as f64) / 3.0;
        Point::new(centroid_x.round() as i32, centroid_y.round() as i32)
    }

    pub fn normalize(&self) -> Self {
        let centroid = self.centroid();
        let normals = self.points.map(|p| p - centroid);
        Self::new(normals, self.color)
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, centroid: Point, alpha: u8) -> Result<(), String> {
        let mut vx = vec![];
        let mut vy = vec![];
        for p in self.points.iter() {
            let p = *p + centroid;
            vx.push(p.x() as i16);
            vy.push(p.y() as i16);
        }
        canvas.filled_polygon(&vx, &vy, Color::RGBA(self.color.r, self.color.g, self.color.b, alpha))
    }
}

impl PolygonArea for Triangle {
    fn contains_point(&self, p: Point) -> bool {
        let x = p.x() as f64;
        let y = p.y() as f64;
        let [x0, x1, x2] = self.points.map(|p| p.x() as f64);
        let [y0, y1, y2] = self.points.map(|p| p.y() as f64);
        let a = 0.5 * (-y1 * x2 + y0 * (-x1 + x2) + x0 * (y1 - y2) + x1 * y2);
        let sign = if a < 0.0 { -1.0 } else { 1.0 };
        let s = (y0 * x2 - x0 * y2 + (y2 - y0) * x + (x0 - x2) * y) * sign;
        let t = (x0 * y1 - y0 * x1 + (y0 - y1) * x + (x1 - x0) * y) * sign;
        s > 0.0 && t > 0.0 && (s + t) < 2.0 * a * sign
    }

    fn aabb(&self) -> Rect {
        let x0 = self.points.map(|p| p.x()).into_iter().min().unwrap();
        let y0 = self.points.map(|p| p.y()).into_iter().min().unwrap();
        let x1 = self.points.map(|p| p.x()).into_iter().max().unwrap();
        let y1 = self.points.map(|p| p.y()).into_iter().max().unwrap();
        Rect::new(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32)
    }
}
impl Into<SpriteTriangle> for Triangle {
    fn into(self) -> SpriteTriangle {
        let mut triangle = SpriteTriangle::new(self.points.map(|p| SpritePoint::new(p.x() as f64, p.y() as f64)));
        triangle.set_color(self.color.r, self.color.g, self.color.b);
        triangle
    }
}

impl Default for Triangle {
    fn default() -> Self {
        let origin = Point::new(0, 0);
        Triangle::new([origin, origin, origin], Color::BLACK)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Circle {
    radius: u32,
    center: Point
}

impl Circle {
    pub fn new<P : Into<Point>>(radius: u32, center: P) -> Self {
        Self { radius, center: center.into() }
    }
}

impl PolygonArea for Circle {
    fn contains_point(&self, p: Point) -> bool {
        let mut p = B2vec2::new(p.x() as f32, p.y() as f32);
        p -= B2vec2::new(self.center.x() as f32, self.center.y() as f32);
        let d = p.length().ceil() as u32;
         d < self.radius
    }

    fn aabb(&self) -> Rect {
        Rect::from_center(self.center, self.radius, self.radius)
    }
}