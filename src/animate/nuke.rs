use std::time::Duration;
use super::event::AnimationEvent;

const NUKE_DELAY: f64 = 0.1;

#[derive(Debug, Clone)]
pub struct NukeAnimationState {
    delay: Duration,
    duration: Duration,
    queue: Vec<u128>
}

impl NukeAnimationState {
    pub fn new(queue: Vec<u128>) -> Self {
        let delay = Duration::from_secs_f64(NUKE_DELAY);
        Self {
            delay,
            duration: delay, // destroy first body right away
            queue
        }
    }

    fn update<F : FnMut(u128)>(&mut self, delta: Duration, mut callback: F) {
        if self.is_finished() {
            return;
        }
        self.duration += delta;
        while self.duration >= self.delay {
            self.duration -= self.delay;
            if let Some(id) = self.queue.pop() {
                callback(id)
            } else {
                return;
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.queue.is_empty()
    }

}

pub struct NukeAnimation {
    state: Option<NukeAnimationState>
}

impl NukeAnimation {
    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn nuke(&mut self, ids: Vec<u128>) {
        if let Some(state) = self.state.as_mut() {
            // merge
            state.queue = state.queue.iter().copied().chain(ids).collect::<Vec<u128>>();
            state.queue.sort();
            state.queue.dedup();
        } else {
            self.state = Some(NukeAnimationState::new(ids));
        }
    }

    pub fn update<F : FnMut(AnimationEvent)>(&mut self, delta: Duration, mut callback: F) {
        if let Some(state) = self.state.as_mut() {
            state.update(delta, |id| callback(AnimationEvent::DestroyAsset { id }));
            if state.is_finished() {
                self.state = None;
            }
        }
    }
}