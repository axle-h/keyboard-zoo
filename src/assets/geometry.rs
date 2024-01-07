use std::fmt::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct SpritePoint {
    x: f64,
    y: f64
}

impl SpritePoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteTriangle {
    points: [SpritePoint; 3],
    color: [u8; 3]
}

impl SpriteTriangle {
    pub fn new(points: [SpritePoint; 3]) -> Self {
        Self { points, color: [0, 0, 0] }
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        self.color = [r, g, b];
    }

    pub fn points(&self) -> [SpritePoint; 3] {
        self.points
    }

    pub fn contains_point(&self, p: SpritePoint) -> bool {
        let [p0, p1, p2] = self.points;
        let a = 0.5 * (-p1.y * p2.x + p0.y * (-p1.x + p2.x) + p0.x * (p1.y - p2.y) + p1.x * p2.y);
        let sign = if a < 0.0 { -1.0 } else { 1.0 };
        let s = (p0.y * p2.x - p0.x * p2.y + (p2.y - p0.y) * p.x + (p0.x - p2.x) * p.y) * sign;
        let t = (p0.x * p1.y - p0.y * p1.x + (p0.y - p1.y) * p.x + (p1.x - p0.x) * p.y) * sign;
        s > 0.0 && t > 0.0 && (s + t) < 2.0 * a * sign
    }

    fn aabb(&self) -> SpriteRect {
        let x0 = self.points.map(|p| p.x).into_iter().min_by(|&a, b| a.partial_cmp(b).unwrap()).unwrap();
        let y0 = self.points.map(|p| p.y).into_iter().min_by(|&a, b| a.partial_cmp(b).unwrap()).unwrap();
        let x1 = self.points.map(|p| p.x).into_iter().max_by(|&a, b| a.partial_cmp(b).unwrap()).unwrap();
        let y1 = self.points.map(|p| p.y).into_iter().max_by(|&a, b| a.partial_cmp(b).unwrap()).unwrap();
        SpriteRect::from_p1_p2(x0, y0, x1, y1)
    }

    pub fn interior_points(&self) -> Vec<SpritePoint> {
        self.aabb().interior_points().into_iter()
            .filter(|p| self.contains_point(*p))
            .collect()
    }

    pub fn color(&self) -> [u8; 3] {
        self.color
    }
}

impl Serialize for SpriteTriangle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.points.len() * 2))?;
        for p in self.points {
            seq.serialize_element(&p.x)?;
            seq.serialize_element(&p.y)?;
        }
        for i in 0..3 {
            seq.serialize_element(&self.color[i])?;
        }
        seq.end()
    }
}

struct TriangleVisitor;
impl<'de> Visitor<'de> for TriangleVisitor {
    type Value = SpriteTriangle;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("sequence of 6 coordinates & 3 color components")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut points = vec![];
        while let Some(x) = seq.next_element()? {
            if let Some(y) = seq.next_element()? {
                points.push(SpritePoint::new(x, y));
            } else {
                return Err(serde::de::Error::custom(format!("not an even length sequence")));
            }
            if points.len() == 3 {
                break;
            }
        }
        assert_eq!(points.len(), 3);
        let mut triangle = SpriteTriangle::new(points.try_into().unwrap());
        triangle.set_color(seq.next_element::<u8>()?.unwrap(), seq.next_element::<u8>()?.unwrap(), seq.next_element::<u8>()?.unwrap());
        Ok(triangle)
    }
}

impl<'de> Deserialize<'de> for SpriteTriangle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(TriangleVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct SpriteRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl SpriteRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_p1_p2(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    pub fn lower_bound(&self) -> SpritePoint {
        SpritePoint::new(self.x, self.y)
    }

    pub fn upper_bound(&self) -> SpritePoint {
        SpritePoint::new(self.x + self.width, self.y + self.height)
    }

    pub fn interior_points(&self) -> Vec<SpritePoint> {
        let mut result = vec![];
        let lower_bound = self.lower_bound();
        let upper_bound = self.upper_bound();
        for x in lower_bound.x.floor() as u32 ..= upper_bound.x.ceil() as u32 {
            for y in lower_bound.y.floor() as u32 ..= upper_bound.y.ceil() as u32 {
                result.push(SpritePoint::new(x as f64, y as f64));
            }
        }
        result
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SpriteAsset {
    name: String,
    snip: SpriteRect,
    triangles: Vec<SpriteTriangle>,
    unit_scale: f64
}

impl SpriteAsset {
    pub fn new(name: String, snip: SpriteRect, triangles: Vec<SpriteTriangle>, unit_scale: f64) -> Self {
        Self { name, snip, triangles, unit_scale }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn snip(&self) -> SpriteRect {
        self.snip
    }
    pub fn triangles(&self) -> &Vec<SpriteTriangle> {
        &self.triangles
    }

    pub fn aabb(&self) -> SpriteRect {
        SpriteRect::new(0.0, 0.0, self.unit_scale * self.snip.width, self.unit_scale * self.snip.height)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SpriteAssetSheet {
    sprites: Vec<SpriteAsset>
}

impl SpriteAssetSheet {
    pub fn new(sprites: Vec<SpriteAsset>) -> Self {
        Self { sprites }
    }


    pub fn sprites(&self) -> &Vec<SpriteAsset> {
        &self.sprites
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_contains_point() {
        let triangle = SpriteTriangle::new([
            SpritePoint::new(88.0, 260.0),
            SpritePoint::new(99.0, 121.0),
            SpritePoint::new(259.0, 111.0)
        ]);
        assert!(triangle.contains_point(SpritePoint::new(135.0, 162.0)));
        assert!(!triangle.contains_point(SpritePoint::new(204.0, 182.0)));
    }
}