use crate::characters::CharacterType;
use crate::game::physics::Body;

#[derive(Clone, Debug)]
pub enum GameEvent {
    Spawned(Body),
    Destroy(Body),
    Explosion { x: i32, y: i32 },
    CharacterAttack(CharacterType),
    HeavyCollision
}
