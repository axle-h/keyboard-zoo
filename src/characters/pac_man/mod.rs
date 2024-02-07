use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use box2d_rs::b2_math::B2vec2;
use box2d_rs::b2_shape::ShapeDefPtr;
use box2d_rs::shapes::b2_circle_shape::B2circleShape;
use rand::distributions::{Distribution, Standard};
use rand::prelude::SliceRandom;
use rand::Rng;
use rand::rngs::ThreadRng;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use crate::characters::animation::SpriteAnimationType;
use crate::characters::{CharacterType, CharacterWorldState};
use crate::characters::lifetime::{CharacterLifetime, CharacterLifetimeFactory, CharacterState};
use crate::characters::sound::CharacterSoundData;
use crate::characters::sprites::{CharacterSprites, SpriteSheetFormat};

const SPRITE: &[u8] = include_bytes!("sprite.png");
const SPRITE_DEATH: &[u8] = include_bytes!("death.png");

const SOUND_CREATE: &[u8] = include_bytes!("chomp.ogg");
const SOUND_DESTROY: &[u8] = include_bytes!("death.ogg");
const SOUND_EAT: &[u8] = include_bytes!("eat_ghost.ogg");

const PAC_MAN_RADIUS_METERS: f32 = 0.2;
const PAC_MAN_LIFETIME: f32 = 20.0;
const PAC_MAN_VELOCITY: f32 = 1.0;
const TURN_PROBABILITY: f32 = 0.01;
const FEEDING_TURN_PROBABILITY: f32 = 0.05;
const PAC_MAN_FRAMES: usize = 3;
const PAC_MAN_DEATH_FRAMES: usize = 12;

pub fn pac_man_sprites(texture_creator: &TextureCreator<WindowContext>) -> Result<CharacterSprites, String> {
    CharacterSprites::new(
        texture_creator,
        SpriteSheetFormat::exclusive_linear(SPRITE, PAC_MAN_FRAMES),
        SpriteSheetFormat::exclusive_linear(SPRITE_DEATH, PAC_MAN_DEATH_FRAMES),
    )
}

pub fn pac_man_lifetime() -> CharacterLifetimeFactory {
    CharacterLifetimeFactory::new(
        Duration::from_secs_f32(PAC_MAN_LIFETIME),
        SpriteAnimationType::YoYo {
            duration: Duration::from_millis(100),
            frames: PAC_MAN_FRAMES
        },
        SpriteAnimationType::LinearWithPause {
            duration: Duration::from_millis(150),
            pause_for: Duration::from_millis(200),
            resume_from_frame: 0,
            frames: PAC_MAN_DEATH_FRAMES
        },
    )
}
pub fn pac_man_sound() -> CharacterSoundData {
    CharacterSoundData::new(CharacterType::PacMan, SOUND_CREATE, SOUND_DESTROY, SOUND_EAT)
}

pub fn pac_man_shape(polygon_scale: f32) -> ShapeDefPtr{
    let mut shape = B2circleShape::default();
    shape.base.m_radius = PAC_MAN_RADIUS_METERS * polygon_scale;
    Rc::new(RefCell::new(shape))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacManDirection {
    Up, Down, Left, Right
}

impl PacManDirection {
    pub fn angle(&self) -> f32 {
        match self {
            PacManDirection::Up => 270.0,
            PacManDirection::Down => 90.0,
            PacManDirection::Left => 180.0,
            PacManDirection::Right => 0.0
        }
    }

    pub fn velocity(&self, polygon_scale: f32) -> B2vec2 {
        let velocity = polygon_scale * PAC_MAN_VELOCITY;
        match self {
            PacManDirection::Up => B2vec2::new(0.0, -velocity),
            PacManDirection::Down => B2vec2::new(0.0, velocity),
            PacManDirection::Left => B2vec2::new(-velocity, 0.0),
            PacManDirection::Right => B2vec2::new(velocity, 0.0),
        }
    }

    pub fn other_directions(&self) -> Vec<PacManDirection> {
        [Self::Up, Self::Down, Self::Left, Self::Right].into_iter()
            .filter(|d| d != self)
            .collect::<Vec<PacManDirection>>()
    }
}

impl Distribution<PacManDirection> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PacManDirection {
        match rng.gen_range(0..4) {
            0 => PacManDirection::Up,
            1 => PacManDirection::Down,
            2 => PacManDirection::Left,
            _ => PacManDirection::Right,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PacManState {
    direction: PacManDirection,
    lifetime: CharacterLifetime,
    since_last_turn: f32
}

impl PacManState {
    pub fn new(rng: &mut ThreadRng, lifetime: CharacterLifetime) -> Self {
        Self { direction: rng.gen(), lifetime, since_last_turn: 0.0 }
    }

    pub fn velocity(&self, polygon_scale: f32) -> B2vec2 {
        match self.lifetime.state() {
            CharacterState::Alive => self.direction.velocity(polygon_scale),
            CharacterState::Dead | CharacterState::Death => B2vec2::zero(),
        }
    }

    pub fn angle(&self) -> f32 {
        match self.lifetime.state() {
            CharacterState::Alive => self.direction.angle(),
            CharacterState::Dead | CharacterState::Death => 0.0
        }
    }

    pub fn lifetime(&self) -> &CharacterLifetime {
        &self.lifetime
    }

    pub fn lifetime_mut(&mut self) -> &mut CharacterLifetime {
        &mut self.lifetime
    }

    pub fn update(&mut self, rng: &mut ThreadRng, delta: Duration, world_state: CharacterWorldState) -> CharacterState {
        let state = self.lifetime.update(delta);
        if state.is_alive() {
            self.since_last_turn += delta.as_secs_f32();
            if world_state.is_colliding {
                self.direction = Self::preferred_direction(world_state).or_else(||
                    self.direction.other_directions().choose(rng).copied()
                ).unwrap();
                self.since_last_turn = 0.0;
            } else if let Some(preferred_direction) = Self::preferred_direction(world_state) {
                // behaviour when feeding, prefer turning to direction of closest body with slight debounce
                if self.direction != preferred_direction && self.should_turn(rng, FEEDING_TURN_PROBABILITY) {
                    self.direction = preferred_direction;
                    self.since_last_turn = 0.0;
                }
            // behaviour when nothing left to eat: slow, random turns
            } else if self.should_turn(rng, TURN_PROBABILITY) {
                self.direction = rng.gen();
                self.since_last_turn = 0.0;
            }
        }
        state
    }

    fn should_turn(&self, rng: &mut ThreadRng, probability: f32) -> bool {
        (self.since_last_turn * probability) > rng.gen::<f32>()
    }

    fn preferred_direction(world_state: CharacterWorldState) -> Option<PacManDirection> {
        world_state.closest_body.map(|d| {
            if d.x.abs() > d.y.abs() {
                // farthest away in the x direction
                if d.x > 0.0 {
                    PacManDirection::Left
                } else {
                    PacManDirection::Right
                }
            } else if d.y > 0.0 {
                PacManDirection::Up
            } else {
                PacManDirection::Down
            }
        })
    }
}
