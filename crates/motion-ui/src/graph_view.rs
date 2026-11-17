use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use motion_core::CubicBezier;

pub fn render_graph_editor(ui: &mut Ui, bezier: &mut CubicBezier) {
    ui.heading("Graph Editor (Bezier Easing)");
    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Linear").clicked() {
            *bezier = CubicBezier::linear();
        }
        if ui.button("Ease In").clicked() {
            *bezier = CubicBezier::ease_in();
        }
        if ui.button("Ease Out").clicked() {
            *bezier = CubicBezier::ease_out();
        }
        if ui.button("Ease In-Out").clicked() {
            *bezier = CubicBezier::ease_in_out();
        }
    });

    let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 160.0), egui::Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 4.0, Color32::from_rgb(25, 27, 32));
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(50, 54, 64)));

    // Draw curve
    let steps = 40;
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let y_val = bezier.sample(t);
        let px = rect.left() + 20.0 + t * (rect.width() - 40.0);
        let py = rect.bottom() - 20.0 - y_val * (rect.height() - 40.0);
        points.push(Pos2::new(px, py));
    }

    for i in 0..points.len() - 1 {
        painter.line_segment([points[i], points[i + 1]], Stroke::new(2.5, Color32::from_rgb(60, 200, 255)));
    }
}
