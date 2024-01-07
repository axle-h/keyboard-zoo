use crate::particles::color::ParticleColor;
use crate::particles::geometry::Vec2D;
use crate::particles::particle::ParticleWave;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

#[derive(Clone, Debug)]
pub struct VariableQuantity<T: Clone> {
    rng: ThreadRng,
    quantity: T,
    variance: T,
}

impl<T: Clone> VariableQuantity<T> {
    pub fn new(quantity: T, variance: T) -> Self {
        Self {
            rng: thread_rng(),
            quantity,
            variance,
        }
    }

    fn rand_signed_f64(&mut self) -> f64 {
        2.0 * self.rng.gen::<f64>() - 1.0
    }
}

impl<T: Clone> From<(T, T)> for VariableQuantity<T> {
    fn from((quantity, variance): (T, T)) -> Self {
        VariableQuantity::new(quantity, variance)
    }
}

impl From<f64> for VariableQuantity<f64> {
    fn from(quantity: f64) -> Self {
        VariableQuantity::new(quantity, 0.0)
    }
}

impl From<Vec2D> for VariableQuantity<Vec2D> {
    fn from(quantity: Vec2D) -> Self {
        VariableQuantity::new(quantity, Vec2D::ZERO)
    }
}

impl From<ParticleColor> for VariableQuantity<ParticleColor> {
    fn from(quantity: ParticleColor) -> Self {
        VariableQuantity::new(quantity, ParticleColor::ZERO)
    }
}

impl VariableQuantity<f64> {
    pub fn next(&mut self) -> f64 {
        self.quantity + self.variance * self.rand_signed_f64()
    }
}

impl VariableQuantity<Vec2D> {
    pub fn next(&mut self) -> Vec2D {
        self.quantity
            + Vec2D::new(
                self.variance.x() * self.rand_signed_f64(),
                self.variance.y() * self.rand_signed_f64(),
            )
    }
}

impl VariableQuantity<ParticleColor> {
    pub fn next(&mut self) -> ParticleColor {
        self.quantity
            + self.variance * ParticleColor::rgb(self.rng.gen(), self.rng.gen(), self.rng.gen())
    }
}

impl VariableQuantity<ParticleWave> {
    pub fn next(&mut self) -> ParticleWave {
        let magnitude =
            self.quantity.magnitude() + self.variance.magnitude() * self.rng.gen::<f64>();
        let frequency =
            self.quantity.frequency() + self.variance.frequency() * self.rng.gen::<f64>();
        ParticleWave::new(magnitude, frequency)
    }
}

#[derive(Clone, Debug)]
struct ProbabilityRow<T: Clone> {
    value: T,
    range_from: f64,
    range_to: f64,
}

impl<T: Clone> ProbabilityRow<T> {
    fn matches(&self, random: f64) -> bool {
        random >= self.range_from && random <= self.range_to
    }
}

#[derive(Clone, Debug)]
pub struct ProbabilityTable<T: Clone> {
    rows: Vec<ProbabilityRow<T>>,
    total: f64,
    rng: ThreadRng,
}

impl<T: Clone> ProbabilityTable<T> {
    pub fn new() -> Self {
        Self {
            rows: vec![],
            rng: thread_rng(),
            total: 0.0,
        }
    }

    pub fn identity(value: T) -> Self {
        Self {
            rows: vec![],
            rng: thread_rng(),
            total: 0.0,
        }
        .with(value, 1.0)
    }

    pub fn with<Z: Into<T>>(mut self, value: Z, probability: f64) -> Self {
        self.rows.push(ProbabilityRow {
            value: value.into(),
            range_from: self.total,
            range_to: self.total + probability,
        });
        self.total += probability;
        self
    }

    pub fn with_1<Z: Into<T>>(self, value: Z) -> Self {
        self.with(value, 1.0)
    }

    pub fn next_mut(&mut self) -> &mut T {
        if self.rows.is_empty() {
            panic!("no probability rows")
        }

        &mut match self.rows.len() {
            0 => panic!("no probability rows"),
            1 => self.rows.first_mut().unwrap(),
            _ => {
                let random = self.total * self.rng.gen::<f64>();
                self.rows.iter_mut().find(|r| r.matches(random)).unwrap()
            }
        }
        .value
    }
}
