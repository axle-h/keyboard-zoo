use crate::particles::meta::ParticleSprite::*;
use sdl2::rect::Rect;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::game::polygon::Triangle;

const PARTICLE_SPRITE_SIZE: u32 = 512;

fn snip(i: i32, j: i32) -> Rect {
    Rect::new(
        i * PARTICLE_SPRITE_SIZE as i32,
        j * PARTICLE_SPRITE_SIZE as i32,
        PARTICLE_SPRITE_SIZE,
        PARTICLE_SPRITE_SIZE,
    )
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum ParticleSprite {
    Circle01,
    Circle02,
    Circle03,
    Circle04,
    Circle05,
    Dirt01,
    Dirt02,
    Dirt03,
    Fire01,
    Fire02,
    Flare01,
    Light01,
    Light02,
    Light03,
    Magic01,
    Magic02,
    Magic03,
    Magic04,
    Magic05,
    Scorch01,
    Scorch02,
    Scorch03,
    Smoke01,
    Smoke02,
    Smoke03,
    Smoke04,
    Smoke05,
    Smoke06,
    Smoke07,
    Smoke08,
    Smoke09,
    Smoke10,
    Spark01,
    Spark02,
    Spark03,
    Spark04,
    Star01,
    Star02,
    Star03,
    Star04,
    Star05,
    Star06,
    Star07,
    Star08,
    Star09,
    Symbol01,
    Symbol02,
    Twirl01,
    Twirl02,
    Twirl03,
    SpriteTriangle(Triangle)
}

impl ParticleSprite {
    pub const STARS: [ParticleSprite; 9] = [
        Star01, Star02, Star03, Star04, Star05, Star06, Star07, Star08, Star09,
    ];
    pub const HOLLOW_CIRCLES: [ParticleSprite; 4] = [Circle01, Circle02, Circle03, Circle04];

    pub fn all_sprite_based() -> Vec<Self> {
        Self::iter().filter(|s| s.snip().is_some()).collect()
    }

    pub fn snip(&self) -> Option<Rect> {
        match self {
            Circle01 => Some(snip(0, 0)),
            Circle02 => Some(snip(1, 0)),
            Circle03 => Some(snip(2, 0)),
            Circle04 => Some(snip(3, 0)),
            Circle05 => Some(snip(4, 0)),
            Dirt01 => Some(snip(5, 0)),
            Dirt02 => Some(snip(6, 0)),
            Twirl03 => Some(snip(7, 0)),
            Dirt03 => Some(snip(0, 1)),
            Fire01 => Some(snip(1, 1)),
            Fire02 => Some(snip(2, 1)),
            Flare01 => Some(snip(3, 1)),
            Light01 => Some(snip(4, 1)),
            Light02 => Some(snip(5, 1)),
            Light03 => Some(snip(6, 1)),
            Magic01 => Some(snip(0, 2)),
            Magic02 => Some(snip(1, 2)),
            Magic03 => Some(snip(2, 2)),
            Magic04 => Some(snip(3, 2)),
            Magic05 => Some(snip(4, 2)),
            Scorch01 => Some(snip(5, 2)),
            Scorch02 => Some(snip(6, 2)),
            Scorch03 => Some(snip(0, 3)),
            Smoke01 => Some(snip(1, 3)),
            Smoke02 => Some(snip(2, 3)),
            Smoke03 => Some(snip(3, 3)),
            Smoke04 => Some(snip(4, 3)),
            Smoke05 => Some(snip(5, 3)),
            Smoke06 => Some(snip(6, 3)),
            Smoke07 => Some(snip(0, 4)),
            Smoke08 => Some(snip(1, 4)),
            Smoke09 => Some(snip(2, 4)),
            Smoke10 => Some(snip(3, 4)),
            Spark01 => Some(snip(4, 4)),
            Spark02 => Some(snip(5, 4)),
            Spark03 => Some(snip(6, 4)),
            Spark04 => Some(snip(0, 5)),
            Star01 => Some(snip(1, 5)),
            Star02 => Some(snip(2, 5)),
            Star03 => Some(snip(3, 5)),
            Star04 => Some(snip(4, 5)),
            Star05 => Some(snip(5, 5)),
            Star06 => Some(snip(6, 5)),
            Star07 => Some(snip(0, 6)),
            Star08 => Some(snip(1, 6)),
            Star09 => Some(snip(2, 6)),
            Symbol01 => Some(snip(3, 6)),
            Symbol02 => Some(snip(4, 6)),
            Twirl01 => Some(snip(5, 6)),
            Twirl02 => Some(snip(6, 6)),
            _ => None
        }
    }
}
