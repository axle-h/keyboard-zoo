use sdl2::mixer::Chunk;

pub trait Playable {
    fn play(&self) -> Result<(), String>;
}

impl Playable for Chunk {
    fn play(&self) -> Result<(), String> {
        // TODO ignore cannot play sound
        sdl2::mixer::Channel::all().play(self, 0)?;
        Ok(())
    }
}