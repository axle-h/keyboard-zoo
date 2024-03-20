use crate::particles::geometry::{RectF, Vec2D};
use crate::particles::source::ParticlePositionSource;
use sdl2::rect::{Point, Rect};
use std::cmp::max;
use crate::assets::geometry::SpriteTriangle;
use crate::game::polygon::{PolygonArea, Triangle};

const LATTICE_SCALE: usize = 5;
const PERIMETER_SCALE: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    window_width: f64,
    window_height: f64,
}

impl Scale {
    pub fn new(window_size: (u32, u32)) -> Self {
        let (window_width, window_height) = window_size;
        Self {
            window_width: window_width as f64,
            window_height: window_height as f64,
        }
    }

    pub fn point_to_particle_space<P: Into<Point>>(&self, point: P) -> Vec2D {
        let point = point.into();
        Vec2D::new(
            point.x() as f64 / self.window_width,
            point.y() as f64 / self.window_height,
        )
    }

    pub fn point_to_render_space<P: Into<Vec2D>>(&self, point: P) -> Point {
        let point = point.into();
        Point::new(
            (point.x() * self.window_width).round() as i32,
            (point.y() * self.window_height).round() as i32,
        )
    }

    pub fn rect_to_particle_space<R: Into<Rect>>(&self, rect: R) -> RectF {
        let rect = rect.into();
        RectF::new(
            rect.x() as f64 / self.window_width,
            rect.y() as f64 / self.window_height,
            rect.width() as f64 / self.window_width,
            rect.height() as f64 / self.window_height,
        )
    }

    pub fn static_source<P: Into<Point>>(&self, point: P) -> ParticlePositionSource {
        ParticlePositionSource::Static(self.point_to_particle_space(point.into()))
    }

    pub fn rect_source<R: Into<Rect>>(&self, rect: R) -> ParticlePositionSource {
        ParticlePositionSource::Rect(self.rect_to_particle_space(rect.into()))
    }

    pub fn random_rect_source<R: Into<Rect>>(&self, rect: R) -> ParticlePositionSource {
        ParticlePositionSource::RandomCascade(self.rect_to_particle_space(rect.into()))
    }

    pub fn rect_lattice_source(&self, rects: &[Rect]) -> ParticlePositionSource {
        let points = rects.iter().flat_map(|r| self.lattice_points(*r)).collect();
        ParticlePositionSource::Lattice(points)
    }

    pub fn polygon_lattice_source<A : PolygonArea + Copy>(&self, polygons: &[A]) -> ParticlePositionSource {
        let points = polygons.iter().flat_map(|&t| self.area_lattice_points(t)).collect();
        ParticlePositionSource::Lattice(points)
    }


    pub fn perimeter_lattice_sources(&self, rect: Rect) -> [ParticlePositionSource; 4] {
        let rows = max(rect.height() as usize / PERIMETER_SCALE, 3);
        let cols = max(rect.width() as usize / PERIMETER_SCALE, 3);

        let cell_width = (rect.width() as f64 / (cols - 1) as f64).round() as i32;
        let cell_height = (rect.height() as f64 / (rows - 1) as f64).round() as i32;

        let top = (0..cols).map(|i| Point::new(rect.x() + i as i32 * cell_width, rect.top()));
        let bottom = (0..cols).map(|i| Point::new(rect.x() + i as i32 * cell_width, rect.bottom()));
        let right = (0..rows).map(|j| Point::new(rect.right(), rect.y() + j as i32 * cell_height));
        let left = (0..rows).map(|j| Point::new(rect.left(), rect.y() + j as i32 * cell_height));

        [
            self.build_lattice(top),
            self.build_lattice(right),
            self.build_lattice(bottom),
            self.build_lattice(left),
        ]
    }

    pub fn build_lattice<I: Iterator<Item = Point>>(&self, iter: I) -> ParticlePositionSource {
        ParticlePositionSource::Lattice(iter.map(|p| self.point_to_particle_space(p)).collect())
    }

    pub fn build_ephemeral_lattice<I: Iterator<Item = Point>>(
        &self,
        iter: I,
    ) -> ParticlePositionSource {
        ParticlePositionSource::EphemeralLattice(
            iter.map(|p| self.point_to_particle_space(p)).collect(),
        )
    }

    fn lattice_points(&self, rect: Rect) -> Vec<Vec2D> {
        let rows = max(rect.height() as usize / LATTICE_SCALE, 3);
        let cols = max(rect.width() as usize / LATTICE_SCALE, 3);

        assert!(rows > 0);
        assert!(cols > 0);
        let cell_width = (rect.width() as f64 / (cols - 1) as f64).round() as i32;
        let cell_height = (rect.height() as f64 / (rows - 1) as f64).round() as i32;

        (0..rows as i32)
            .flat_map(|j| {
                (0..cols as i32)
                    .map(move |i| Point::new(rect.x() + i * cell_width, rect.y() + j * cell_height))
            })
            .map(|p| self.point_to_particle_space(p))
            .collect()
    }

    fn area_lattice_points<A : PolygonArea>(&self, triangle: A) -> Vec<Vec2D> {
        let rect = triangle.aabb();

        let rows = max(rect.height() as usize / LATTICE_SCALE, 3);
        let cols = max(rect.width() as usize / LATTICE_SCALE, 3);

        assert!(rows > 0);
        assert!(cols > 0);
        let cell_width = (rect.width() as f64 / (cols - 1) as f64).round() as i32;
        let cell_height = (rect.height() as f64 / (rows - 1) as f64).round() as i32;

        (0..rows as i32)
            .flat_map(|j| {
                (0..cols as i32)
                    .map(move |i| Point::new(rect.x() + i * cell_width, rect.y() + j * cell_height))
            })
            .filter(|&p| triangle.contains_point(p))
            .map(|p| self.point_to_particle_space(p))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_render_to_particle_space() {
        let scale = Scale::new((1920, 1080));
        let observed = scale.point_to_particle_space((480, 540));
        assert_eq!(observed, Vec2D::new(0.25, 0.5))
    }

    #[test]
    fn point_particle_to_render_space() {
        let scale = Scale::new((1920, 1080));
        let observed = scale.point_to_render_space((0.25, 0.5));
        assert_eq!(observed, Point::new(480, 540))
    }

    #[test]
    fn rect_render_to_particle_space() {
        let scale = Scale::new((1920, 1080));
        let observed = scale.rect_to_particle_space(Rect::new(480, 540, 96, 27));
        assert_eq!(observed, RectF::new(0.25, 0.5, 0.05, 0.025))
    }
}
