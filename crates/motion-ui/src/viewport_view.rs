use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use motion_core::Composition;

pub fn render_viewport(
    ui: &mut Ui,
    comp: &Composition,
    _selected_layer_id: Option<u64>,
    _time: f32,
) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = response.rect;

    // Viewport dark background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(18, 19, 22));

    // Calculate composition scale to fit viewport with margin
    let margin = 20.0;
    let available_w = (rect.width() - margin * 2.0).max(10.0);
    let available_h = (rect.height() - margin * 2.0).max(10.0);

    let scale_x = available_w / comp.width as f32;
    let scale_y = available_h / comp.height as f32;
    let fit_scale = scale_x.min(scale_y);

    let comp_w = comp.width as f32 * fit_scale;
    let comp_h = comp.height as f32 * fit_scale;

    let comp_rect = Rect::from_center_size(rect.center(), Vec2::new(comp_w, comp_h));

    // Draw Composition Canvas Bounds & Checkerboard
    painter.rect_filled(comp_rect, 0.0, Color32::from_rgb(0, 0, 0));
    painter.rect_stroke(comp_rect, 0.0, Stroke::new(1.5, Color32::from_rgb(100, 105, 120)));

    // Viewport Overlay Info
    painter.text(
        Pos2::new(rect.left() + 10.0, rect.top() + 10.0),
        egui::Align2::LEFT_TOP,
        format!("{} ({}x{}) @ {:.0}fps", comp.name, comp.width, comp.height, comp.fps),
        egui::FontId::proportional(12.0),
        Color32::from_rgb(180, 185, 195),
    );
}
