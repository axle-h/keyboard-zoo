use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::time::Duration;
use crate::config::{GameInputConfig, InputConfig};

const AUTO_REPEAT_DELAY: Duration = Duration::from_millis(300);
const AUTO_REPEAT_ITERATION: Duration = Duration::from_millis(25);

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameInputKey {
    Up,
    Down,
    Left,
    Right,
    Spawn(char),
    SpawnRandom,
    Nuke,
    Quit,
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
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
        let mut map = match config.game {
            GameInputConfig::BabySmash => HashMap::from([
                (Keycode::A, GameInputKey::Spawn('A')),
                (Keycode::B, GameInputKey::Spawn('B')),
                (Keycode::C, GameInputKey::Spawn('C')),
                (Keycode::D, GameInputKey::Spawn('D')),
                (Keycode::E, GameInputKey::Spawn('E')),
                (Keycode::F, GameInputKey::Spawn('F')),
                (Keycode::G, GameInputKey::Spawn('G')),
                (Keycode::H, GameInputKey::Spawn('H')),
                (Keycode::I, GameInputKey::Spawn('I')),
                (Keycode::J, GameInputKey::Spawn('J')),
                (Keycode::K, GameInputKey::Spawn('K')),
                (Keycode::L, GameInputKey::Spawn('L')),
                (Keycode::M, GameInputKey::Spawn('M')),
                (Keycode::N, GameInputKey::Spawn('N')),
                (Keycode::O, GameInputKey::Spawn('O')),
                (Keycode::P, GameInputKey::Spawn('P')),
                (Keycode::Q, GameInputKey::Spawn('Q')),
                (Keycode::R, GameInputKey::Spawn('R')),
                (Keycode::S, GameInputKey::Spawn('S')),
                (Keycode::T, GameInputKey::Spawn('T')),
                (Keycode::U, GameInputKey::Spawn('U')),
                (Keycode::V, GameInputKey::Spawn('V')),
                (Keycode::W, GameInputKey::Spawn('W')),
                (Keycode::X, GameInputKey::Spawn('X')),
                (Keycode::Y, GameInputKey::Spawn('Y')),
                (Keycode::Z, GameInputKey::Spawn('Z')),
                (Keycode::Num0, GameInputKey::Spawn('0')),
                (Keycode::Num1, GameInputKey::Spawn('1')),
                (Keycode::Num2, GameInputKey::Spawn('2')),
                (Keycode::Num3, GameInputKey::Spawn('3')),
                (Keycode::Num4, GameInputKey::Spawn('4')),
                (Keycode::Num5, GameInputKey::Spawn('5')),
                (Keycode::Num6, GameInputKey::Spawn('6')),
                (Keycode::Num7, GameInputKey::Spawn('7')),
                (Keycode::Num8, GameInputKey::Spawn('8')),
                (Keycode::Num9, GameInputKey::Spawn('9'))
            ]),
            GameInputConfig::Arcade(arcade) => HashMap::from([
                (arcade.spawn, GameInputKey::SpawnRandom)
            ])
        };

        map.insert(config.up, GameInputKey::Up);
        map.insert(config.down, GameInputKey::Down);
        map.insert(config.left, GameInputKey::Left);
        map.insert(config.right, GameInputKey::Right);
        map.insert(config.nuke, GameInputKey::Nuke);

        map
    }
}
