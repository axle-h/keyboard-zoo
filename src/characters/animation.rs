use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum SpriteAnimationType {
    Static,
    Linear {
        frames: usize,
        duration: Duration,
    },
    YoYo {
        frames: usize,
        duration: Duration,
    },
    LinearWithPause {
        frames: usize,
        duration: Duration,
        pause_for: Duration,
        resume_from_frame: usize,
    },
}

impl SpriteAnimationType {
    pub fn into_animation(self) -> SpriteAnimation {
        SpriteAnimation::new(self)
    }

    pub fn max_frame(&self) -> usize {
        match self {
            SpriteAnimationType::Static => 0,
            SpriteAnimationType::Linear { frames, .. } => *frames,
            SpriteAnimationType::YoYo { frames, .. } => *frames,
            SpriteAnimationType::LinearWithPause { frames, .. } => *frames,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpriteAnimation {
    animation_type: SpriteAnimationType,
    duration: Duration,
    paused_for: Option<Duration>,
    frame: usize,
    invert: bool,
    iteration: usize,
    max_frame: usize,
}

impl SpriteAnimation {
    pub fn new(animation_type: SpriteAnimationType) -> Self {
        Self {
            animation_type,
            duration: Duration::ZERO,
            paused_for: None,
            frame: 0,
            invert: false,
            iteration: 0,
            max_frame: animation_type.max_frame(),
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;
        match self.animation_type {
            SpriteAnimationType::Static => {
                self.frame = 0;
                self.iteration = 0;
            }
            SpriteAnimationType::Linear {
                duration: frame_duration,
                ..
            } => {
                self.next_linear(frame_duration, false);
            }
            SpriteAnimationType::YoYo {
                duration: frame_duration,
                ..
            } => {
                self.next_linear(frame_duration, true);
            }
            SpriteAnimationType::LinearWithPause {
                duration: frame_duration,
                pause_for,
                resume_from_frame,
                ..
            } => {
                if let Some(paused_for) = self.paused_for {
                    // maybe unpause
                    self.paused_for = paused_for.checked_sub(delta);
                    if self.paused_for == Some(Duration::ZERO) {
                        self.paused_for = None;
                    }
                    if self.paused_for.is_none() {
                        // unpause
                        self.duration = Duration::ZERO;
                        self.iteration += 1;
                        self.frame = resume_from_frame;
                    }
                } else {
                    self.register_frames(frame_duration);
                    if self.frame >= self.max_frame {
                        // pause
                        self.frame = self.max_frame - 1;
                        self.paused_for = Some(pause_for);
                    }
                }
            }
        }
    }

    fn register_frames(&mut self, frame_duration: Duration) {
        while let Some(remainder) = self.duration.checked_sub(frame_duration) {
            self.duration = remainder;
            self.frame += 1;
        }
    }

    fn next_linear(&mut self, frame_duration: Duration, invert: bool) {
        self.register_frames(frame_duration);
        if self.frame >= self.max_frame {
            self.iteration += 1;
            self.frame %= self.max_frame;
            if invert {
                self.invert = !self.invert;
            }
        }
    }

    pub fn reset(&mut self) {
        self.duration = Duration::ZERO;
        self.frame = 0;
        self.invert = false;
    }

    pub fn frame(&self) -> usize {
        if self.invert {
            self.max_frame - self.frame - 1
        } else {
            self.frame
        }
    }

    pub fn iteration(&self) -> usize {
        self.iteration
    }
}
