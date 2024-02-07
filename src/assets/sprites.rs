use std::collections::HashMap;
use std::io::Cursor;
use itertools::Itertools;
use std::default::Default;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::assets::geometry::{SpriteAsset, SpriteAssetSheet};
use crate::game::physics::Body;
use crate::assets::letters;
use crate::assets::numbers;
use crate::random::BagRandom;

pub struct Sprites<'a> {
    char_bag: BagRandom<char>,
    sprites_by_name: HashMap<String, SpriteAsset>,
    sprites_by_char: HashMap<char, BagRandom<SpriteAsset>>,
    letters: Texture<'a>,
    numbers: Texture<'a>,
}

impl<'a> Sprites<'a> {
    fn load_sprites(src:  &'static [u8]) -> Result<Vec<SpriteAsset>, String> {
        let decompressed = zstd::stream::Decoder::new(Cursor::new(src))
            .map_err(|e| e.to_string())?;

        let sprite_sheet: SpriteAssetSheet = serde_json::from_reader(decompressed)
            .map_err(|e| e.to_string())?;
        Ok(sprite_sheet.into_sprites())
    }

    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        let sprites = Self::load_sprites(letters::SPRITES_JSON_ZST)?.into_iter()
            .chain(Self::load_sprites(numbers::SPRITES_JSON_ZST)?.into_iter())
            .collect::<Vec<SpriteAsset>>();

        let total_triangles: usize = sprites.iter().map(|s| s.triangles().len()).sum();
        dbg!(total_triangles); // 1.0 = 50965, 2.0 = 34333

        let sprites_by_name: HashMap<String, SpriteAsset> = sprites.iter()
            .map(|sprite| (sprite.name().to_string(), sprite.clone()))
            .collect();

        let mut sprites_by_char: HashMap<char, BagRandom<SpriteAsset>> = sprites.into_iter()
            .sorted_by(|a, b| a.character().cmp(&b.character()))
            .group_by(|sprite| sprite.character().to_ascii_uppercase())
            .into_iter()
            .map(|(ch, grp)| (ch, BagRandom::new(grp.collect())))
            .collect();

        let char_bag = BagRandom::new(sprites_by_char.keys().copied().collect());

        let letters = texture_creator.load_texture_bytes(letters::SPRITES_PNG)?;
        let numbers = texture_creator.load_texture_bytes(numbers::SPRITES_PNG)?;

        Ok(Self { sprites_by_name, sprites_by_char, letters, numbers, char_bag })
    }

    pub fn names(&self) -> Vec<String> {
        self.sprites_by_name.keys().into_iter().cloned().collect()
    }

    pub fn pick_sprite_by_char(&mut self, key: char) -> Option<SpriteAsset> {
        self.sprites_by_char.get_mut(&key)
            .and_then(|entries| entries.next())
            .map(|sprite| sprite.as_ref().clone()) // todo maybe we can leave it in a box
    }

    pub fn pick_random_sprite(&mut self) -> SpriteAsset {
        let ch = self.char_bag.next().unwrap();
        self.pick_sprite_by_char(*ch).unwrap()
    }

    pub fn draw_sprite(&self, canvas: &mut WindowCanvas, name: &str, aabb: Rect, angle: f64) -> Result<(), String> {
        if let Some(sprite) = self.sprites_by_name.get(name) {
            let snip = sprite.snip();
            canvas.copy_ex(
                if sprite.character().is_numeric() { &self.numbers } else { &self.letters },
                Rect::new(snip.x() as i32, snip.y() as i32, snip.width(), snip.height()),
                aabb,
                angle,
                None,
                false,
                false
            )
        } else {
            Err(format!("unknown sprite {}", name))
        }
    }
}