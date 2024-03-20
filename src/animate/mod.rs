use std::time::Duration;
use self::event::AnimationEvent;
use self::nuke::NukeAnimation;

mod nuke;
pub mod event;

pub struct Animations {
    nuke: NukeAnimation
}

impl Animations {
    pub fn new() -> Self {
        Self { nuke: NukeAnimation::new() }
    }

    pub fn update(&mut self, delta: Duration) -> Vec<AnimationEvent> {
        let mut events = vec![];
        self.nuke.update(delta, |e| events.push(e));
        events
    }

    pub fn nuke(&mut self, ids: Vec<u128>) {
        self.nuke.nuke(ids);
    }
}