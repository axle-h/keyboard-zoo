use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::time::Duration;
use crate::characters::CharacterType;
use crate::config::InputConfig;

const AUTO_REPEAT_DELAY: Duration = Duration::from_millis(300);
const AUTO_REPEAT_ITERATION: Duration = Duration::from_millis(25);

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameInputKey {
    Up,
    Down,
    Left,
    Right,
    SpawnAsset(char),
    SpawnRandomAsset,
    SpawnCharacter(CharacterType),
    SpawnRandomCharacter,
    Nuke,
    Quit,
    Explosion,
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
struct GameInput {
    key: GameInputKey,
    duration: Duration,
    repeating: bool,
}

impl GameInput {
    fn new(key: GameInputKey) -> Self {
        Self {
            key,
            duration: Duration::ZERO,
            repeating: false,
        }
    }
}

enum KeyState {
    Down(GameInputKey),
    Up(GameInputKey),
}

pub struct GameInputContext {
    current: HashMap<GameInputKey, GameInput>,
    input_map: HashMap<Keycode, GameInputKey>
}

impl GameInputContext {
    pub fn new(config: InputConfig) -> Self {
        Self {
            current: HashMap::new(),
            input_map: Self::input_map(config)
        }
    }

    pub fn update<I>(&mut self, delta: Duration, sdl_events: I) -> Vec<GameInputKey>
    where
        I: Iterator<Item = Event>,
    {
        let mut result: Vec<GameInputKey> = vec![];

        // update any keys that might still be held with the delta
        for event in self.current.values_mut() {
            event.duration += delta;
        }

        for sdl_event in sdl_events {
            if let Some(key_state) = self.map_from_sdl_event(sdl_event) {
                match key_state {
                    KeyState::Down(key) => {
                        let event = GameInput::new(key);
                        self.current.insert(key, event);
                        result.push(key);
                    }
                    KeyState::Up(key) => {
                        self.current.remove(&key);
                    }
                }
            }
        }

        // check for any held keys that have triggered a repeat
        for event in self.current.values_mut() {
            if matches!(event.key, GameInputKey::Up | GameInputKey::Down | GameInputKey::Left | GameInputKey::Right) {
                // check auto-repeat
                if event.repeating {
                    if event.duration >= AUTO_REPEAT_ITERATION {
                        event.duration = Duration::ZERO;
                        result.push(event.key);
                    }
                } else if event.duration >= AUTO_REPEAT_DELAY {
                    event.duration = Duration::ZERO;
                    event.repeating = true;
                    result.push(event.key);
                }
            }
        }

        result
    }

    fn map_from_sdl_event(&self, event: Event) -> Option<KeyState> {
        match event {
            Event::Quit { .. } => Some(KeyState::Down(GameInputKey::Quit)),
            Event::KeyDown {
                keycode: Some(keycode),
                repeat: false,
                ..


            } => self.input_map.get(&keycode).map(|&k| KeyState::Down(k)),
            Event::KeyUp {
                keycode: Some(keycode),
                repeat: false,
                ..
            } => self.input_map.get(&keycode).map(|&k| KeyState::Up(k)),
            _ => None,
        }
    }

    fn input_map(config: InputConfig) -> HashMap<Keycode, GameInputKey> {
        let mut map = if config.baby_smash_mode {
            HashMap::from([
                (Keycode::A, GameInputKey::SpawnAsset('A')),
                (Keycode::B, GameInputKey::SpawnAsset('B')),
                (Keycode::C, GameInputKey::SpawnAsset('C')),
                (Keycode::D, GameInputKey::SpawnAsset('D')),
                (Keycode::E, GameInputKey::SpawnAsset('E')),
                (Keycode::F, GameInputKey::SpawnAsset('F')),
                (Keycode::G, GameInputKey::SpawnAsset('G')),
                (Keycode::H, GameInputKey::SpawnAsset('H')),
                (Keycode::I, GameInputKey::SpawnAsset('I')),
                (Keycode::J, GameInputKey::SpawnAsset('J')),
                (Keycode::K, GameInputKey::SpawnAsset('K')),
                (Keycode::L, GameInputKey::SpawnAsset('L')),
                (Keycode::M, GameInputKey::SpawnAsset('M')),
                (Keycode::N, GameInputKey::SpawnAsset('N')),
                (Keycode::O, GameInputKey::SpawnAsset('O')),
                (Keycode::P, GameInputKey::SpawnAsset('P')),
                (Keycode::Q, GameInputKey::SpawnAsset('Q')),
                (Keycode::R, GameInputKey::SpawnAsset('R')),
                (Keycode::S, GameInputKey::SpawnAsset('S')),
                (Keycode::T, GameInputKey::SpawnAsset('T')),
                (Keycode::U, GameInputKey::SpawnAsset('U')),
                (Keycode::V, GameInputKey::SpawnAsset('V')),
                (Keycode::W, GameInputKey::SpawnAsset('W')),
                (Keycode::X, GameInputKey::SpawnAsset('X')),
                (Keycode::Y, GameInputKey::SpawnAsset('Y')),
                (Keycode::Z, GameInputKey::SpawnAsset('Z')),
                (Keycode::Num0, GameInputKey::SpawnAsset('0')),
                (Keycode::Num1, GameInputKey::SpawnAsset('1')),
                (Keycode::Num2, GameInputKey::SpawnAsset('2')),
                (Keycode::Num3, GameInputKey::SpawnAsset('3')),
                (Keycode::Num4, GameInputKey::SpawnAsset('4')),
                (Keycode::Num5, GameInputKey::SpawnAsset('5')),
                (Keycode::Num6, GameInputKey::SpawnAsset('6')),
                (Keycode::Num7, GameInputKey::SpawnAsset('7')),
                (Keycode::Num8, GameInputKey::SpawnAsset('8')),
                (Keycode::Num9, GameInputKey::SpawnAsset('9'))
            ])
        } else {
            HashMap::new()
        };

        map.insert(config.spawn_character, GameInputKey::SpawnRandomCharacter);
        map.insert(config.spawn_asset, GameInputKey::SpawnRandomAsset);
        map.insert(config.up, GameInputKey::Up);
        map.insert(config.down, GameInputKey::Down);
        map.insert(config.left, GameInputKey::Left);
        map.insert(config.right, GameInputKey::Right);
        map.insert(config.nuke, GameInputKey::Nuke);
        map.insert(config.explosion, GameInputKey::Explosion);

        map
    }
}
