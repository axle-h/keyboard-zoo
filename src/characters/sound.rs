use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use sdl2::mixer::{Channel, Chunk, Music, set_channel_finished};
use crate::assets::sound::LoadSound;
use crate::assets::sound::playable::Playable;
use crate::characters::CharacterType;
use crate::config::AudioConfig;

#[derive(Debug, Clone)]
pub struct CharacterSoundData {
    character: CharacterType,
    create: &'static [u8],
    destroy: &'static [u8],
    attack: &'static [u8],
}

impl CharacterSoundData {
    pub fn new(character: CharacterType, create: &'static [u8], destroy: &'static [u8], attack: &'static [u8]) -> Self {
        Self { character, create, destroy, attack }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum CharacterSoundType { Create, Destroy, Attack }

struct CharacterSoundEntry {
    sample: Chunk,
    is_playing: bool
}

impl CharacterSoundEntry {
    fn new(config: &AudioConfig, sample: &'static [u8]) -> Result<Rc<RefCell<Self>>, String> {
        let entry = Self { sample: config.load_chunk(sample)?, is_playing: false };
        Ok(Rc::new(RefCell::new(entry)))
    }
}


static mut SOUND_MAP: Option<HashMap<i32, Rc<RefCell<CharacterSoundEntry>>>> = None;

pub struct CharacterSound {
    sound: HashMap<(CharacterType, CharacterSoundType), Rc<RefCell<CharacterSoundEntry>>>
}

impl CharacterSound {
    pub fn new(config: AudioConfig, data: &[CharacterSoundData]) -> Result<Self, String> {
        let mut sound = HashMap::new();
        for data in data {
            sound.insert(
                (data.character, CharacterSoundType::Create),
                CharacterSoundEntry::new(&config, data.create)?
            );
            sound.insert(
                (data.character, CharacterSoundType::Destroy),
                CharacterSoundEntry::new(&config, data.destroy)?
            );
            sound.insert(
                (data.character, CharacterSoundType::Attack),
                CharacterSoundEntry::new(&config, data.attack)?
            );
        }
        unsafe {
            SOUND_MAP = Some(HashMap::new());
        }
        set_channel_finished(Self::channel_callback);
        Ok(Self { sound })
    }

    fn play(&mut self, character: CharacterType, sound_type: CharacterSoundType) -> Result<(), String> {
        if let Some(sound_ptr) = self.sound.get_mut(&(character, sound_type)) {
            sound_ptr.borrow_mut().sample.play().map(|_| ())
        } else {
            Ok(())
        }
    }

    fn play_debounced(&mut self, character: CharacterType, sound_type: CharacterSoundType) -> Result<(), String> {
        if let Some(sound_ptr) = self.sound.get_mut(&(character, sound_type)) {
            let channel = {
                let mut sound = sound_ptr.borrow_mut();
                let Channel(channel) = sound.sample.play()?;
                if !sound.is_playing {
                    let Channel(channel) = sound.sample.play()?;
                    sound.is_playing = true;
                    Some(channel)
                } else {
                    None
                }
            };
            if let Some(channel) = channel {
                unsafe {
                    SOUND_MAP.as_mut().unwrap().insert(channel, sound_ptr.clone());
                }
            }
        }
        Ok(())
    }

    pub fn play_create(&mut self, character: CharacterType) -> Result<(), String> {
        self.play(character, CharacterSoundType::Create)
    }

    pub fn play_destroy(&mut self, character: CharacterType) -> Result<(), String> {
        self.play(character, CharacterSoundType::Destroy)
    }

    pub fn play_attack(&mut self, character: CharacterType) -> Result<(), String> {
        self.play_debounced(character, CharacterSoundType::Attack)
    }

    fn channel_callback(channel: Channel) {
        unsafe {
            if let Some(sound_map) = SOUND_MAP.as_mut() {
                let Channel(channel) = channel;
                if let Some(sound) = sound_map.get_mut(&channel) {
                    sound.borrow_mut().is_playing = false;
                    sound_map.remove(&channel);
                }
            }
        }

    }
}