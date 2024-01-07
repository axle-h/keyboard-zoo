use crate::particles::color::ParticleColor;
use crate::particles::geometry::Vec2D;
use crate::particles::meta::ParticleSprite;

/// A particle wave modelled as a sin function magnitude * sin(frequency * lifetime)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleWave {
    magnitude: f64,
    frequency: f64,
}

impl ParticleWave {
    pub fn new(magnitude: f64, frequency: f64) -> Self {
        Self {
            magnitude,
            frequency,
        }
    }

    pub fn next(&self, lifetime: f64) -> f64 {
        self.magnitude * (self.frequency * lifetime).sin()
    }
    pub fn magnitude(&self) -> f64 {
        self.magnitude
    }
    pub fn frequency(&self) -> f64 {
        self.frequency
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    position: Vec2D,
    velocity: Vec2D,
    acceleration: Vec2D,
    max_alpha: f64,
    alpha: f64,
    pulse: Option<ParticleWave>,
    color: ParticleColor,
    time_to_live: Option<f64>,
    sprite: ParticleSprite,
    size: f64,
    rotation: f64,
    angular_velocity: f64
}

impl Particle {
    pub fn new(
        position: Vec2D,
        velocity: Vec2D,
        acceleration: Vec2D,
        max_alpha: f64,
        alpha: f64,
        pulse: Option<ParticleWave>,
        color: ParticleColor,
        time_to_live: Option<f64>,
        sprite: ParticleSprite,
        size: f64,
        angular_velocity: f64,
    ) -> Self {
        Self {
            position,
            velocity,
            acceleration,
            max_alpha,
            alpha,
            pulse,
            color,
            time_to_live,
            sprite,
            size,
            rotation: 0.0,
            angular_velocity,
        }
    }

    /// checks if the particle is out of bounds (0-1) and trajectory will not bring it back
    pub fn is_escaped(&self) -> bool {
        const THRESHOLD_MAX: f64 = 1.05;
        const THRESHOLD_MIN: f64 = -0.05;
        (self.position.x() > THRESHOLD_MAX
            && self.velocity.x() >= 0.0
            && self.acceleration.x() >= 0.0)
            || (self.position.x() < THRESHOLD_MIN
                && self.velocity.x() <= 0.0
                && self.acceleration.x() <= 0.0)
            || (self.position.y() > THRESHOLD_MAX
                && self.velocity.y() >= 0.0
                && self.acceleration.y() >= 0.0)
            || (self.position.y() < THRESHOLD_MIN
                && self.velocity.y() <= 0.0
                && self.acceleration.y() <= 0.0)
    }

    pub fn update(&mut self, delta_time: f64, lifetime: f64) {
        self.velocity += self.acceleration * delta_time;
        self.position += self.velocity * delta_time;
        self.rotation += self.angular_velocity * delta_time;
    }

    pub fn position(&self) -> Vec2D {
        self.position
    }
    pub fn alpha(&self) -> f64 {
        self.alpha
    }
    pub fn color(&self) -> ParticleColor {
        self.color
    }
    pub fn sprite(&self) -> ParticleSprite {
        self.sprite
    }
    pub fn size(&self) -> f64 {
        self.size
    }
    pub fn rotation(&self) -> f64 {
        self.rotation
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroup {
    lifetime: f64,
    anchor_for: Option<f64>,
    fade_in: Option<f64>,
    fade_out: bool,
    orbit: Option<Vec2D>,
    particles: Vec<Particle>,
}

impl ParticleGroup {
    pub fn new(
        anchor_for: Option<f64>,
        fade_in: Option<f64>,
        fade_out: bool,
        orbit: Option<Vec2D>,
        particles: Vec<Particle>,
    ) -> Self {
        Self {
            lifetime: 0.0,
            anchor_for,
            fade_in,
            fade_out,
            orbit,
            particles,
        }
    }

    pub fn update_life(&mut self, delta_time: f64) {
        self.lifetime += delta_time;

        // remove dead particles
        let mut to_remove = vec![];
        for (index, particle) in self.particles.iter().enumerate() {
            if particle.is_escaped() {
                to_remove.push(index);
            } else if let Some(time_to_live) = particle.time_to_live {
                if self.lifetime >= time_to_live {
                    to_remove.push(index);
                }
            }
        }
        for index in to_remove.into_iter().rev() {
            self.particles.remove(index);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    pub fn len(&self) -> usize {
        self.particles.len()
    }

    pub fn update_particles(&mut self, delta_time: f64) {
        // spatial
        if let Some(anchor_for) = self.anchor_for {
            self.anchor_for = if delta_time >= anchor_for {
                None
            } else {
                Some(anchor_for - delta_time)
            }
        } else {
            // orbit
            if let Some(orbit) = self.orbit {
                for particle in self.particles.iter_mut() {
                    let delta = particle.position - orbit;
                    let magnitude_squared = delta.magnitude_squared();
                    // only apply gravitation when particle is sufficiently distant as this approximation breaks down for small distances
                    if magnitude_squared > 0.001 {
                        // Vector form of Newtons law of gravitation with empirically ideal G * m
                        let f = delta.unit_vector() * (-0.001 / magnitude_squared);
                        particle.velocity += f * delta_time;
                    }
                }
            }

            for particle in self.particles.iter_mut() {
                particle.update(delta_time, self.lifetime);
            }
        }

        // alpha
        if let Some(fade_in) = self.fade_in {
            if self.lifetime >= fade_in {
                self.fade_in = None;
            } else {
                for particle in self.particles.iter_mut() {
                    particle.alpha = particle.max_alpha * self.lifetime.min(fade_in) / fade_in;
                }
            }
        }
        // fade out
        else if self.fade_out {
            for particle in self.particles.iter_mut() {
                if let Some(ttl) = particle.time_to_live {
                    particle.alpha = particle.max_alpha * (1.0 - self.lifetime.min(ttl) / ttl);
                }
            }
        }

        for particle in self.particles.iter_mut() {
            // pulse
            if let Some(pulse) = particle.pulse.as_ref() {
                let pulse_magnitude = pulse.next(self.lifetime);
                particle.alpha = (particle.alpha + pulse_magnitude)
                    .min(particle.max_alpha)
                    .max(0.0);
            }
        }
    }

    pub fn particles(&self) -> &[Particle] {
        self.particles.as_slice()
    }
}
