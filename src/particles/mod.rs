use crate::particles::source::ParticleSource;
use particle::{Particle, ParticleGroup};

use std::time::Duration;

pub mod color;
pub mod geometry;
mod meta;
pub mod particle;
pub mod prescribed;
pub mod quantity;
pub mod render;
pub mod scale;
pub mod source;

pub struct Particles {
    particles: Vec<ParticleGroup>,
    sources: Vec<Box<dyn ParticleSource>>,
    max_particles: usize,
}

impl Particles {
    pub fn new(max_particles: usize) -> Self {
        Self {
            sources: vec![],
            particles: vec![],
            max_particles,
        }
    }

    pub fn particles(&self) -> Vec<&Particle> {
        self.particles.iter().flat_map(|g| g.particles()).collect()
    }

    pub fn update(&mut self, delta: Duration) {
        let delta_time = delta.as_secs_f64();
        self.update_life(delta_time);
        self.update_particles(delta_time);
        self.emit_particles(delta);
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.sources.clear();
    }

    fn update_life(&mut self, delta_time: f64) {
        let mut to_remove = vec![];
        for (i, group) in self.particles.iter_mut().enumerate() {
            group.update_life(delta_time);
            if group.is_empty() {
                to_remove.push(i);
            }
        }
        for i in to_remove.into_iter().rev() {
            self.particles.remove(i);
        }
    }

    fn update_particles(&mut self, delta_time: f64) {
        for group in self.particles.iter_mut() {
            group.update_particles(delta_time);
        }
    }

    fn emit_particles(&mut self, delta: Duration) {
        let current_particles = self.particles.iter().map(|g| g.len()).sum::<usize>() as i32;
        let mut max_particles = self.max_particles as i32 - current_particles;

        let mut to_remove = vec![];
        for (index, source) in self.sources.iter_mut().enumerate() {
            if max_particles <= 0 {
                return;
            }

            for group in source.update(delta, max_particles as u32) {
                max_particles -= group.len() as i32;
                self.particles.push(group);
            }

            if source.is_complete() {
                to_remove.push(index);
            }
        }
        for index in to_remove.into_iter().rev() {
            self.sources.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {}
