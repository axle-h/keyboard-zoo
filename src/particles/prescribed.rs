use crate::particles::color::ParticleColor;
use crate::particles::geometry::Vec2D;
use crate::particles::meta::ParticleSprite;
use crate::particles::particle::ParticleWave;
use crate::particles::quantity::ProbabilityTable;
use crate::particles::scale::Scale;
use crate::particles::source::{AggregateParticleSource, ParticleModulation, ParticleProperties, ParticleSource, RandomParticleSource};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::time::Duration;
use crate::game::physics::{AssetBody, Body};
use crate::game::polygon::{Circle, Triangle};

pub fn explosion<P : Into<Point>>(center: P, scale: &Scale) -> Box<dyn ParticleSource> {
    let source = scale.polygon_lattice_source(&[Circle::new(100, center)]);
    RandomParticleSource::new(source, ParticleModulation::Cascade)
        .with_properties(ProbabilityTable::identity(ParticleProperties::new(
            ParticleSprite::all_sprite_based().as_slice(),
            (
                ParticleColor::rgb(0.7, 0.4, 0.4),
                ParticleColor::rgb(0.3, 0.3, 0.3),
            ),
            1.5,
            0.0,
        )))
        .with_velocity((Vec2D::new(0.0, 0.0), Vec2D::new(0.25, 0.25)))
        .with_fade_out((1.5, 0.5))
        .into_box()
}

pub fn fireworks(window: Rect, scale: &Scale) -> Box<dyn ParticleSource> {
    let modulation = ParticleModulation::Constant {
        count: 50,
        step: Duration::from_millis(50),
    };
    let buffer = window.height() / 5;
    let rect = Rect::from_center(
        window.center(),
        window.width() - buffer,
        window.height() - buffer,
    );
    RandomParticleSource::new(scale.random_rect_source(rect), modulation)
        .with_static_properties(
            ParticleSprite::Circle05,
            (
                ParticleColor::rgb(0.5, 0.5, 0.5),
                ParticleColor::rgb(0.5, 0.5, 0.5),
            ),
            1.0,
            0.0,
        )
        .with_velocity((Vec2D::new(0.0, -0.05), Vec2D::new(0.15, 0.15)))
        .with_fade_out((1.5, 0.5))
        .with_acceleration(Vec2D::new(0.0, 0.01)) // gravity
        .with_alpha((0.6, 0.1))
        .into_box()
}

pub fn orbit(window: Rect, scale: &Scale) -> Box<dyn ParticleSource> {
    const V: f64 = 0.05;
    let [top_left, top_right, bottom_right, bottom_left] = rect_quadrants(window);
    let sources = vec![
        orbit_source(scale, top_left, (V, -V)),
        orbit_source(scale, top_right, (V, V)),
        orbit_source(scale, bottom_right, (-V, V)),
        orbit_source(scale, bottom_left, (-V, -V)),
    ];
    AggregateParticleSource::new(sources).into_box()
}

pub fn sprite_triangle_source(triangle: Triangle, scale: &Scale) -> Box<dyn ParticleSource> {
    let position = scale.static_source(triangle.centroid());
    RandomParticleSource::new(position, ParticleModulation::SINGLE)
        .with_static_properties(ParticleSprite::SpriteTriangle(triangle.normalize()), ParticleColor::ZERO, 1.0, 0.0)
        .with_fade_out((3.0, 1.5))
        .with_velocity((Vec2D::new(0.0, -0.4), Vec2D::new(0.1, 0.1)))
        .with_acceleration(Vec2D::new(0.0, 1.5)) // gravity
        .into_box()
}

pub fn sprite_lattice_source(body: AssetBody, scale: &Scale) -> Box<dyn ParticleSource> {
    let position = scale.polygon_lattice_source(body.polygons());
    RandomParticleSource::new(position, ParticleModulation::Cascade)
        .with_static_properties(
            ParticleSprite::Circle05,
            ParticleColor::from_sdl(Color::WHITE),
            1.0,
            0.0,
        )
        .with_fade_out((1.0, 1.5))
        .with_velocity((Vec2D::ZERO, Vec2D::new(0.2, 0.2))) // gravity
        .into_box()
}

fn orbit_source<V: Into<Vec2D>>(scale: &Scale, rect: Rect, velocity: V) -> RandomParticleSource {
    let modulation = ParticleModulation::Constant {
        count: 10,
        step: Duration::from_millis(1000),
    };
    let velocity = velocity.into();
    RandomParticleSource::new(scale.rect_source(rect), modulation)
        .with_properties(
            ProbabilityTable::new()
                .with(
                    ParticleProperties::simple(&[ParticleSprite::Circle05], (1.0, 0.3)),
                    0.8,
                )
                .with(
                    ParticleProperties::new(
                        &ParticleSprite::HOLLOW_CIRCLES,
                        (
                            ParticleColor::rgb(0.6, 0.6, 0.8),
                            ParticleColor::rgb(0.1, 0.1, 0.1),
                        ),
                        (1.5, 0.4),
                        0.0,
                    ),
                    0.1,
                )
                .with(
                    ParticleProperties::new(
                        &ParticleSprite::STARS,
                        (
                            ParticleColor::rgb(0.8, 0.6, 0.6),
                            ParticleColor::rgb(0.1, 0.1, 0.1),
                        ),
                        (1.6, 0.4),
                        0.0,
                    ),
                    0.1,
                ),
        )
        .with_fade_in(Duration::from_millis(500))
        .with_fade_out((10.0, 2.5))
        .with_pulse((ParticleWave::new(0.03, 8.0), ParticleWave::new(0.01, 1.0)))
        .with_velocity((velocity, velocity * 0.5))
        .with_alpha((0.9, 0.1))
        .with_orbit((0.5, 0.5))
}

fn rect_quadrants(rect: Rect) -> [Rect; 4] {
    fn quad(point: Point, rect: Rect) -> Rect {
        Rect::new(point.x(), point.y(), rect.width() / 2, rect.height() / 2)
    }
    [
        quad(rect.top_left(), rect),                            // top left
        quad(Point::new(rect.center().x(), rect.top()), rect),  // top right
        quad(rect.center(), rect),                              // bottom right
        quad(Point::new(rect.left(), rect.center().y()), rect), // bottom left
    ]
}
