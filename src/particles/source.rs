use crate::particles::color::ParticleColor;
use crate::particles::geometry::{RectF, Vec2D};
use crate::particles::meta::ParticleSprite;
use crate::particles::particle::{Particle, ParticleGroup, ParticleWave};
use crate::particles::quantity::{ProbabilityTable, VariableQuantity};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use std::time::Duration;
use rand::seq::SliceRandom;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum ParticlePositionSource {
    /// All particles are emitted from one point
    Static(Vec2D),

    /// All particles in each cascade are emitted from a random point within a rectangle.
    RandomCascade(RectF),

    /// Emitted randomly within a rectangle
    Rect(RectF),

    Lattice(Vec<Vec2D>),

    EphemeralLattice(Vec<Vec2D>),
}

impl ParticlePositionSource {
    pub const ORIGIN: Self = Self::Static(Vec2D::ZERO);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ParticleModulation {
    /// All available particles are emitted as soon as possible
    Cascade,

    /// A maximum number of particles are emitted
    CascadeLimit { count: u32 },

    /// A maximum number of particles are emitted at a constant time step
    Constant { count: u32, step: Duration },
}

impl ParticleModulation {
    pub const SINGLE: Self = Self::CascadeLimit { count: 1 };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParticleSourceState {
    Complete,
    Emit,
    Delay(Duration),
}

#[derive(Debug, Clone)]
pub struct ParticleProperties {
    rng: ThreadRng,
    sprites: Vec<ParticleSprite>,
    color: VariableQuantity<ParticleColor>,
    size: VariableQuantity<f64>,
    angular_velocity: VariableQuantity<f64>,
}

impl ParticleProperties {
    pub fn new<C, S, R>(sprites: &[ParticleSprite], color: C, size: S, angular_velocity: R) -> Self
    where
        C: Into<VariableQuantity<ParticleColor>>,
        S: Into<VariableQuantity<f64>>,
        R: Into<VariableQuantity<f64>>,
    {
        assert!(!sprites.is_empty());
        Self {
            rng: thread_rng(),
            sprites: sprites.to_vec(),
            color: color.into(),
            size: size.into(),
            angular_velocity: angular_velocity.into(),
        }
    }

    pub fn simple<S>(sprites: &[ParticleSprite], size: S) -> Self
    where
        S: Into<VariableQuantity<f64>>,
    {
        assert!(!sprites.is_empty());
        Self::new(sprites, ParticleColor::WHITE, size, 0.0)
    }

    pub fn angular_velocity<R: Into<VariableQuantity<f64>>>(mut self, value: R) -> Self {
        self.angular_velocity = value.into();
        self
    }

    pub fn default() -> Self {
        Self::new(
            &[ParticleSprite::Circle05],
            ParticleColor::WHITE,
            (1.0, 0.0),
            0.0,
        )
    }

    pub fn next_sprite(&mut self) -> &ParticleSprite {
        if self.sprites.len() == 1 {
            self.sprites.first()
        } else {
            self.sprites.choose(&mut self.rng)
        }
        .unwrap()
    }
}

pub trait ParticleSource {
    fn is_complete(&self) -> bool;
    fn update(&mut self, delta_time: Duration, max_particles: u32) -> Vec<ParticleGroup>;
}

#[derive(Debug, Clone)]
pub struct RandomParticleSource {
    state: ParticleSourceState,
    position_source: ParticlePositionSource,
    modulation: ParticleModulation,
    anchor_for: Option<Duration>,
    fade_in: Option<Duration>,
    fade_out: bool,
    pulse: Option<VariableQuantity<ParticleWave>>,
    lifetime_secs: Option<VariableQuantity<f64>>,
    velocity: VariableQuantity<Vec2D>,
    acceleration: VariableQuantity<Vec2D>,
    alpha: VariableQuantity<f64>,
    orbit: Option<Vec2D>,
    properties: ProbabilityTable<ParticleProperties>,
}

impl ParticleSource for RandomParticleSource {
    fn is_complete(&self) -> bool {
        self.state == ParticleSourceState::Complete
    }

    fn update(&mut self, delta_time: Duration, max_particles: u32) -> Vec<ParticleGroup> {
        if self.state == ParticleSourceState::Complete {
            return vec![];
        }
        let _limit_emitted: Option<u32> = None;
        let emit_particles = match self.modulation {
            ParticleModulation::Cascade => self.cascade(max_particles),
            ParticleModulation::CascadeLimit { count } => self.cascade(count),
            ParticleModulation::Constant { count, step } => self.constant(count, step, delta_time),
        }
        .min(max_particles);

        if emit_particles == 0 {
            return vec![];
        }

        let particles: Vec<Particle> = match &mut self.position_source {
            ParticlePositionSource::EphemeralLattice(points) => {
                let new_length = points.len() - (emit_particles as usize).min(points.len());
                if new_length == 0 {
                    self.state = ParticleSourceState::Complete;
                }
                points.drain(new_length..).collect::<Vec<Vec2D>>()
            }
            ParticlePositionSource::Lattice(points) => points
                .iter()
                .take(emit_particles as usize)
                .copied()
                .collect::<Vec<Vec2D>>(),
            ParticlePositionSource::RandomCascade(_rect) => {
                let point = self.next_position();
                (0..emit_particles).map(|_| point).collect()
            }
            _ => (0..emit_particles).map(|_| self.next_position()).collect(),
        }
        .into_iter()
        .map(|p| self.next_particle(p))
        .collect();

        vec![ParticleGroup::new(
            self.anchor_for.map(|d| d.as_secs_f64()),
            self.fade_in.map(|d| d.as_secs_f64()),
            self.fade_out,
            self.orbit,
            particles,
        )]
    }
}

// todo trait this out so I can have an aggregate particle source
impl RandomParticleSource {
    pub fn new(position_source: ParticlePositionSource, modulation: ParticleModulation) -> Self {
        Self {
            state: ParticleSourceState::Emit,
            position_source,
            modulation,
            anchor_for: None,
            fade_in: None,
            fade_out: false,
            pulse: None,
            lifetime_secs: None,
            velocity: VariableQuantity::new(Vec2D::ZERO, Vec2D::ZERO),
            acceleration: VariableQuantity::new(Vec2D::ZERO, Vec2D::ZERO),
            alpha: VariableQuantity::new(1.0, 0.0),
            orbit: None,
            properties: ProbabilityTable::identity(ParticleProperties::default()),
        }
    }

    pub fn burst<C, V, L, A>(
        position_source: ParticlePositionSource,
        sprite: ParticleSprite,
        color: C,
        velocity: V,
        fade_out: L,
        alpha: A,
    ) -> Self
    where
        C: Into<VariableQuantity<ParticleColor>>,
        V: Into<VariableQuantity<Vec2D>>,
        L: Into<VariableQuantity<f64>>,
        A: Into<VariableQuantity<f64>>,
    {
        Self {
            state: ParticleSourceState::Emit,
            position_source,
            modulation: ParticleModulation::Cascade,
            anchor_for: None,
            fade_in: None,
            fade_out: true,
            pulse: None,
            lifetime_secs: Some(fade_out.into()),
            velocity: velocity.into(),
            acceleration: VariableQuantity::new(Vec2D::ZERO, Vec2D::ZERO),
            alpha: alpha.into(),
            orbit: None,
            properties: ProbabilityTable::identity(ParticleProperties::new(
                &[sprite],
                color,
                1.0,
                0.0,
            )),
        }
    }

    pub fn into_box(self) -> Box<dyn ParticleSource> {
        Box::new(self)
    }

    pub fn with_modulation(mut self, modulation: ParticleModulation) -> Self {
        self.modulation = modulation;
        self
    }

    pub fn with_alpha<A: Into<VariableQuantity<f64>>>(mut self, value: A) -> Self {
        self.alpha = value.into();
        self
    }

    pub fn with_static_properties<C, S, R>(
        self,
        sprite: ParticleSprite,
        color: C,
        size: S,
        angular_velocity: R,
    ) -> Self
    where
        C: Into<VariableQuantity<ParticleColor>>,
        S: Into<VariableQuantity<f64>>,
        R: Into<VariableQuantity<f64>>,
    {
        self.with_properties(ProbabilityTable::identity(ParticleProperties::new(
            &[sprite],
            color,
            size,
            angular_velocity,
        )))
    }

    pub fn with_properties(mut self, properties: ProbabilityTable<ParticleProperties>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_anchor(mut self, value: Duration) -> Self {
        self.anchor_for = Some(value);
        self
    }

    pub fn with_fade_in(mut self, value: Duration) -> Self {
        self.fade_in = Some(value);
        self
    }

    pub fn with_fade_out<L: Into<VariableQuantity<f64>>>(mut self, value: L) -> Self {
        self.fade_out = true;
        self.lifetime_secs = Some(value.into());
        self
    }

    pub fn with_velocity<V: Into<VariableQuantity<Vec2D>>>(mut self, value: V) -> Self {
        self.velocity = value.into();
        self
    }

    pub fn with_acceleration<A: Into<VariableQuantity<Vec2D>>>(mut self, value: A) -> Self {
        self.acceleration = value.into();
        self
    }

    pub fn with_orbit<O: Into<Vec2D>>(mut self, value: O) -> Self {
        self.orbit = Some(value.into());
        self
    }

    pub fn with_pulse<P: Into<VariableQuantity<ParticleWave>>>(mut self, value: P) -> Self {
        self.pulse = Some(value.into());
        self
    }

    fn cascade(&mut self, count: u32) -> u32 {
        self.state = ParticleSourceState::Complete;
        count
    }

    fn constant(&mut self, count: u32, step: Duration, delta_time: Duration) -> u32 {
        match self.state {
            ParticleSourceState::Emit => {
                self.state = ParticleSourceState::Delay(step);
                count
            }
            ParticleSourceState::Delay(delay) => {
                let delta_time_nanos = delay.as_nanos() as u64 + delta_time.as_nanos() as u64;
                let step_nanos = step.as_nanos() as u64;
                let n_steps = delta_time_nanos / step_nanos;
                self.state =
                    ParticleSourceState::Delay(Duration::from_nanos(delta_time_nanos % step_nanos));
                n_steps as u32 * count
            }
            _ => unreachable!(),
        }
    }

    fn next_position(&self) -> Vec2D {
        match self.position_source {
            ParticlePositionSource::Static(point) => point,
            ParticlePositionSource::RandomCascade(rect) | ParticlePositionSource::Rect(rect) => {
                let x = rect.x() + rect.width() * rand::random::<f64>();
                let y = rect.y() + rect.height() * rand::random::<f64>();
                Vec2D::new(x, y)
            }
            _ => unreachable!(),
        }
    }

    fn next_particle(&mut self, position: Vec2D) -> Particle {
        let max_alpha = self.alpha.next();
        let properties = self.properties.next_mut();
        Particle::new(
            position,
            self.velocity.next(),
            self.acceleration.next(),
            max_alpha,
            if self.fade_in.is_some() {
                0.0
            } else {
                max_alpha
            },
            self.pulse.as_mut().map(|p| p.next()),
            properties.color.next(),
            self.lifetime_secs.as_mut().map(|l| l.next()),
            *properties.next_sprite(),
            properties.size.next(),
            properties.angular_velocity.next(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct AggregateParticleSource {
    sources: Vec<RandomParticleSource>,
}

impl AggregateParticleSource {
    pub fn new(sources: Vec<RandomParticleSource>) -> Self {
        Self { sources }
    }

    pub fn into_box(self) -> Box<dyn ParticleSource> {
        Box::new(self)
    }
}

impl ParticleSource for AggregateParticleSource {
    fn is_complete(&self) -> bool {
        self.sources.iter().all(|s| s.is_complete())
    }

    fn update(&mut self, delta_time: Duration, max_particles: u32) -> Vec<ParticleGroup> {
        let mut result = vec![];
        let mut max_particles = max_particles;
        for source in self.sources.iter_mut() {
            if max_particles == 0 {
                break;
            }
            let groups = source.update(delta_time, max_particles);
            for group in groups {
                max_particles -= (group.len() as u32).min(max_particles);
                result.push(group)
            }
        }
        result
    }
}
