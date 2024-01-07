#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up, Down, Left, Right
}

#[derive(Debug, Clone, Copy)]
pub enum PhysicsAction {
    Push(Direction)
}