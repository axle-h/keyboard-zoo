use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::characters::Character;
use crate::characters::sprites::CharacterSprites;
use crate::characters::pac_man::pac_man_sprites;
use crate::game::physics::CharacterBody;

pub struct CharacterRender<'a> {
    pac_man: CharacterSprites<'a>
}

impl<'a> CharacterRender<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        Ok(Self { pac_man: pac_man_sprites(texture_creator)? })
    }

    pub fn draw_character(&self, canvas: &mut WindowCanvas, body: CharacterBody) -> Result<(), String> {
        match body.character() {
            Character::PacMan(pac_man) => {
                let (state, frame) = pac_man.lifetime().animation_frame();
                let sprites = if state.is_alive() {
                    self.pac_man.sprites()
                } else {
                    self.pac_man.death_sprites()
                };
                sprites.draw_frame(canvas, body.aabb(), frame, body.angle())
            }
        }
    }
}