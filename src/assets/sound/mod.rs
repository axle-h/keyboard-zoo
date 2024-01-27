use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use sdl2::get_error;
use sdl2::mixer::{Chunk, Music};
use sdl2::rwops::RWops;
use sdl2::sys::mixer;
use crate::assets::sound::playable::Playable;
use crate::config::AudioConfig;

mod create;
pub mod playable;
mod destroy;
mod music;
mod explosion;

static mut MUSIC_QUEUE: Option<Rc<RefCell<VecDeque<Music<'static>>>>> = None;

pub struct Sound {
    rng: ThreadRng,
    create_sounds: HashMap<String, Chunk>,
    destroy_sounds: Vec<Chunk>,
    explosion_sounds: Vec<Chunk>
}

impl Sound {
    pub fn new(names: Vec<String>, config: AudioConfig) -> Result<Self, String> {
        let mut create_sounds = HashMap::new();

        for name in names.into_iter() {
            let create_chunk = config.load_chunk(create::asset(&name))?;
            create_sounds.insert(name, create_chunk);
        }

        let destroy_sounds = destroy::ASSETS.into_iter()
            .map(|b| config.load_chunk(b).unwrap())
            .collect();

        let explosion_sounds = explosion::ASSETS.into_iter()
            .map(|b| config.load_chunk(b).unwrap())
            .collect();

        Ok(Self { rng: thread_rng(), create_sounds, destroy_sounds, explosion_sounds })
    }

    pub fn play_create(&self, name: &str) {
        if let Some(chunk) = self.create_sounds.get(name) {
            chunk.try_play();
        }
    }

    pub fn play_destroy(&mut self) {
        self.destroy_sounds.choose(&mut self.rng).unwrap().try_play();
    }

    pub fn play_explosion(&mut self) {
        self.explosion_sounds.choose(&mut self.rng).unwrap().try_play();
    }

    pub fn play_music(&mut self) -> Result<(), String> {
        Music::unhook_finished();
        let mut queue = music::ASSETS
            .choose_multiple(&mut self.rng, music::ASSETS.len())
            .into_iter()
            .map(|&a| Music::from_static_bytes(a).unwrap())
            .collect::<VecDeque<Music<'static>>>();
        let next_music = queue.pop_front().unwrap();
        next_music.play(1)?;
        queue.push_back(next_music);
        unsafe {
            MUSIC_QUEUE = Some(Rc::new(RefCell::new(queue)));
        }
        Music::hook_finished(Self::play_next);
        Ok(())
    }

    pub fn halt_music(&mut self) {
        Music::unhook_finished();
        unsafe {
            MUSIC_QUEUE = None;
        }
    }

    fn play_next() {
        unsafe {
            if let Some(queue) = MUSIC_QUEUE.as_ref() {
                let mut queue = queue.borrow_mut();
                let next_music = queue.pop_front().unwrap();
                next_music.play(1).unwrap();
                queue.push_back(next_music);
            }
        }
    }
}

pub trait LoadSound {
    fn load_chunk(&self, buffer: &[u8]) -> Result<Chunk, String>;
}

impl LoadSound for AudioConfig {
    fn load_chunk(&self, buffer: &[u8]) -> Result<Chunk, String> {
        let raw = unsafe { mixer::Mix_LoadWAV_RW(RWops::from_bytes(buffer)?.raw(), 0) };
        if raw.is_null() {
            Err(get_error())
        } else {
            let mut chunk = Chunk { raw, owned: true };
            chunk.set_volume(self.effects_volume());
            Ok(chunk)
        }
    }
}