use std::cell::{RefCell, RefMut};
use std::collections::hash_set::IntoIter;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;
use box2d_rs::b2_body::{B2body, B2bodyDef, B2bodyType, BodyPtr};
use box2d_rs::b2_collision::B2AABB;
use box2d_rs::b2_contact::B2contactDynTrait;
use box2d_rs::b2_fixture::{B2fixtureDef, FixturePtr};
use box2d_rs::b2_math::{b2_mul_transform_by_vec2, B2Transform, B2vec2};
use box2d_rs::b2_world::{B2world, B2worldPtr};
use box2d_rs::b2_world_callbacks::B2contactListener;
use box2d_rs::b2rs_common::UserDataType;
use box2d_rs::shapes::b2_edge_shape::B2edgeShape;
use box2d_rs::shapes::b2_polygon_shape::B2polygonShape;
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use crate::characters::{Character, CharacterFactory, CharacterType};
use crate::assets::geometry::{SpriteRect, SpriteAsset};
use crate::characters::lifetime::CharacterState;
use crate::config::PhysicsConfig;
use crate::game::action::{Direction, PhysicsAction};
use crate::game::event::{GameEvent};
use crate::game::physics_debug::SdlPhysicsDraw;
use crate::game::polygon::Triangle;
use crate::game::scale::PhysicsScale;

#[derive(Debug, Clone)]
pub enum Body {
    Asset(AssetBody),
    Character(CharacterBody)
}

impl Body {
    pub fn id(&self) -> u128 {
        match self {
            Body::Asset(body) => body.id,
            Body::Character(body) => body.id
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetBody {
    id: u128,
    aabb: Rect,
    angle: f64,
    asset_name: String,
    polygons: Vec<Triangle>,
}

impl AssetBody {
    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn aabb(&self) -> Rect {
        self.aabb
    }

    pub fn asset_name(&self) -> &str {
        &self.asset_name
    }

    pub fn angle(&self) -> f64 {
        self.angle
    }

    pub fn polygons(&self) -> &Vec<Triangle> {
        &self.polygons
    }
}

#[derive(Debug, Clone)]
pub struct CharacterBody {
    id: u128,
    aabb: Rect,
    angle: f64,
    character: Character
}

impl CharacterBody {
    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn aabb(&self) -> Rect {
        self.aabb
    }

    pub fn angle(&self) -> f64 {
        self.angle
    }

    pub fn character(&self) -> &Character {
        &self.character
    }
}

#[derive(Default, Debug, Clone)]
struct BodyData {
    id: u128,
    width: f32,
    height: f32,
    asset_name: Option<String>,
    character: Option<Character>
}

#[derive(Debug, Clone)]
struct FixtureData {
    id: u128,
    color: Color
}

impl Default for FixtureData {
    fn default() -> Self {
        Self { id: 0, color: Color::BLACK }
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
    scale: PhysicsScale,
    config: PhysicsConfig,
    character_factory: CharacterFactory,
    contact_listener: Rc<RefCell<ContactListener>>,
    time_since_last_explosion: Duration
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct ContactedBody {
    body_id: u128,
    fixture_id: Option<u128>,
    character: Option<CharacterType>
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum Contact {
    WithGround { body: ContactedBody },
    WithAsset { body: ContactedBody, target: ContactedBody },
    WithCharacter { body: ContactedBody, target: CharacterType }
}

impl Contact {
    pub fn new<B : Into<Option<ContactedBody>>>(body: ContactedBody, target: B) -> Self {
        match target.into() {
            Some(target) if target.character.is_some() => Self::WithCharacter { body, target: target.character.unwrap() },
            Some(target) => Self::WithAsset { body, target },
            None => Self::WithGround { body }
        }
    }

    fn body(&self) -> &ContactedBody {
        match self {
            Contact::WithGround { body, .. } => body,
            Contact::WithAsset { body, .. } => body,
            Contact::WithCharacter { body, .. } => body
        }
    }
}

struct ContactListener {
    contacts: HashSet<Contact>
}

impl ContactListener {
    fn new() -> Self {
        Self { contacts: HashSet::new() }
    }

    fn contact_meta(fixture_ptr: FixturePtr<UserDataTypes>) -> Option<ContactedBody> {
        let fixture = fixture_ptr.borrow();
        if let Some(data) = fixture.get_body().borrow().get_user_data() {
            Some(ContactedBody {
                body_id: data.id,
                fixture_id: fixture.get_user_data().map(|f| f.id),
                character: data.character.map(|c| c.character_type()),
            })
        } else {
            None
        }
    }
}

impl B2contactListener<UserDataTypes> for ContactListener {
    fn begin_contact(&mut self, contact: &mut dyn B2contactDynTrait<UserDataTypes>) {
        let base = contact.get_base();
        let body_a = Self::contact_meta(base.get_fixture_a());
        let body_b = Self::contact_meta(base.get_fixture_b());

        if let Some(body_a) = body_a {
            self.contacts.insert(Contact::new(body_a, body_b));
        }

        if let Some(body_b) = body_b {
            self.contacts.insert(Contact::new(body_b, body_a));
        }
    }
}

impl Physics {
    pub fn new(scale: PhysicsScale, config: PhysicsConfig) -> Self {
        let gravity = B2vec2::new(0.0, config.gravity);
        let mut world: B2worldPtr<UserDataTypes> = B2world::new(gravity);

        let contact_listener = Rc::new(RefCell::new(ContactListener::new()));
        world.borrow_mut().set_contact_listener(contact_listener.clone());
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

        Self {
            world,
            scale,
            config,
            rng: thread_rng(),
            character_factory: CharacterFactory::new(config),
            contact_listener,
            time_since_last_explosion: Duration::ZERO
        }
    }

    pub fn set_sdl_debug_draw(&mut self, canvas: Rc<RefCell<WindowCanvas>>) {
        let debug_draw = Rc::new(RefCell::new(SdlPhysicsDraw::new(canvas, self.scale.clone())));
        self.world.borrow_mut().set_debug_draw(debug_draw)
    }

    pub fn debug_draw(&self) {
        self.world.borrow().debug_draw();
    }

    pub fn update(&mut self, delta: Duration) -> Vec<GameEvent> {
        self.time_since_last_explosion += delta;
        self.world.borrow_mut().step(delta.as_secs_f32(), self.config.velocity_iterations, self.config.position_iterations);

        let mut to_destroy = HashSet::new();
        let mut character_ground_collisions = HashSet::new();
        let mut events = vec![];
        for contact in self.contact_listener.borrow_mut().contacts.drain() {

            match contact {
                Contact::WithGround { body } if body.character.is_some() => {
                    // report collision to the character
                    character_ground_collisions.insert(body.body_id);
                }
                Contact::WithCharacter { body, target } if target.destroys_on_collision() => {
                    to_destroy.insert(body.body_id);
                    events.push(GameEvent::CharacterAttack(target));
                }
                _ => {}
            }
        }

        let mut to_transform = vec![];
        for body_ptr in self.world.borrow().get_body_list().iter() {
            let mut body = body_ptr.borrow_mut();
            if let Some(data) = body.get_user_data().as_mut() {
                if let Some(character) = data.character.as_mut() {
                    let is_ground_collision = character_ground_collisions.contains(&data.id);
                    let character_physics = self.character_factory.update(character, delta, is_ground_collision);
                    if !character.state().is_alive() {
                        to_destroy.insert(data.id);
                    }

                    body.set_linear_velocity(character_physics.velocity());
                    to_transform.push((body_ptr.clone(), body.get_position(), character_physics.angle() * PI as f32 / 180.0));
                    body.set_user_data(data); // update the character
                }
            }
        }

        for (body, position, angle) in to_transform {
            body.borrow_mut().set_transform(position, angle);
        }


        self.destroy_bodies_by_id(to_destroy).into_iter().chain(events).collect()
    }

    pub fn action(&mut self, action: PhysicsAction) -> Vec<GameEvent> {
        let mut result = vec![];
        match action {
            PhysicsAction::Push(direction) => {
                let magnitude = self.config.push_force_magnitude;
                let force = match direction {
                    Direction::Up => B2vec2::new(0.0, -magnitude),
                    Direction::Down => B2vec2::new(0.0, magnitude),
                    Direction::Left => B2vec2::new(-magnitude, 0.0),
                    Direction::Right => B2vec2::new(magnitude, 0.0),
                };

                let world = self.world.borrow_mut();
                for body in world.get_body_list().iter() {
                    let mut body = body.borrow_mut();
                    if body.get_type() == B2bodyType::B2DynamicBody {
                        body.apply_force_to_center(force, true);
                    }
                }
            }
            PhysicsAction::Explode => {
                // the simulation falls apart if too many explosions are spammed
                if self.time_since_last_explosion < Duration::from_millis(100) {
                    return result;
                }
                self.time_since_last_explosion = Duration::ZERO;

                let location = self.rng_world_coordinates(0.1, 0.1).get_center();

                let world = self.world.borrow_mut();
                for body in world.get_body_list().iter() {
                    let mut body = body.borrow_mut();

                    if body.get_type() != B2bodyType::B2DynamicBody {
                        continue;
                    }

                    let mut vector = body.get_position() - location;
                    let distance = vector.normalize();
                    // force is inversely proportional to the distance between body and explosion center
                    let force = self.config.explosion_force_magnitude - (distance / self.config.explosion_distance) * self.config.explosion_force_magnitude;
                    if force > 0.0 {
                        vector *= force;
                        body.apply_force_to_center(vector, true);
                    }
                }

                let point = self.scale.b2d_vec2_to_sdl(location);
                result.push(GameEvent::Explosion { x: point.x(), y: point.y() });
            }
        }
        result
    }

    fn rng_world_coordinates(&mut self, width: f32, height: f32) -> B2AABB {
        const SPAWN_BUFFER: f32 = 0.1;
        let hw = width / 2.0 + SPAWN_BUFFER;
        let hh = height / 2.0 + SPAWN_BUFFER;
        let (world_width, world_height) = self.scale.b2d_size();
        let x = self.rng.gen_range(hw .. (world_width - hw));
        let y = self.rng.gen_range(hh .. (world_height - hh));
        B2AABB {
            lower_bound: B2vec2::new(x - hw, y - hh),
            upper_bound: B2vec2::new(x + hw, y + hh)
        }
    }

    fn rng_spawn_position_attempt(&mut self, width: f32, height: f32) -> (B2AABB, Vec<BodyPtr<UserDataTypes>>) {
        let aabb = self.rng_world_coordinates(width, height);
        let world = self.world.borrow();

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

        let center = aabb.get_center();
        (aabb, to_destroy)
    }

    fn rng_spawn_position(&mut self, width: f32, height: f32) -> (B2vec2, Vec<BodyPtr<UserDataTypes>>) {
        let mut attempt = 0;
        loop {
            let (aabb, to_destroy) = self.rng_spawn_position_attempt(width, height);
            if to_destroy.is_empty() || attempt >= 10 {
                return (aabb.get_center(), to_destroy)
            }
            attempt += 1;
        }
    }

    pub fn spawn_character(&mut self, character_type: CharacterType) -> Vec<GameEvent> {
        let shape = self.character_factory.shape(character_type);
        let mut aabb = B2AABB::default();
        shape.borrow().compute_aabb(&mut aabb, B2Transform::default(), 0);

        let width = aabb.upper_bound.x - aabb.lower_bound.x;
        let height = aabb.upper_bound.y - aabb.lower_bound.y;

        let (position, to_destroy) = self.rng_spawn_position(width, height);
        let mut events = self.destroy_bodies(to_destroy);

        let character = self.character_factory.new_character(character_type);
        let body_data = BodyData {
            id: self.rng.gen(),
            width,
            height,
            asset_name: None,
            character: Some(character)
        };
        let body_def = B2bodyDef {
            position,
            body_type: B2bodyType::B2DynamicBody,
            user_data: Some(body_data),
            ..B2bodyDef::default()
        };
        let body = B2world::create_body(self.world.clone(), &body_def);

        let fixture_def = B2fixtureDef {
            shape: Some(shape),
            density: self.config.body_density,
            friction: self.config.body_friction,
            restitution: self.config.body_restitution,
            ..B2fixtureDef::default()
        };
        B2body::create_fixture(body.clone(), &fixture_def);

        let body = self.body(body).unwrap();
        events.push(GameEvent::Spawned(body));
        events
    }

    pub fn spawn_asset(&mut self, sprite: SpriteAsset) -> Vec<GameEvent> {
        let polygon_scale = self.config.polygon_scale;

        let sprite_aabb = sprite.aabb();
        let width = sprite_aabb.width() as f32 * polygon_scale;
        let height = sprite_aabb.height() as f32 * polygon_scale;

        let (position, to_destroy) = self.rng_spawn_position(width, height);
        let mut events = self.destroy_bodies(to_destroy);

        let body_data = BodyData {
            id: self.rng.gen(),
            width,
            height,
            asset_name: Some(sprite.name().to_string()),
            character: None
        };
        let body_def = B2bodyDef {
            position,
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
            fixture_def.user_data = Some(FixtureData { id: self.rng.gen(), color: Color::RGB(r, g, b) });
            B2body::create_fixture(body.clone(), &fixture_def);
        }

        let body = self.body(body).unwrap();
        events.push(GameEvent::Spawned(body));
        events
    }

    pub fn destroy_body(&mut self, id: u128) -> Option<GameEvent> {
        self.destroy_bodies_by_id(HashSet::from([id])).first().cloned()
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
            let aabb = Rect::from_center(
                self.scale.b2d_vec2_to_sdl(position),
                self.scale.b2d_to_sdl(data.width) as u32,
                self.scale.b2d_to_sdl(data.height) as u32
            );
            let angle = body.get_angle() as f64 * 180.0 / PI;

            if let Some(character) = data.character {
                let character_body = CharacterBody {
                    id: data.id,
                    aabb,
                    angle,
                    character,
                };
                Some(Body::Character(character_body))
            } else if let Some(asset_name) = data.asset_name {
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
                let asset_body = AssetBody {
                    id: data.id,
                    aabb,
                    asset_name,
                    angle,
                    polygons,
                };
                Some(Body::Asset(asset_body))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn destroy_bodies_by_id(&mut self, mut ids: HashSet<u128>) -> Vec<GameEvent> {
        if ids.is_empty() {
            return vec![]
        }

        let mut to_destroy = vec![];
        for body in self.world.borrow().get_body_list().iter() {
            if let Some(data) = body.borrow().get_user_data() {
                if ids.contains(&data.id) {
                    to_destroy.push(body.clone());
                    ids.remove(&data.id);
                    if ids.is_empty() {
                        break;
                    }
                }
            }
        }

        self.destroy_bodies(to_destroy)
    }

    fn destroy_bodies(&mut self, bodies: Vec<BodyPtr<UserDataTypes>>) -> Vec<GameEvent> {
        let mut result = vec![];
        let mut to_disable = vec![];

        {
            // we have to borrow the world in a clojure as B2body::set_enabled borrows the world again
            let mut world = self.world.borrow_mut();
            for body_ptr in bodies.into_iter() {
                enum Behaviour { Destroy { notify: bool }, Disable }
                impl Behaviour {
                    fn destroy_if_enabled(body: RefMut<B2body<UserDataTypes>>) -> Option<Behaviour> {
                        if body.is_enabled() {
                            Some(Behaviour::Disable)
                        } else {
                            None
                        }
                    }
                }

                let mut behaviour: Option<Behaviour> = Some(Behaviour::Destroy { notify: true });
                {
                    let mut body = body_ptr.borrow_mut();
                    if let Some(body_data) = body.get_user_data().as_mut() {
                        if let Some(character) = body_data.character.as_mut() {
                            behaviour = match character.state() {
                                CharacterState::Alive => {
                                    // "kill" this character
                                    character.destroy();
                                    body.set_user_data(body_data);
                                    Behaviour::destroy_if_enabled(body)
                                }
                                CharacterState::Death => {
                                    // character already going to die, just disable it
                                    Behaviour::destroy_if_enabled(body)
                                }
                                CharacterState::Dead => {
                                    // character is dead, remove the body but do not notify
                                    // as we should already have done when it was disabled
                                    Some(Behaviour::Destroy { notify: false })
                                }
                            };
                        }
                    }
                }

                if let Some(behaviour) = behaviour {
                    let event = GameEvent::Destroy(self.body(body_ptr.clone()).unwrap());
                    match behaviour {
                        Behaviour::Destroy { notify } => {
                            if notify {
                                result.push(event);
                            }
                            world.destroy_body(body_ptr)
                        },
                        Behaviour::Disable => {
                            result.push(event);
                            to_disable.push(body_ptr)
                        }
                    }
                }
            }
        }

        for body_ptr in to_disable.into_iter() {
            B2body::set_enabled(body_ptr, false);
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
