use std::time::Duration;
use box2d_rs::b2_math::B2vec2;
use box2d_rs::b2_shape::{ShapeDefPtr};
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};
use crate::characters::lifetime::{CharacterLifetimeFactory, CharacterState};
use crate::characters::pac_man::{pac_man_lifetime, pac_man_shape, pac_man_sound, PacManDirection, PacManState};
use crate::characters::sound::CharacterSound;
use crate::config::{AudioConfig, PhysicsConfig};

mod pac_man;
mod sprites;
pub mod render;
mod animation;
pub mod lifetime;
mod sound;

pub struct CharacterPhysics {
    angle: f32,
    velocity: B2vec2
}

impl CharacterPhysics {
    pub fn new(angle: f32, velocity: B2vec2) -> Self {
        Self { angle, velocity }
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }

    pub fn velocity(&self) -> B2vec2 {
        self.velocity
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum CharacterType {
    PacMan
}

impl CharacterType {
    pub fn destroys_on_collision(&self) -> bool {
        match self {
            CharacterType::PacMan => true
        }
    }
}

impl Distribution<CharacterType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CharacterType {
        CharacterType::PacMan
    }
}

#[derive(Debug, Clone)]
pub enum Character {
    PacMan(PacManState)
}

impl Character {
    pub fn character_type(&self) -> CharacterType {
        match self {
            Character::PacMan(_) => CharacterType::PacMan
        }
    }

    pub fn destroy(&mut self) {
        match self {
            Character::PacMan(pac_man) => {
                pac_man.lifetime_mut().destroy()
            }
        }
    }

    pub fn state(&self) -> CharacterState {
        match self {
            Character::PacMan(pac_man) => {
                pac_man.lifetime().state()
            }
        }
    }
}

pub struct CharacterFactory {
    pac_man: CharacterLifetimeFactory,
    rng: ThreadRng,
    config: PhysicsConfig
}

impl CharacterFactory {
    pub fn new(config: PhysicsConfig) -> Self {
        Self { pac_man: pac_man_lifetime(), rng: thread_rng(), config }
    }

    pub fn new_character(&mut self, character: CharacterType) -> Character {
        match character {
            CharacterType::PacMan => {
                let lifetime = pac_man_lifetime().new_lifetime(&mut self.rng);
                Character::PacMan(PacManState::new(&mut self.rng, lifetime))
            }
        }
    }

    pub fn shape(&mut self, character: CharacterType) -> ShapeDefPtr {
        match character {
            CharacterType::PacMan => pac_man_shape(self.config.polygon_scale)
        }
    }

    pub fn update(&mut self, character: &mut Character, delta: Duration, is_ground_collision: bool) -> CharacterPhysics {
        match character {
            Character::PacMan(pac_man) => {
                pac_man.update(&mut self.rng, delta, is_ground_collision);
                CharacterPhysics::new(pac_man.angle(), pac_man.velocity(self.config.polygon_scale))
            }
        }
    }
}

pub fn sound(config: AudioConfig) -> Result<CharacterSound, String> {
    CharacterSound::new(config, &[pac_man_sound()])
}