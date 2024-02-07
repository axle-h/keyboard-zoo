use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use sdl2::get_error;
use sdl2::mixer::{Channel, Chunk, Music, reserve_channels};
use sdl2::rwops::RWops;
use sdl2::sys::mixer;
use crate::assets::sound::playable::Playable;
use crate::config::AudioConfig;
use crate::random::BagRandom;

pub mod playable;
mod music;
mod effects;

static mut MUSIC_QUEUE: Option<Rc<RefCell<VecDeque<Music<'static>>>>> = None;

const COLLISION_CHANNELS: usize = 5;

pub struct Sound {
    alphanumeric: HashMap<char, Chunk>,
    destroy: BagRandom<Chunk>,
    explosion: BagRandom<Chunk>,
    collision: BagRandom<Chunk>,
    collision_channels: [Channel; COLLISION_CHANNELS]
}

impl Sound {
    fn load_sounds(config: &AudioConfig, assets: &[&'static [u8]]) -> BagRandom<Chunk> {
        BagRandom::new(
            assets.into_iter()
                .map(|b| config.load_chunk(b).unwrap())
                .collect()
        )
    }

    pub fn new(config: AudioConfig) -> Result<Self, String> {
        let alphanumeric = effects::alphanumeric::alphanumeric_sounds(&config);
        let destroy = Self::load_sounds(&config, &effects::destroy::ASSETS);
        let explosion = Self::load_sounds(&config, &effects::explosion::ASSETS);
        let collision = Self::load_sounds(&config, &effects::collision::ASSETS);

        reserve_channels(COLLISION_CHANNELS as i32);
        let collision_channels = (0..COLLISION_CHANNELS)
            .map(|i| Channel(i as i32))
            .collect::<Vec<Channel>>()
            .try_into()
            .unwrap();

        Ok(Self { alphanumeric, destroy, explosion, collision, collision_channels })
    }

    pub fn play_alphanumeric(&self, ch: char) {
        if let Some(chunk) = self.alphanumeric.get(&ch) {
            chunk.try_play();
        }
    }

    pub fn play_destroy(&mut self) {
        self.destroy.next().unwrap().try_play();
    }

    pub fn play_explosion(&mut self) {
        self.explosion.next().unwrap().try_play();
    }

    pub fn play_collision(&mut self) {
        for channel in self.collision_channels.iter() {
            if !channel.is_playing() {
                channel.play(self.collision.next().unwrap().as_ref(), 0).unwrap();
            }
        }
    }

    pub fn play_music(&mut self) -> Result<(), String> {
        Music::unhook_finished();

        let mut rng = thread_rng();
        let mut queue = music::ASSETS
            .choose_multiple(&mut rng, music::ASSETS.len())
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