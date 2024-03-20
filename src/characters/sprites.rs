use sdl2::rect::{Rect};
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::texture::{TextureFactory, TextureQuery};

#[derive(Debug, Clone)]
enum FrameFormat {
    /// texture only contains a linear set of square frames, nothing else
    /// i.e. the width of a frame == height of frame == height of texture
    ExclusiveSquareLinear,

    /// texture only contains a linear set of frames, nothing else
    /// i.e. the width of a frame is texture width / frames
    ///      & height is same as texture height
    ExclusiveLinear { count: usize },
}

#[derive(Debug, Clone)]
pub struct SpriteSheetFormat {
    file: &'static [u8],
    format: FrameFormat,
}

impl SpriteSheetFormat {
    pub fn exclusive_square_linear(file: &'static [u8]) -> Self {
        Self {
            file,
            format: FrameFormat::ExclusiveSquareLinear,
        }
    }

    pub fn exclusive_linear(file: &'static [u8], frames: usize) -> Self {
        assert!(frames > 0);
        Self {
            file,
            format: FrameFormat::ExclusiveLinear { count: frames },
        }
    }

    pub fn sprite_sheet<'a>(
        &self,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<SpriteSheet<'a>, String> {
        let texture = texture_creator.load_texture_bytes_blended(self.file)?;
        let (texture_width, texture_height) = texture.size();

        let frames = match self.format {
            FrameFormat::ExclusiveLinear { count } => {
                let frame_width = texture_width / count as u32;
                (0..count as u32)
                    .map(|i| Rect::new((i * frame_width) as i32, 0, frame_width, texture_height))
                    .collect()
            }
            FrameFormat::ExclusiveSquareLinear => {
                let frame_size = texture_height;
                let count = texture_width / frame_size;
                (0..count)
                    .map(|i| Rect::new((i * frame_size) as i32, 0, frame_size, frame_size))
                    .collect()
            }
        };

        Ok(SpriteSheet::new(texture, frames))
    }
}

pub struct SpriteSheet<'a> {
    texture: Texture<'a>,
    frames: Vec<Rect>,
    frame_width: u32,
    frame_height: u32,
}

impl<'a> SpriteSheet<'a> {
    pub fn new(texture: Texture<'a>, frames: Vec<Rect>) -> Self {
        let first_frame = frames.first().expect("empty animation");
        Self {
            texture,
            frame_width: first_frame.width(),
            frame_height: first_frame.height(),
            frames,
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn frame_size(&self) -> (u32, u32) {
        (self.frame_width, self.frame_height)
    }

    pub fn draw_frame<A : Into<Option<f64>>>(
        &self,
        canvas: &mut WindowCanvas,
        dest: Rect,
        frame: usize,
        angle: A,
    ) -> Result<(), String> {
        let snip = self.frames[frame];
        if let Some(angle) = angle.into() {
            canvas.copy_ex(&self.texture, snip, dest, angle, None, false, false)
        } else {
            canvas.copy(&self.texture, snip, dest)
        }
    }

}

pub struct CharacterSprites<'a> {
    sprites: SpriteSheet<'a>,
    death_sprites: SpriteSheet<'a>,
}

impl<'a> CharacterSprites<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        sprites: SpriteSheetFormat,
        death_sprites: SpriteSheetFormat,
    ) -> Result<Self, String> {
        Ok(Self {
            sprites: sprites.sprite_sheet(texture_creator)?,
            death_sprites: death_sprites.sprite_sheet(texture_creator)?,
        })
    }

    pub fn sprites(&self) -> &SpriteSheet<'a> {
        &self.sprites
    }

    pub fn death_sprites(&self) -> &SpriteSheet<'a> {
        &self.death_sprites
    }
}