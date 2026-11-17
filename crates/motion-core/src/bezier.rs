use serde::{Deserialize, Serialize};

/// Cubic Bezier control points for easing curves (analogous to After Effects Graph Editor).
/// P0 is always (0, 0) and P3 is always (1, 1).
/// P1 = (x1, y1) and P2 = (x2, y2).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Default for CubicBezier {
    fn default() -> Self {
        Self::linear()
    }
}

impl CubicBezier {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub const fn linear() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    pub const fn ease() -> Self {
        Self::new(0.25, 0.1, 0.25, 1.0)
    }

    pub const fn ease_in() -> Self {
        Self::new(0.42, 0.0, 1.0, 1.0)
    }

    pub const fn ease_out() -> Self {
        Self::new(0.0, 0.0, 0.58, 1.0)
    }

    pub const fn ease_in_out() -> Self {
        Self::new(0.42, 0.0, 0.58, 1.0)
    }

    /// Evaluates the Y value for a given X progress in [0, 1] using Newton-Raphson approximation.
    pub fn sample(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // Fast solve for parametric parameter u such that BezierX(u) == t
        let mut u = t;
        for _ in 0..8 {
            let x = self.calc_bezier(u, self.x1, self.x2);
            let dx = self.calc_bezier_derivative(u, self.x1, self.x2);
            if dx.abs() < 1e-6 {
                break;
            }
            let diff = x - t;
            u -= diff / dx;
            u = u.clamp(0.0, 1.0);
            if diff.abs() < 1e-5 {
                break;
            }
        }

        self.calc_bezier(u, self.y1, self.y2)
    }

    #[inline]
    fn calc_bezier(&self, u: f32, p1: f32, p2: f32) -> f32 {
        let inv = 1.0 - u;
        3.0 * inv * inv * u * p1 + 3.0 * inv * u * u * p2 + u * u * u
    }

    #[inline]
    fn calc_bezier_derivative(&self, u: f32, p1: f32, p2: f32) -> f32 {
        let inv = 1.0 - u;
        3.0 * inv * inv * p1 + 6.0 * inv * u * (p2 - p1) + 3.0 * u * u * (1.0 - p2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_linear() {
        let curve = CubicBezier::linear();
        assert!((curve.sample(0.0) - 0.0).abs() < 1e-4);
        assert!((curve.sample(0.5) - 0.5).abs() < 1e-4);
        assert!((curve.sample(1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_bezier_ease_in_out() {
        let curve = CubicBezier::ease_in_out();
        assert!((curve.sample(0.0) - 0.0).abs() < 1e-4);
        assert!((curve.sample(0.5) - 0.5).abs() < 1e-4);
        assert!((curve.sample(1.0) - 1.0).abs() < 1e-4);
    }
}
