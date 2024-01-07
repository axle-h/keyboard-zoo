use crate::game::physics::Body;

#[derive(Clone, Debug)]
pub enum GameEvent {
    Spawned(Body),
    Destroy(Body)
}

// TODO not needed
#[derive(Clone, Debug)]
pub enum PhysicsEvent {
    Spawned(Body),
    Destroy(Body)
}

impl Into<GameEvent> for PhysicsEvent {
    fn into(self) -> GameEvent {
        match self {
            PhysicsEvent::Spawned(body) => GameEvent::Spawned(body),
            PhysicsEvent::Destroy(body) => GameEvent::Destroy(body)
        }
    }
}