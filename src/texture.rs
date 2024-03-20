use sdl2::image::LoadTexture;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::video::WindowContext;

pub trait TextureQuery {
    fn size(&self) -> (u32, u32);
}

impl TextureQuery for Texture<'_> {
    fn size(&self) -> (u32, u32) {
        let query = self.query();
        (query.width, query.height)
    }
}

pub trait TextureFactory {
    fn create_texture_target_blended(&self, width: u32, height: u32) -> Result<Texture, String>;
    fn load_texture_bytes_blended(&self, buf: &[u8]) -> Result<Texture, String>;
}

impl TextureFactory for TextureCreator<WindowContext> {
    fn create_texture_target_blended(&self, width: u32, height: u32) -> Result<Texture, String> {
        let mut texture = self
            .create_texture_target(None, width, height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        Ok(texture)
    }

    fn load_texture_bytes_blended(&self, buf: &[u8]) -> Result<Texture, String> {
        let mut texture = self.load_texture_bytes(buf)?;
        texture.set_blend_mode(BlendMode::Blend);
        Ok(texture)
    }
}
