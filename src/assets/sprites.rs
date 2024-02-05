use std::collections::HashMap;
use std::io::Cursor;
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::assets::geometry::{SpriteAsset, SpriteAssetSheet};
use crate::game::physics::AssetBody;
use crate::assets::letters;
use crate::assets::numbers;

struct BagRandom<T> {
    rng: ThreadRng,
    sample: Vec<T>
}

impl<T> BagRandom<T> {
    pub fn new(sample: Vec<T>) -> Self {
        Self { rng: thread_rng(), sample }
    }
}

pub struct Sprites<'a> {
    sprites: Vec<SpriteAsset>,
    sprites_by_name: HashMap<String, SpriteAsset>,
    sprites_by_char: HashMap<char, Vec<SpriteAsset>>,
    rng: ThreadRng,
    letters: Texture<'a>,
    numbers: Texture<'a>
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
        let sprites = Self::load_sprites(letters::SPRITES_JSON_ZST)?
            .into_iter()
            .chain(Self::load_sprites(numbers::SPRITES_JSON_ZST)?.into_iter())
            .collect::<Vec<SpriteAsset>>();

        let mut sprites_by_char: HashMap<char, Vec<SpriteAsset>> = HashMap::new();
        let mut sprites_by_name: HashMap<String, SpriteAsset> = HashMap::new();
        for sprite in sprites.iter() {
            if sprites_by_name.insert(sprite.name().to_string(), sprite.clone()).is_some() {
                return Err(format!("duplicate sprite {}", sprite.name()))
            }

            let character = sprite.character().to_ascii_uppercase();
            if let Some(entries) = sprites_by_char.get_mut(&character) {
                entries.push(sprite.clone());
            } else {
                sprites_by_char.insert(character, vec![sprite.clone()]);
            }
        }

        let letters = texture_creator.load_texture_bytes(letters::SPRITES_PNG)?;
        let numbers = texture_creator.load_texture_bytes(numbers::SPRITES_PNG)?;

        Ok(Self { sprites, sprites_by_name, sprites_by_char, rng: Default::default(), letters, numbers })
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
                if sprite.character().is_numeric() { &self.numbers } else { &self.letters },
                Rect::new(snip.x() as i32, snip.y() as i32, snip.width(), snip.height()),
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