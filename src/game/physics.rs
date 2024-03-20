use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashSet;
use std::f64::consts::PI;
use std::ops::Sub;
use std::rc::Rc;
use std::time::Duration;
use box2d_rs::b2_body::{B2body, B2bodyDef, B2bodyType, BodyPtr};
use box2d_rs::b2_collision::{B2AABB, B2worldManifold};
use box2d_rs::b2_contact::B2contactDynTrait;
use box2d_rs::b2_fixture::{B2fixture, B2fixtureDef, FixturePtr};
use box2d_rs::b2_math::{b2_mul_transform_by_vec2, B2Transform, B2vec2};
use box2d_rs::b2_world::{B2world, B2worldPtr};
use box2d_rs::b2_world_callbacks::{B2contactImpulse, B2contactListener};
use box2d_rs::b2rs_common::UserDataType;
use box2d_rs::shapes::b2_edge_shape::B2edgeShape;
use box2d_rs::shapes::b2_polygon_shape::B2polygonShape;
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use crate::characters::{Character, CharacterFactory, CharacterType, CharacterWorldState};
use crate::assets::geometry::SpriteAsset;
use crate::characters::lifetime::CharacterState;
use crate::config::PhysicsConfig;
use crate::game::action::{Direction, PhysicsAction};
use crate::game::event::{GameEvent};
use crate::game::physics_debug::SdlPhysicsDraw;
use crate::game::polygon::Triangle;
use crate::game::scale::PhysicsScale;

#[derive(Debug, Clone)]
pub struct AlphanumericBody {
    pub name: String,
    pub alphanumeric: char,
}

#[derive(Default, Debug, Clone)]
pub enum BodyType {
    #[default]
    Unknown,
    Alphanumeric(AlphanumericBody),
    Character(Character)
}

#[derive(Debug, Clone)]
pub struct Body {
    pub id: u128,
    pub aabb: Rect,
    pub angle: f64,
    pub polygons: Vec<Triangle>,
    pub body_type: BodyType
}


#[derive(Default, Debug, Clone)]
struct BodyData {
    id: u128,
    width: f32,
    height: f32,
    body_type: BodyType
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
enum ContactMagnitude { Light, Heavy }

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct ContactedBody {
    body_id: u128,
    character: Option<CharacterType>
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum ContactTarget {
    Ground,
    Alphanumeric { id: u128 },
    Character { id: u128, character_type: CharacterType }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct Contact {
    body: ContactedBody,
    magnitude: ContactMagnitude,
    target: ContactTarget
}

impl Contact {
    // pub fn new<B : Into<Option<ContactedBody>>>(body: ContactedBody, target: B, magnitude: ContactMagnitude) -> Self {
    //     let target = match target.into() {
    //         Some(target_body) if target_body.character.is_some() => ContactTarget::Character(target_body.character.unwrap()),
    //         Some(target_body) => ContactTarget::Alphanumeric(target_body),
    //         None => ContactTarget::Ground
    //     };
    //
    //     Self { body, target, magnitude }
    // }

    pub fn with_ground(body: ContactedBody, magnitude: ContactMagnitude) -> Self {
        Self { body, magnitude, target: ContactTarget::Ground }
    }

    pub fn between_characters(body: ContactedBody, target: ContactedBody, magnitude: ContactMagnitude) -> Self {
        Self { body, magnitude, target: ContactTarget::Character { id: target.body_id, character_type: target.character.unwrap() } }
    }

    pub fn between_bodies(maybe_character_body: ContactedBody, target: ContactedBody, magnitude: ContactMagnitude) -> Self {
        Self { body: maybe_character_body, magnitude, target: ContactTarget::Alphanumeric { id: target.body_id } }
    }
}

struct ContactListener {
    heavy_collision_threshold: f32,
    contacts: HashSet<Contact>
}

impl ContactListener {
    fn new(heavy_collision_threshold: f32) -> Self {
        Self { heavy_collision_threshold, contacts: HashSet::new() }
    }

    fn contact_meta(body: Ref<B2body<UserDataTypes>>) -> Option<ContactedBody> {
        if let Some(data) = body.get_user_data() {
            Some(ContactedBody {
                body_id: data.id,
                character: if let BodyType::Character(character) = data.body_type {
                    Some(character.character_type())
                } else {
                    None
                },
            })
        } else {
            None
        }
    }
}

impl B2contactListener<UserDataTypes> for ContactListener {

    fn post_solve(&mut self, contact: &mut dyn B2contactDynTrait<UserDataTypes>, _: &B2contactImpulse) {
        let base = contact.get_base();

        let mut world_manifold: B2worldManifold = Default::default();
        base.get_world_manifold(&mut world_manifold);
        let fixture_ptr_a = base.get_fixture_a();
        let fixture_a = fixture_ptr_a.borrow();
        let body_ptr_a = fixture_a.get_body();
        let body_a = body_ptr_a.borrow();

        let fixture_ptr_b = base.get_fixture_b();
        let fixture_b = fixture_ptr_b.borrow();
        let body_ptr_b = fixture_b.get_body();
        let body_b = body_ptr_b.borrow();

        let v_a = body_a.get_linear_velocity_from_world_point(world_manifold.points[0]);
        let v_b = body_b.get_linear_velocity_from_world_point(world_manifold.points[0]);
        let contact_velocity = (v_a - v_b).length();

        let contact_magnitude =
            if contact_velocity > self.heavy_collision_threshold { ContactMagnitude::Heavy } else { ContactMagnitude::Light };

        let meta_a = Self::contact_meta(body_a);
        let meta_b = Self::contact_meta(body_b);

        // if let Some(meta_a) = meta_a {
        //     self.contacts.insert(Contact::new(meta_a, meta_b, contact_magnitude));
        // }
        //
        // if let Some(meta_b) = meta_b {
        //     self.contacts.insert(Contact::new(meta_b, meta_a, contact_magnitude));
        // }

        if meta_a.is_none() && meta_b.is_none() {
            return;
        }

        if meta_a.is_none() || meta_b.is_none() {
            // collision with the ground
            let meta = [meta_a, meta_b].into_iter().find(|m| m.is_some()).unwrap().unwrap();
            self.contacts.insert(Contact::with_ground(meta, contact_magnitude));
            return;
        }

        let meta_a = meta_a.unwrap();
        let meta_b = meta_b.unwrap();
        if meta_a.character.is_some() && meta_b.character.is_some() {
            // collision between two characters
            self.contacts.insert(Contact::between_characters(meta_a, meta_b, contact_magnitude));
        } else if meta_b.character.is_some() {
            // prefer the character body to be the collision subject
            self.contacts.insert(Contact::between_bodies(meta_b, meta_a, contact_magnitude));
        } else {
            self.contacts.insert(Contact::between_bodies(meta_a, meta_b, contact_magnitude));
        }
    }
}

impl Physics {
    pub fn new(scale: PhysicsScale, config: PhysicsConfig) -> Self {
        let gravity = B2vec2::new(0.0, config.gravity);
        let mut world: B2worldPtr<UserDataTypes> = B2world::new(gravity);

        let contact_listener = Rc::new(RefCell::new(ContactListener::new(config.heavy_collision_threshold)));
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
        let mut character_reported_collisions = HashSet::new();
        let mut events = vec![];
        let mut is_heavy_collision = false;
        for contact in self.contact_listener.borrow_mut().contacts.drain() {
            if let Some(character_subject) = contact.body.character {
                // collision where a character is the subject
                match contact.target {
                    ContactTarget::Ground => {
                        // report collision to the character
                        character_reported_collisions.insert(contact.body.body_id);
                    }
                    ContactTarget::Alphanumeric { id } => {
                        if character_subject.destroys_on_collision() {
                            to_destroy.insert(id);
                            events.push(GameEvent::CharacterAttack(character_subject));
                        }
                    }
                    ContactTarget::Character { id, .. } => {
                        // report collision to both characters
                        character_reported_collisions.insert(id);
                        character_reported_collisions.insert(contact.body.body_id);
                    }
                }
            } else {
                // collision between non-character bodies
                if contact.magnitude == ContactMagnitude::Heavy && contact.body.character.is_none() {
                    // a heavy collision between two alphanumerics or a alphanumeric and the ground
                    is_heavy_collision = true;
                }
            }
        }

        if is_heavy_collision {
            events.push(GameEvent::HeavyCollision);
        }

        let mut alphanumeric_positions = vec![];
        for body_ptr in self.world.borrow().get_body_list().iter() {
            let body = body_ptr.borrow_mut();
            if let Some(BodyType::Alphanumeric(_)) = body.get_user_data().map(|d| d.body_type) {
                alphanumeric_positions.push(body.get_position());
            }
        }

        let mut to_transform = vec![];
        for body_ptr in self.world.borrow().get_body_list().iter() {
            let mut body = body_ptr.borrow_mut();
            if let Some(data) = body.get_user_data().as_mut() {
                if let BodyType::Character(character) = &mut data.body_type {
                    let is_colliding = character_reported_collisions.contains(&data.id);
                    let position = body.get_position();
                    let distance_to_closest_body = alphanumeric_positions.iter()
                        .map(|&v| {
                            let distance = position - v;
                            (distance, distance.length())
                        })
                        .filter(|(v, d)| *d > 0.0001 || *d < 0.0001) // exclude self
                        .min_by(|(v1, d1), (v2, d2)| (*d1).total_cmp(d2))
                        .map(|(v1, _)| v1);

                    let character_world_state = CharacterWorldState::new(is_colliding, distance_to_closest_body);
                    let character_physics = self.character_factory.update(character, delta, character_world_state);
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
            body_type: BodyType::Character(character)
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

        let (sprite_width, sprite_height) = sprite.unit_scale();
        let width = sprite_width as f32 * polygon_scale;
        let height = sprite_height as f32 * polygon_scale;

        let (position, to_destroy) = self.rng_spawn_position(width, height);
        let mut events = self.destroy_bodies(to_destroy);

        let body_data = BodyData {
            id: self.rng.gen(),
            width,
            height,
            body_type: BodyType::Alphanumeric(
                AlphanumericBody { name: sprite.name().to_string(), alphanumeric: sprite.character() }
            )
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

        let offset_x = sprite_width / 2.0;
        let offset_y = sprite_height / 2.0;
        for triangle in sprite.triangles().into_iter() {
            let points = triangle.points()
                .map(|p| B2vec2::new(
                    polygon_scale * (p.x - offset_x) as f32,
                    polygon_scale * (p.y - offset_y) as f32)
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
                aabb,
                angle,
                polygons,
                body_type: data.body_type
            })
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
                        if let BodyType::Character(character) = &mut body_data.body_type {
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
}
