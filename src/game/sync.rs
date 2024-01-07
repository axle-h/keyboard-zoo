use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::time::Duration;
use crate::assets::geometry::SpriteAsset;
use crate::config::PhysicsConfig;
use crate::frame_rate::FrameRate;
use crate::game::action::Direction;
use crate::game::default::DefaultGame;
use crate::game::event::GameEvent;
use crate::game::Game;
use crate::game::physics::Body;
use crate::game::scale::WorldScale;

const UPDATE_FREQ: f64 = 1.0 / 60.0;

#[derive(Debug, Clone)]
enum GameSyncCommand {
    Quit,
    Push(Direction),
    Spawn(SpriteAsset),
    Destroy(u128),
}

#[derive(Debug, Clone)]
enum GameSyncEvent {
    GameEvent(GameEvent),
    Bodies(Vec<Body>)
}

pub struct AsyncGame {
    event_rx: Receiver<GameSyncEvent>,
    command_tx: Sender<GameSyncCommand>,
    latest_bodies: Vec<Body>
}

impl AsyncGame {
    pub fn new(scale: WorldScale, physics_config: PhysicsConfig) -> Self {
        let (event_tx, event_rx) = channel();
        let (command_tx, command_rx) = channel();
        let sync = AsyncGame { event_rx, command_tx, latest_bodies: vec![] };

        thread::spawn(move|| {
            let game = DefaultGame::new(scale, physics_config, None);
            let mut thread = GameThread { event_tx, command_rx, game };
            let mut frame_rate = FrameRate::new();
            let update_freq = Duration::from_secs_f64(UPDATE_FREQ);
            loop {
                if thread.iteration(update_freq).is_err() {
                    break;
                }
                let delta = frame_rate.update().unwrap();
                if let Some(block_for) = update_freq.checked_sub(delta) {
                    thread::sleep(block_for);
                }
            }
        });
        sync
    }
}

impl Game for AsyncGame {
    fn push(&mut self, direction: Direction) {
        self.command_tx.send(GameSyncCommand::Push(direction)).unwrap();
    }

    fn spawn(&mut self, sprite: SpriteAsset) {
        self.command_tx.send(GameSyncCommand::Spawn(sprite)).unwrap();
    }

    fn destroy(&mut self, id: u128) {
        self.command_tx.send(GameSyncCommand::Destroy(id)).unwrap();
    }

    fn update(&mut self, delta: Duration) -> Vec<GameEvent> {
        let mut buffer = vec![];
        while let Ok(sync_event) = self.event_rx.try_recv() {
            match sync_event {
                GameSyncEvent::GameEvent(event) => buffer.push(event),
                GameSyncEvent::Bodies(bodies) => self.latest_bodies = bodies,
            }
        }
        buffer
    }

    fn bodies(&self) -> Vec<Body> {
        // TODO move bodies out of events so this call is not dependent on consuming events
        self.latest_bodies.clone()
    }

    fn debug_draw(&self) {
        // do nothing, not supported
    }
}

struct GameThread {
    event_tx: Sender<GameSyncEvent>,
    command_rx: Receiver<GameSyncCommand>,
    game: DefaultGame,
}

impl GameThread {
    fn iteration(&mut self, delta: Duration) -> Result<(), String> {
        while let Ok(command) = self.command_rx.try_recv() {
            match command {
                GameSyncCommand::Quit => return Err("received quit command".to_string()),
                GameSyncCommand::Push(direction) => self.game.push(direction),
                GameSyncCommand::Spawn(asset) => self.game.spawn(asset),
                GameSyncCommand::Destroy(id) => self.game.destroy(id),
            }
        }
        for event in self.game.update(delta).into_iter() {
            self.event_tx.send(GameSyncEvent::GameEvent(event))
                .map_err(|e| e.to_string())?;
        }
        self.event_tx.send(GameSyncEvent::Bodies(self.game.bodies()))
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}