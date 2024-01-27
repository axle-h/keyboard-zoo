use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use sdl2::render::WindowCanvas;
use crate::characters::CharacterType;
use crate::assets::geometry::SpriteAsset;
use crate::config::PhysicsConfig;
use crate::game::action::{Direction, PhysicsAction};
use crate::game::event::GameEvent;
use crate::game::Game;
use crate::game::physics::{Body, Physics};
use crate::game::scale::PhysicsScale;


pub struct DefaultGame {
    events: Vec<GameEvent>,
    physics: Physics
}

impl DefaultGame {
    pub fn new<C: Into<Option<Rc<RefCell<WindowCanvas>>>>>(scale: PhysicsScale, physics_config: PhysicsConfig, canvas: C) -> Self {
        let mut physics = Physics::new(scale, physics_config);
        if let Some(canvas) = canvas.into() {
            physics.set_sdl_debug_draw(canvas);
        }
        Self { events: vec![], physics }
    }
}

impl Game for DefaultGame {
    fn push(&mut self, direction: Direction) {
        for event in self.physics.action(PhysicsAction::Push(direction)) {
            self.events.push(event);
        }
    }

    fn spawn_asset(&mut self, sprite: SpriteAsset) {
        for event in self.physics.spawn_asset(sprite).into_iter() {
            self.events.push(event);
        }
    }

    fn spawn_character(&mut self, character: CharacterType) {
        for event in self.physics.spawn_character(character).into_iter() {
            self.events.push(event);
        }
    }

    fn destroy(&mut self, id: u128) {
        if let Some(event) = self.physics.destroy_body(id) {
            self.events.push(event);
        }
    }

    fn explosion(&mut self) {
        for event in self.physics.action(PhysicsAction::Explode) {
            self.events.push(event);
        }
    }

    fn update(&mut self, delta: Duration) -> Vec<GameEvent> {
        let mut buffer = vec![];
        for event in self.physics.update(delta).into_iter() {
            buffer.push(event);
        }
        while let Some(event) = self.events.pop() {
            buffer.push(event);
        }
        buffer
    }

    fn bodies(&self) -> Vec<Body> {
        self.physics.bodies()
    }

    fn debug_draw(&self) {
        self.physics.debug_draw();
    }
}
