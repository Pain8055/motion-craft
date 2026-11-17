use crate::layer::Layer;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Composition {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration: f32, // in seconds
    pub layers: Vec<Layer>,
    next_layer_id: u64,
}

impl Default for Composition {
    fn default() -> Self {
        Self::new("Composition 1", 1920, 1080, 60.0, 5.0)
    }
}

impl Composition {
    pub fn new(name: &str, width: u32, height: u32, fps: f32, duration: f32) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            fps,
            duration,
            layers: Vec::new(),
            next_layer_id: 1,
        }
    }

    pub fn total_frames(&self) -> u32 {
        (self.duration * self.fps).round() as u32
    }

    pub fn frame_to_time(&self, frame: u32) -> f32 {
        frame as f32 / self.fps
    }

    pub fn time_to_frame(&self, time: f32) -> u32 {
        (time * self.fps).round() as u32
    }

    pub fn add_layer(&mut self, mut layer: Layer) -> u64 {
        let id = self.next_layer_id;
        self.next_layer_id += 1;
        layer.id = id;
        self.layers.insert(0, layer); // top layer first
        id
    }

    pub fn remove_layer(&mut self, id: u64) -> Option<Layer> {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            Some(self.layers.remove(pos))
        } else {
            None
        }
    }

    pub fn get_layer(&self, id: u64) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn get_layer_mut(&mut self, id: u64) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Computes the final world transformation matrix for a layer, taking parenting into account.
    pub fn compute_world_matrix(&self, layer_id: u64, time: f32) -> glam::Mat4 {
        if let Some(layer) = self.get_layer(layer_id) {
            let local_mat = layer.transform.compute_matrix(time);
            if let Some(parent_id) = layer.parent_id {
                let parent_mat = self.compute_world_matrix(parent_id, time);
                parent_mat * local_mat
            } else {
                local_mat
            }
        } else {
            glam::Mat4::IDENTITY
        }
    }
}
