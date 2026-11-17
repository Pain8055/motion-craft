use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    GaussianBlur {
        radius: f32,
    },
    Glow {
        threshold: f32,
        intensity: f32,
        radius: f32,
    },
    ColorBalance {
        brightness: f32,
        contrast: f32,
        saturation: f32,
    },
    ChromaticAberration {
        offset: f32,
    },
    Vignette {
        amount: f32,
        roundness: f32,
    },
}

impl Effect {
    pub fn name(&self) -> &'static str {
        match self {
            Self::GaussianBlur { .. } => "Gaussian Blur",
            Self::Glow { .. } => "Glow",
            Self::ColorBalance { .. } => "Color Balance",
            Self::ChromaticAberration { .. } => "Chromatic Aberration",
            Self::Vignette { .. } => "Vignette",
        }
    }
}
