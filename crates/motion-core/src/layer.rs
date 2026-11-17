use crate::blend::BlendMode;
use crate::keyframe::PropertyTrack;
use glam::{Mat4, Vec2, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub anchor_point: PropertyTrack<Vec2>,
    pub position: PropertyTrack<Vec2>,
    pub scale: PropertyTrack<Vec2>,
    pub rotation: PropertyTrack<f32>, // in degrees
    pub opacity: PropertyTrack<f32>,  // in 0..100%
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            anchor_point: PropertyTrack::new(Vec2::ZERO),
            position: PropertyTrack::new(Vec2::new(960.0, 540.0)),
            scale: PropertyTrack::new(Vec2::new(100.0, 100.0)),
            rotation: PropertyTrack::new(0.0),
            opacity: PropertyTrack::new(100.0),
        }
    }
}

impl Transform {
    pub fn compute_matrix(&self, time: f32) -> Mat4 {
        let anchor = self.anchor_point.sample(time);
        let pos = self.position.sample(time);
        let scale = self.scale.sample(time) / 100.0;
        let rot = self.rotation.sample(time).to_radians();

        Mat4::from_translation(pos.extend(0.0))
            * Mat4::from_rotation_z(rot)
            * Mat4::from_scale(scale.extend(1.0))
            * Mat4::from_translation((-anchor).extend(0.0))
    }

    pub fn compute_opacity(&self, time: f32) -> f32 {
        (self.opacity.sample(time) / 100.0).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayerContent {
    Solid {
        color: Vec4,
        width: f32,
        height: f32,
    },
    Text {
        text: String,
        font_size: f32,
        color: Vec4,
    },
    Image {
        path: String,
        width: f32,
        height: f32,
    },
    Adjustment,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: u64,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub solo: bool,
    pub in_point: f32,
    pub out_point: f32,
    pub blend_mode: BlendMode,
    pub parent_id: Option<u64>,
    pub transform: Transform,
    pub content: LayerContent,
}

impl Layer {
    pub fn new_solid(id: u64, name: &str, color: Vec4, width: f32, height: f32, duration: f32) -> Self {
        let mut transform = Transform::default();
        transform.anchor_point = PropertyTrack::new(Vec2::new(width * 0.5, height * 0.5));
        Self {
            id,
            name: name.to_string(),
            visible: true,
            locked: false,
            solo: false,
            in_point: 0.0,
            out_point: duration,
            blend_mode: BlendMode::Normal,
            parent_id: None,
            transform,
            content: LayerContent::Solid { color, width, height },
        }
    }

    pub fn new_text(id: u64, name: &str, text: &str, font_size: f32, color: Vec4, duration: f32) -> Self {
        Self {
            id,
            name: name.to_string(),
            visible: true,
            locked: false,
            solo: false,
            in_point: 0.0,
            out_point: duration,
            blend_mode: BlendMode::Normal,
            parent_id: None,
            transform: Transform::default(),
            content: LayerContent::Text {
                text: text.to_string(),
                font_size,
                color,
            },
        }
    }

    pub fn is_active_at(&self, time: f32) -> bool {
        self.visible && time >= self.in_point && time <= self.out_point
    }
}
