use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Vec2D {
    x: f64,
    y: f64,
}

const CMP_EPSILON: f64 = 1e-6;

macro_rules! approx_eq {
    ($a:expr, $b:expr) => {{
        ($a - $b).abs() < CMP_EPSILON
    }};
}

impl Vec2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const ZERO: Vec2D = Vec2D::new(0.0, 0.0);

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn is_zero(&self) -> bool {
        self == &Vec2D::ZERO
    }

    pub fn unit_vector(&self) -> Self {
        if self.is_zero() {
            return *self;
        }

        let n = self.x.powi(2) + self.y.powi(2);
        if approx_eq!(n, 1.0) {
            // already normalized.
            return *self;
        }
        let n = n.sqrt();
        Self::new(self.x / n, self.y / n)
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.x().powi(2) + self.y().powi(2)
    }
}

impl PartialEq for Vec2D {
    fn eq(&self, other: &Self) -> bool {
        approx_eq!(self.x, other.x) && approx_eq!(self.y, other.y)
    }
}

impl Add for Vec2D {
    type Output = Vec2D;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<f64> for Vec2D {
    type Output = Vec2D;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2D::new(self.x * rhs, self.y * rhs)
    }
}

impl Sub for Vec2D {
    type Output = Vec2D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl From<(f64, f64)> for Vec2D {
    fn from((x, y): (f64, f64)) -> Vec2D {
        Vec2D::new(x, y)
    }
}

impl From<Vec2D> for (f64, f64) {
    fn from(val: Vec2D) -> Self {
        (val.x, val.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectF {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl RectF {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        assert!(width > 0.0);
        assert!(height > 0.0);
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }
}

impl From<(f64, f64, f64, f64)> for RectF {
    fn from((x, y, width, height): (f64, f64, f64, f64)) -> RectF {
        RectF::new(x, y, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_1_SQRT_2;

    #[test]
    fn unit_vector_of_zero_is_zero() {
        assert_eq!(Vec2D::ZERO.unit_vector(), Vec2D::ZERO);
    }

    #[test]
    fn unit_vector_of_unit_vector_is_equal() {
        let point = Vec2D::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2);
        assert_eq!(point.unit_vector(), point);
    }

    #[test]
    fn unit_vector() {
        let normal = Vec2D::new(123.456, 789.0).unit_vector();
        assert_eq!(normal, Vec2D::new(0.15459048205347672, 0.9879786348188272));
    }
}
