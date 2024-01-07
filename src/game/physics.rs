use std::cell::RefCell;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::rc::Rc;
use std::time::Duration;
use box2d_rs::b2_body::{B2body, B2bodyDef, B2bodyType, BodyPtr};
use box2d_rs::b2_collision::B2AABB;
use box2d_rs::b2_fixture::{B2fixtureDef, FixturePtr};
use box2d_rs::b2_math::{b2_mul_transform_by_vec2, B2vec2};
use box2d_rs::b2_world::{B2world, B2worldPtr};
use box2d_rs::b2rs_common::UserDataType;
use box2d_rs::shapes::b2_edge_shape::B2edgeShape;
use box2d_rs::shapes::b2_polygon_shape::B2polygonShape;
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use crate::assets::geometry::{SpriteRect, SpriteAsset};
use crate::config::PhysicsConfig;
use crate::game::action::{Direction, PhysicsAction};
use crate::game::event::PhysicsEvent;
use crate::game::physics_debug::SdlPhysicsDraw;
use crate::game::polygon::Triangle;
use crate::game::scale::WorldScale;

#[derive(Debug, Clone)]
pub struct Body {
    id: u128,
    sprite_name: String,
    aabb: Rect,
    angle: f64,
    polygons: Vec<Triangle>
}

impl Body {
    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn aabb(&self) -> Rect {
        self.aabb
    }

    pub fn sprite_name(&self) -> &str {
        &self.sprite_name
    }

    pub fn angle(&self) -> f64 {
        self.angle
    }

    pub fn polygons(&self) -> &Vec<Triangle> {
        &self.polygons
    }
}

#[derive(Default, Debug, Clone)]
struct BodyData {
    id: u128,
    width: f32,
    height: f32,
    sprite_name: String
}

#[derive(Debug, Clone)]
struct FixtureData {
    color: Color
}

impl Default for FixtureData {
    fn default() -> Self {
        Self { color: Color::BLACK }
    }
}

#[derive(Default, Debug, Clone)]
struct UserDataTypes;
impl UserDataType for UserDataTypes {
    type Fixture = FixtureData;
    type Body = BodyData;
    type Joint = ();
}

pub struct Physics {
    rng: ThreadRng,
    world: B2worldPtr<UserDataTypes>,
    scale: WorldScale,
    config: PhysicsConfig
}

impl Physics {
    pub fn new(scale: WorldScale, config: PhysicsConfig) -> Self {
        let gravity = B2vec2::new(0.0, config.gravity);
        let world: B2worldPtr<UserDataTypes> = B2world::new(gravity);

        let (world_width, world_height) = scale.b2d_size();

        let ground = B2world::create_body(world.clone(), &B2bodyDef::default());

        let shape = Rc::new(RefCell::new(B2edgeShape::default()));
        let fixture_def = B2fixtureDef {
            shape: Some(shape.clone()),
            density: 0.0,
            friction: config.body_friction,
            ..B2fixtureDef::default()
        };

        let bottom_left = B2vec2::new(0.0, 0.0);
        let bottom_right = B2vec2::new(world_width, 0.0);
        let top_left = B2vec2::new(0.0, world_height);
        let top_right = B2vec2::new(world_width, world_height);

        // bottom
        shape.borrow_mut().set_two_sided(bottom_left, bottom_right);
        B2body::create_fixture(ground.clone(), &fixture_def);

        // left
        shape.borrow_mut().set_two_sided(bottom_left, top_left);
        B2body::create_fixture(ground.clone(), &fixture_def);

        // top
        shape.borrow_mut().set_two_sided(top_left, top_right);
        B2body::create_fixture(ground.clone(), &fixture_def);

        // right
        shape.borrow_mut().set_two_sided(bottom_right, top_right);
        B2body::create_fixture(ground.clone(), &fixture_def);

        Self { world, scale, config, rng: thread_rng() }
    }

    pub fn set_sdl_debug_draw(&mut self, canvas: Rc<RefCell<WindowCanvas>>) {
        let debug_draw = Rc::new(RefCell::new(SdlPhysicsDraw::new(canvas, self.scale.clone())));
        self.world.borrow_mut().set_debug_draw(debug_draw)
    }

    pub fn debug_draw(&self) {
        self.world.borrow().debug_draw();
    }

    pub fn update(&mut self, delta: Duration) -> Vec<PhysicsEvent> {
        self.world.borrow_mut().step(delta.as_secs_f32(), self.config.velocity_iterations, self.config.position_iterations);
        vec![]
    }

    pub fn action(&mut self, action: PhysicsAction) {
        let world = self.world.borrow_mut();
        match action {
            PhysicsAction::Push(direction) => {
                let magnitude = self.config.push_force_magnitude;
                let force = match direction {
                    Direction::Up => B2vec2::new(0.0, -magnitude),
                    Direction::Down => B2vec2::new(0.0, magnitude),
                    Direction::Left => B2vec2::new(-magnitude, 0.0),
                    Direction::Right => B2vec2::new(magnitude, 0.0),
                };

                for body in world.get_body_list().iter() {
                    let mut body = body.borrow_mut();
                    if let Some(data) = body.get_user_data() {
                        body.apply_force_to_center(force, true);
                    }
                }
            }
        }
    }

    fn rng_spawn_position_attempt(&mut self, width: f32, height: f32) -> (f32, f32, Vec<BodyPtr<UserDataTypes>>) {
        let world = self.world.borrow();
        const SPAWN_BUFFER: f32 = 0.1;
        let hw = width / 2.0 + SPAWN_BUFFER;
        let hh = height / 2.0 + SPAWN_BUFFER;
        let (world_width, world_height) = self.scale.b2d_size();
        let x = self.rng.gen_range(hw .. (world_width - hw));
        let y = self.rng.gen_range(hh .. (world_height - hh));

        let aabb = B2AABB {
            lower_bound: B2vec2::new(x - hw, y - hh),
            upper_bound: B2vec2::new(x + hw, y + hh)
        };
        let mut to_destroy_set = HashSet::new();
        world.query_aabb(|f: FixturePtr<UserDataTypes>| {
            if let Some(data) = f.borrow().get_body().borrow().get_user_data() {
                to_destroy_set.insert(data.id);
            }
            true
        }, aabb);

        let mut to_destroy = vec![];
        for body in world.get_body_list().iter() {
            if matches!(body.borrow().get_user_data(), Some(data) if to_destroy_set.contains(&data.id)) {
                to_destroy.push(body)
            }
        }
        (x, y, to_destroy)
    }

    fn rng_spawn_position(&mut self, width: f32, height: f32) -> (f32, f32, Vec<BodyPtr<UserDataTypes>>) {
        let mut attempt = 0;
        loop {
            let (x, y, to_destroy) = self.rng_spawn_position_attempt(width, height);
            if to_destroy.is_empty() || attempt >= 10 {
                return (x, y, to_destroy)
            }
            attempt += 1;
        }
    }

    pub fn spawn_body(&mut self, sprite: SpriteAsset) -> Vec<PhysicsEvent> {
        let polygon_scale = self.config.polygon_scale;

        let sprite_aabb = sprite.aabb();
        let width = sprite_aabb.width() as f32 * polygon_scale;
        let height = sprite_aabb.height() as f32 * polygon_scale;

        let (x, y, to_destroy) = self.rng_spawn_position(width, height);
        let destroyed_bodies = self.destroy_bodies(to_destroy);

        let body_data = BodyData {
            id: self.rng.gen(),
            width,
            height,
            sprite_name: sprite.name().to_string()
        };
        let body_def = B2bodyDef {
            position: B2vec2::new(x, y),
            body_type: B2bodyType::B2DynamicBody,
            user_data: Some(body_data),
            ..B2bodyDef::default()
        };
        let body = B2world::create_body(self.world.clone(), &body_def);

        let shape = Rc::new(RefCell::new(B2polygonShape::default()));
        let mut fixture_def = B2fixtureDef {
            shape: Some(shape.clone()),
            density: self.config.body_density,
            friction: self.config.body_friction,
            restitution: self.config.body_restitution,
            ..B2fixtureDef::default()
        };

        let offset_x = sprite_aabb.width() / 2.0;
        let offset_y = sprite_aabb.height() / 2.0;
        for triangle in sprite.triangles().into_iter() {
            let points = triangle.points()
                .map(|p| B2vec2::new(
                    polygon_scale * (p.x() - offset_x) as f32,
                    polygon_scale * (p.y() - offset_y) as f32)
                );
            shape.borrow_mut().set(&points);
            let [r, g, b] = triangle.color();
            fixture_def.user_data = Some(FixtureData { color: Color::RGB(r, g, b) });
            B2body::create_fixture(body.clone(), &fixture_def);
        }

        let body = self.body(body).unwrap();
        destroyed_bodies.into_iter()
            .map(|b| PhysicsEvent::Destroy(b))
            .chain([PhysicsEvent::Spawned(body)])
            .collect()
    }

    pub fn destroy_body(&mut self, id: u128) -> Option<PhysicsEvent> {
        let mut world = self.world.borrow_mut();
        for body in world.get_body_list().iter() {
            if matches!(body.borrow().get_user_data(), Some(data) if id == data.id) {
                let event = PhysicsEvent::Destroy(self.body(body.clone()).unwrap());
                world.destroy_body(body);
                return Some(event)
            }
        }
        None
    }

    pub fn bodies(&self) -> Vec<Body> {
        self.world.borrow().get_body_list().iter()
            .map(|b| self.body(b))
            .flatten()
            .collect()
    }

    fn body(&self, body: Rc<RefCell<B2body<UserDataTypes>>>) -> Option<Body> {
        let body = body.borrow();
        if let Some(data) = body.get_user_data() {
            let position = body.get_position();
            let snip = Rect::from_center(
                self.scale.b2d_vec2_to_sdl(position),
                self.scale.b2d_to_sdl(data.width) as u32,
                self.scale.b2d_to_sdl(data.height) as u32
            );
            let transform = body.get_transform();
            let mut polygons = vec![];
            for fixture in body.get_fixture_list().iter() {
                let fixture = fixture.borrow();
                if let Some(polygon) = fixture.get_shape().as_polygon() {
                    let points: [Point; 3] = polygon.m_vertices[..polygon.m_count].iter()
                        .map(|v| self.scale.b2d_vec2_to_sdl(b2_mul_transform_by_vec2(transform, *v)))
                        .collect::<Vec<Point>>()
                        .try_into()
                        .unwrap();
                    let color = fixture.get_user_data().unwrap().color;
                    polygons.push(Triangle::new(points, color));
                }
            }
            Some(Body {
                id: data.id,
                aabb: snip,
                sprite_name: data.sprite_name,
                angle: body.get_angle() as f64 * 180.0 / PI,
                polygons
            })
        } else {
            None
        }
    }

    fn destroy_bodies(&mut self, bodies: Vec<BodyPtr<UserDataTypes>>) -> Vec<Body> {
        let mut result = vec![];
        let mut world = self.world.borrow_mut();
        for body in bodies.into_iter() {
            result.push(self.body(body.clone()).unwrap());
            world.destroy_body(body);
        }
        result
    }

    fn to_b2_aabb(&self, rect: SpriteRect) -> B2AABB {
        B2AABB {
            lower_bound: self.scale.point_to_b2d_vec2(rect.lower_bound()),
            upper_bound: self.scale.point_to_b2d_vec2(rect.upper_bound()),
        }
    }
}
