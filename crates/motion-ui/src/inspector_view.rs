use egui::Ui;
use glam::Vec2;
use motion_core::{BlendMode, Composition, KeyframeInterpolation, Layer};

pub fn render_inspector(
    ui: &mut Ui,
    comp: &mut Composition,
    selected_layer_id: Option<u64>,
    current_time: f32,
) {
    ui.heading("Inspector");
    ui.separator();

    let layer = match selected_layer_id.and_then(|id| comp.get_layer_mut(id)) {
        Some(l) => l,
        None => {
            ui.label("No layer selected.\nTap or click a layer to view and animate properties.");
            return;
        }
    };

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut layer.name);
    });

    ui.horizontal(|ui| {
        ui.label("Blend Mode:");
        egui::ComboBox::from_id_salt("blend_mode_select")
            .selected_text(layer.blend_mode.label())
            .show_ui(ui, |ui| {
                for mode in BlendMode::ALL {
                    ui.selectable_value(&mut layer.blend_mode, mode, mode.label());
                }
            });
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut layer.visible, "Visible");
        ui.checkbox(&mut layer.locked, "Locked");
        ui.checkbox(&mut layer.solo, "Solo");
    });

    ui.separator();
    ui.label(egui::RichText::new("Transform").strong());

    // Position
    let mut pos = layer.transform.position.sample(current_time);
    let has_pos_kf = layer.transform.position.keyframes.iter().any(|k| (k.time - current_time).abs() < 1e-4);
    ui.horizontal(|ui| {
        let btn_text = if has_pos_kf { "◆" } else { "◇" };
        if ui.button(btn_text).on_hover_text("Add/Remove Position Keyframe").clicked() {
            if has_pos_kf {
                layer.transform.position.remove_keyframe_at(current_time);
            } else {
                layer.transform.position.add_keyframe(current_time, pos, KeyframeInterpolation::Linear);
            }
        }
        ui.label("Position:");
        let mut x = pos.x;
        let mut y = pos.y;
        if ui.add(egui::DragValue::new(&mut x).speed(1.0).prefix("X: ")).changed()
            || ui.add(egui::DragValue::new(&mut y).speed(1.0).prefix("Y: ")).changed()
        {
            pos = Vec2::new(x, y);
            if layer.transform.position.is_animated() {
                layer.transform.position.add_keyframe(current_time, pos, KeyframeInterpolation::Linear);
            } else {
                layer.transform.position.base_value = pos;
            }
        }
    });

    // Scale
    let mut scale = layer.transform.scale.sample(current_time);
    let has_scale_kf = layer.transform.scale.keyframes.iter().any(|k| (k.time - current_time).abs() < 1e-4);
    ui.horizontal(|ui| {
        let btn_text = if has_scale_kf { "◆" } else { "◇" };
        if ui.button(btn_text).on_hover_text("Add/Remove Scale Keyframe").clicked() {
            if has_scale_kf {
                layer.transform.scale.remove_keyframe_at(current_time);
            } else {
                layer.transform.scale.add_keyframe(current_time, scale, KeyframeInterpolation::Linear);
            }
        }
        ui.label("Scale (%):");
        let mut sx = scale.x;
        let mut sy = scale.y;
        if ui.add(egui::DragValue::new(&mut sx).speed(0.5).prefix("X: ")).changed()
            || ui.add(egui::DragValue::new(&mut sy).speed(0.5).prefix("Y: ")).changed()
        {
            scale = Vec2::new(sx, sy);
            if layer.transform.scale.is_animated() {
                layer.transform.scale.add_keyframe(current_time, scale, KeyframeInterpolation::Linear);
            } else {
                layer.transform.scale.base_value = scale;
            }
        }
    });

    // Rotation
    let mut rot = layer.transform.rotation.sample(current_time);
    let has_rot_kf = layer.transform.rotation.keyframes.iter().any(|k| (k.time - current_time).abs() < 1e-4);
    ui.horizontal(|ui| {
        let btn_text = if has_rot_kf { "◆" } else { "◇" };
        if ui.button(btn_text).on_hover_text("Add/Remove Rotation Keyframe").clicked() {
            if has_rot_kf {
                layer.transform.rotation.remove_keyframe_at(current_time);
            } else {
                layer.transform.rotation.add_keyframe(current_time, rot, KeyframeInterpolation::Linear);
            }
        }
        ui.label("Rotation:");
        if ui.add(egui::DragValue::new(&mut rot).speed(0.5).suffix("°")).changed() {
            if layer.transform.rotation.is_animated() {
                layer.transform.rotation.add_keyframe(current_time, rot, KeyframeInterpolation::Linear);
            } else {
                layer.transform.rotation.base_value = rot;
            }
        }
    });

    // Opacity
    let mut opacity = layer.transform.opacity.sample(current_time);
    let has_op_kf = layer.transform.opacity.keyframes.iter().any(|k| (k.time - current_time).abs() < 1e-4);
    ui.horizontal(|ui| {
        let btn_text = if has_op_kf { "◆" } else { "◇" };
        if ui.button(btn_text).on_hover_text("Add/Remove Opacity Keyframe").clicked() {
            if has_op_kf {
                layer.transform.opacity.remove_keyframe_at(current_time);
            } else {
                layer.transform.opacity.add_keyframe(current_time, opacity, KeyframeInterpolation::Linear);
            }
        }
        ui.label("Opacity:");
        if ui.add(egui::Slider::new(&mut opacity, 0.0..=100.0).suffix("%")).changed() {
            if layer.transform.opacity.is_animated() {
                layer.transform.opacity.add_keyframe(current_time, opacity, KeyframeInterpolation::Linear);
            } else {
                layer.transform.opacity.base_value = opacity;
            }
        }
    });
}
