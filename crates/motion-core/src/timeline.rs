use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Playing,
    Paused,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timeline {
    pub current_time: f32,
    pub state: PlaybackState,
    pub is_looping: bool,
    pub zoom: f32, // pixels per second in the UI
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            state: PlaybackState::Paused,
            is_looping: true,
            zoom: 120.0,
        }
    }
}

impl Timeline {
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    pub fn toggle_play(&mut self) {
        self.state = match self.state {
            PlaybackState::Playing => PlaybackState::Paused,
            PlaybackState::Paused => PlaybackState::Playing,
        };
    }

    pub fn seek(&mut self, time: f32, max_duration: f32) {
        self.current_time = time.clamp(0.0, max_duration);
    }

    pub fn advance(&mut self, delta_seconds: f32, max_duration: f32) {
        if self.state == PlaybackState::Playing {
            self.current_time += delta_seconds;
            if self.current_time >= max_duration {
                if self.is_looping {
                    self.current_time %= max_duration.max(1e-4);
                } else {
                    self.current_time = max_duration;
                    self.state = PlaybackState::Paused;
                }
            }
        }
    }
}
