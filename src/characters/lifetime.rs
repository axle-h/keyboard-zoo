use std::time::Duration;
use rand::prelude::ThreadRng;
use rand::Rng;
use crate::characters::animation::{SpriteAnimation, SpriteAnimationType};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CharacterState {
    Alive,
    Death,
    Dead
}

impl CharacterState {
    pub fn is_alive(&self) -> bool {
        self == &Self::Alive
    }
}

#[derive(Debug, Clone)]
pub struct CharacterLifetime {
    max_lifetime: Duration,
    duration: Duration,
    state: CharacterState,
    animation: SpriteAnimation,
    death_animation: SpriteAnimation
}

impl CharacterLifetime {
    pub fn new(max_lifetime: Duration, animation: SpriteAnimation, death_animation: SpriteAnimation) -> Self {
        Self {
            max_lifetime,
            duration: Duration::ZERO,
            state: CharacterState::Alive,
            animation,
            death_animation,
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn update(&mut self, delta: Duration) -> CharacterState {
        self.duration += delta;
        match self.state {
            CharacterState::Alive => {
                if self.duration < self.max_lifetime {
                    self.animation.update(delta);
                } else {
                    self.state = CharacterState::Death
                }
            }
            CharacterState::Death => {
                self.death_animation.update(delta);
                // run for one iteration of the death animation
                if self.death_animation.iteration() > 0 {
                    self.state = CharacterState::Dead
                }
            }
            CharacterState::Dead => {}
        }
        self.state
    }

    pub fn animation_frame(&self) -> (CharacterState, usize) {
        let frame = match self.state {
            CharacterState::Alive => self.animation.frame(),
            CharacterState::Death | CharacterState::Dead => self.death_animation.frame(),
        };
        (self.state, frame)
    }


    pub fn state(&self) -> CharacterState {
        self.state
    }

    pub fn destroy(&mut self) {
        if self.state.is_alive() {
            self.state = CharacterState::Death;
        }
    }
}

const LIFETIME_VARIANCE: f32 = 0.25;

#[derive(Debug, Copy, Clone)]
pub struct CharacterLifetimeFactory {
    average_lifetime: Duration,
    animation: SpriteAnimationType,
    death_animation: SpriteAnimationType,
}

impl CharacterLifetimeFactory {
    pub fn new(average_lifetime: Duration, animation: SpriteAnimationType, death_animation: SpriteAnimationType) -> Self {
        Self { average_lifetime, animation, death_animation }
    }

    pub fn new_lifetime(&self, rng: &mut ThreadRng) -> CharacterLifetime {
        let lifetime_secs = self.average_lifetime.as_secs_f32();
        let range = lifetime_secs * LIFETIME_VARIANCE;
        let lifetime_offset = rng.gen::<f32>() * range - range / 2.0;
        let max_lifetime = Duration::from_secs_f32(lifetime_secs + lifetime_offset);
        CharacterLifetime::new(
            max_lifetime,
            self.animation.into_animation(),
            self.death_animation.into_animation()
        )
    }
}