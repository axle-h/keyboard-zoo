use sdl2::mixer::{Channel, Chunk};

pub trait Playable {
    fn play(&self) -> Result<Channel, String>;

    fn try_play(&self);
}

impl Playable for Chunk {
    fn play(&self) -> Result<Channel, String> {
        Channel::all().play(self, 0)
    }

    fn try_play(&self) {
        let _ = self.play();
    }
}