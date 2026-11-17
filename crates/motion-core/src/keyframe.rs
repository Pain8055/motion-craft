use crate::bezier::CubicBezier;
use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

/// Type of interpolation between keyframes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum KeyframeInterpolation {
    #[default]
    Linear,
    Hold,
    Bezier(CubicBezier),
}

/// A single keyframe at a specific time in seconds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
    pub interpolation: KeyframeInterpolation,
}

impl<T> Keyframe<T> {
    pub fn new(time: f32, value: T) -> Self {
        Self {
            time,
            value,
            interpolation: KeyframeInterpolation::Linear,
        }
    }

    pub fn with_easing(time: f32, value: T, bezier: CubicBezier) -> Self {
        Self {
            time,
            value,
            interpolation: KeyframeInterpolation::Bezier(bezier),
        }
    }
}

/// An animatable property track holding sorted keyframes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyTrack<T> {
    pub base_value: T,
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T: Clone> PropertyTrack<T> {
    pub fn new(base_value: T) -> Self {
        Self {
            base_value,
            keyframes: Vec::new(),
        }
    }

    pub fn is_animated(&self) -> bool {
        !self.keyframes.is_empty()
    }

    pub fn add_keyframe(&mut self, time: f32, value: T, interpolation: KeyframeInterpolation) {
        if let Some(pos) = self.keyframes.iter().position(|k| (k.time - time).abs() < 1e-4) {
            self.keyframes[pos].value = value;
            self.keyframes[pos].interpolation = interpolation;
        } else {
            self.keyframes.push(Keyframe {
                time,
                value,
                interpolation,
            });
            self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
    }

    pub fn remove_keyframe_at(&mut self, time: f32) -> bool {
        if let Some(pos) = self.keyframes.iter().position(|k| (k.time - time).abs() < 1e-4) {
            self.keyframes.remove(pos);
            true
        } else {
            false
        }
    }
}

pub trait Interpolatable: Clone {
    fn lerp_val(&self, other: &Self, t: f32) -> Self;
}

impl Interpolatable for f32 {
    fn lerp_val(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolatable for Vec2 {
    fn lerp_val(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl Interpolatable for Vec3 {
    fn lerp_val(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl Interpolatable for Vec4 {
    fn lerp_val(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl<T: Interpolatable> PropertyTrack<T> {
    pub fn sample(&self, time: f32) -> T {
        if self.keyframes.is_empty() {
            return self.base_value.clone();
        }

        if time <= self.keyframes.first().unwrap().time {
            return self.keyframes.first().unwrap().value.clone();
        }

        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().value.clone();
        }

        for i in 0..self.keyframes.len() - 1 {
            let k1 = &self.keyframes[i];
            let k2 = &self.keyframes[i + 1];

            if time >= k1.time && time <= k2.time {
                let duration = k2.time - k1.time;
                if duration <= 1e-5 {
                    return k2.value.clone();
                }

                let normalized_t = (time - k1.time) / duration;
                let eased_t = match k1.interpolation {
                    KeyframeInterpolation::Linear => normalized_t,
                    KeyframeInterpolation::Hold => 0.0,
                    KeyframeInterpolation::Bezier(bezier) => bezier.sample(normalized_t),
                };

                return k1.value.lerp_val(&k2.value, eased_t);
            }
        }

        self.base_value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_interpolation() {
        let mut track = PropertyTrack::new(0.0f32);
        track.add_keyframe(0.0, 100.0, KeyframeInterpolation::Linear);
        track.add_keyframe(1.0, 200.0, KeyframeInterpolation::Linear);

        assert!((track.sample(0.0) - 100.0).abs() < 1e-4);
        assert!((track.sample(0.5) - 150.0).abs() < 1e-4);
        assert!((track.sample(1.0) - 200.0).abs() < 1e-4);
    }
}
