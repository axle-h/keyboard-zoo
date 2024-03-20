use std::fmt::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct SpritePoint {
    pub x: f64,
    pub y: f64
}

impl SpritePoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
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
    pub fn new(points: [SpritePoint; 3], color: [u8; 3]) -> Self {
        Self { points, color }
    }

    pub fn points(&self) -> [SpritePoint; 3] {
        self.points
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
        let triangle = SpriteTriangle::new(
            points.try_into().unwrap(),
            [seq.next_element::<u8>()?.unwrap(), seq.next_element::<u8>()?.unwrap(), seq.next_element::<u8>()?.unwrap()]
        );
        Ok(triangle)
    }
}

impl<'de> Deserialize<'de> for SpriteTriangle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(TriangleVisitor)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub struct SpriteSnip {
    x: u32,
    y: u32,
    width: u32,
    height: u32
}

impl SpriteSnip {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_corners((x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> Self {
        Self::new(x1, y1, x2 - x1, y2 - y1)
    }

    pub fn bottom_right(&self) -> (u32, u32) {
        (self.x + self.width, self.y + self.height)
    }

    pub fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn contains(&self, other: &SpriteSnip) -> bool {
        let (self_x2, self_y2) = self.bottom_right();
        let (other_x2, other_y2) = other.bottom_right();

        other.x >= self.x && other_x2 <= self_x2 && other.y >= self.y && other_y2 <= self_y2
    }

    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SpriteAsset {
    name: String,
    character: char,
    snip: SpriteSnip,
    triangles: Vec<SpriteTriangle>
}

impl SpriteAsset {
    pub fn new(name: String, character: char, snip: SpriteSnip, triangles: Vec<SpriteTriangle>) -> Self {
        Self { name, character, snip, triangles }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn character(&self) -> char {
        self.character
    }
    pub fn snip(&self) -> SpriteSnip {
        self.snip
    }
    pub fn triangles(&self) -> &Vec<SpriteTriangle> {
        &self.triangles
    }

    pub fn unit_scale(&self) -> (f64, f64) {
        let unit_scale = 1.0 / self.snip.width.max(self.snip.height) as f64;
        (unit_scale * self.snip.width as f64, unit_scale * self.snip.height as f64)
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


    pub fn into_sprites(self) -> Vec<SpriteAsset> {
        self.sprites
    }
}