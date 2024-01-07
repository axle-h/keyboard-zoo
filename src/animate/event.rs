#[derive(Debug, Copy, Clone)]
pub enum AnimationEvent {
    DestroyAsset { id: u128 }
}