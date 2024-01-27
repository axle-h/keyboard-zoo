use std::collections::HashMap;
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::assets::geometry::{SpriteAsset, SpriteAssetSheet};
use crate::game::physics::{AssetBody, Body};

const SPRITES_JSON: &[u8] = include_bytes!("sprites.json");
const SPRITES_PNG: &[u8] = include_bytes!("sprites.png");

pub fn deserialize_sprites() -> Result<SpriteAssetSheet, String> {
    serde_json::from_slice(&SPRITES_JSON).map_err(|e| e.to_string())
}

pub struct Sprites<'a> {
    sprites: Vec<SpriteAsset>,
    sprites_by_name: HashMap<String, SpriteAsset>,
    sprites_by_char: HashMap<char, Vec<SpriteAsset>>,
    rng: ThreadRng,
    texture: Texture<'a>
}

impl<'a> Sprites<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        let sprite_sheet: SpriteAssetSheet = serde_json::from_slice(&SPRITES_JSON).map_err(|e| e.to_string())?;
        let mut sprites_by_char: HashMap<char, Vec<SpriteAsset>> = HashMap::new();
        let mut sprites_by_name: HashMap<String, SpriteAsset> = HashMap::new();
        let mut sprites = vec![];
        for sprite in sprite_sheet.sprites().into_iter() {
            sprites.push(sprite.clone());

            if sprites_by_name.insert(sprite.name().to_string(), sprite.clone()).is_some() {
                return Err(format!("duplicate sprite {}", sprite.name()))
            }

            let key = sprite.name().chars().next().expect("sprite name is empty").to_ascii_uppercase();
            if let Some(entries) = sprites_by_char.get_mut(&key) {
                entries.push(sprite.clone());
            } else {
                sprites_by_char.insert(key, vec![sprite.clone()]);
            }
        }

        let texture = texture_creator.load_texture_bytes(SPRITES_PNG)?;

        Ok(Self { sprites, sprites_by_name, sprites_by_char, rng: Default::default(), texture })
    }

    pub fn names(&self) -> Vec<String> {
        self.sprites_by_name.keys().into_iter().cloned().collect()
    }

    pub fn pick_sprite_by_char(&mut self, key: char) -> Option<SpriteAsset> {
        self.sprites_by_char.get(&key)
            .and_then(|entries| entries.choose(&mut self.rng))
            .cloned()
    }

    pub fn pick_random_sprite(&mut self) -> SpriteAsset {
        self.sprites.choose(&mut self.rng).cloned().unwrap()
    }

    pub fn draw_sprite(&self, canvas: &mut WindowCanvas, body: AssetBody) -> Result<(), String> {
        if let Some(sprite) = self.sprites_by_name.get(body.asset_name()) {
            let snip = sprite.snip();
            canvas.copy_ex(
                &self.texture,
                Rect::new(snip.x() as i32, snip.y() as i32, snip.width() as u32, snip.height() as u32),
                body.aabb(),
                body.angle(),
                None,
                false,
                false
            )
        } else {
            Err(format!("unknown sprite {}", body.asset_name()))
        }
    }
}