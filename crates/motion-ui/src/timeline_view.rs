use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use motion_core::{Composition, KeyframeInterpolation, Timeline};

pub fn render_timeline(
    ui: &mut Ui,
    comp: &mut Composition,
    timeline: &mut Timeline,
    selected_layer_id: &mut Option<u64>,
) {
    ui.vertical(|ui| {
        // Transport Controls
        ui.horizontal(|ui| {
            if ui.button(if timeline.state == motion_core::PlaybackState::Playing { "⏸ Pause" } else { "▶ Play" }).clicked() {
                timeline.toggle_play();
            }

            if ui.button("⏮").clicked() {
                timeline.seek(0.0, comp.duration);
            }

            let current_frame = comp.time_to_frame(timeline.current_time);
            let total_frames = comp.total_frames();
            ui.label(format!(
                "Time: {:02.2}s | Frame: {} / {}",
                timeline.current_time, current_frame, total_frames
            ));

            ui.checkbox(&mut timeline.is_looping, "Loop");

            ui.separator();
            ui.label("Zoom:");
            ui.add(egui::Slider::new(&mut timeline.zoom, 40.0..=300.0).text("px/s"));
        });

        ui.separator();

        // Timeline Tracks Area
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());
        let rect = response.rect;

        let header_height = 28.0;
        let track_height = 36.0;
        let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));

        // Draw Time Ruler Header
        painter.rect_filled(header_rect, 0.0, Color32::from_rgb(30, 32, 36));
        painter.line_segment(
            [header_rect.left_bottom(), header_rect.right_bottom()],
            Stroke::new(1.0, Color32::from_rgb(50, 52, 58)),
        );

        let step_seconds = if timeline.zoom > 150.0 { 0.5 } else { 1.0 };
        let mut t = 0.0;
        while t <= comp.duration {
            let x = rect.left() + 180.0 + t * timeline.zoom;
            if x <= rect.right() {
                painter.line_segment(
                    [Pos2::new(x, rect.top() + 14.0), Pos2::new(x, rect.top() + header_height)],
                    Stroke::new(1.0, Color32::from_rgb(90, 94, 102)),
                );
                painter.text(
                    Pos2::new(x + 4.0, rect.top() + 6.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.1}s", t),
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(160, 164, 172),
                );
            }
            t += step_seconds;
        }

        // Handle Playhead Scrubbing
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let track_x = mouse_pos.x - (rect.left() + 180.0);
                let new_time = (track_x / timeline.zoom).clamp(0.0, comp.duration);
                timeline.seek(new_time, comp.duration);
            }
        }

        // Draw Layer Tracks
        let mut y = rect.top() + header_height;
        for layer in comp.layers.iter_mut() {
            let track_rect = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(rect.width(), track_height));
            let is_selected = *selected_layer_id == Some(layer.id);

            let bg_color = if is_selected {
                Color32::from_rgb(45, 55, 75)
            } else {
                Color32::from_rgb(22, 24, 28)
            };

            painter.rect_filled(track_rect, 0.0, bg_color);
            painter.line_segment(
                [track_rect.left_bottom(), track_rect.right_bottom()],
                Stroke::new(1.0, Color32::from_rgb(38, 40, 46)),
            );

            // Layer Label and Select
            let label_rect = Rect::from_min_size(track_rect.min, Vec2::new(180.0, track_height));
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if label_rect.contains(pos) {
                        *selected_layer_id = Some(layer.id);
                    }
                }
            }

            painter.text(
                Pos2::new(label_rect.left() + 12.0, label_rect.center().y),
                egui::Align2::LEFT_CENTER,
                &layer.name,
                egui::FontId::proportional(13.0),
                if is_selected { Color32::WHITE } else { Color32::from_rgb(180, 185, 195) },
            );

            // Layer Bar Duration on Timeline
            let bar_start_x = rect.left() + 180.0 + layer.in_point * timeline.zoom;
            let bar_end_x = rect.left() + 180.0 + layer.out_point * timeline.zoom;
            let bar_rect = Rect::from_min_max(
                Pos2::new(bar_start_x, y + 6.0),
                Pos2::new(bar_end_x, y + track_height - 6.0),
            );

            painter.rect_filled(bar_rect, 4.0, Color32::from_rgb(70, 95, 140));

            // Render Keyframe Diamonds
            draw_keyframes(&painter, &layer.transform.position.keyframes, timeline.zoom, rect.left() + 180.0, y + track_height * 0.5);
            draw_keyframes(&painter, &layer.transform.scale.keyframes, timeline.zoom, rect.left() + 180.0, y + track_height * 0.5);
            draw_keyframes(&painter, &layer.transform.rotation.keyframes, timeline.zoom, rect.left() + 180.0, y + track_height * 0.5);
            draw_keyframes(&painter, &layer.transform.opacity.keyframes, timeline.zoom, rect.left() + 180.0, y + track_height * 0.5);

            y += track_height;
        }

        // Draw Scrubber / Playhead Line
        let playhead_x = rect.left() + 180.0 + timeline.current_time * timeline.zoom;
        if playhead_x <= rect.right() {
            painter.line_segment(
                [Pos2::new(playhead_x, rect.top()), Pos2::new(playhead_x, rect.bottom())],
                Stroke::new(2.0, Color32::from_rgb(240, 70, 70)),
            );
            // Playhead Handle
            let handle_size = 7.0;
            let handle_top = Pos2::new(playhead_x, rect.top());
            let handle_left = Pos2::new(playhead_x - handle_size, rect.top() + header_height * 0.6);
            let handle_right = Pos2::new(playhead_x + handle_size, rect.top() + header_height * 0.6);
            painter.add(egui::epaint::PathShape::convex_polygon(
                vec![handle_top, handle_right, Pos2::new(playhead_x, rect.top() + header_height * 0.9), handle_left],
                Color32::from_rgb(240, 70, 70),
                Stroke::NONE,
            ));
        }
    });
}

fn draw_keyframes<T>(painter: &egui::Painter, keyframes: &[motion_core::Keyframe<T>], zoom: f32, origin_x: f32, y: f32) {
    for k in keyframes {
        let x = origin_x + k.time * zoom;
        let s = 4.5;
        let diamond = vec![
            Pos2::new(x, y - s),
            Pos2::new(x + s, y),
            Pos2::new(x, y + s),
            Pos2::new(x - s, y),
        ];
        let color = match k.interpolation {
            KeyframeInterpolation::Linear => Color32::from_rgb(255, 215, 0),
            KeyframeInterpolation::Hold => Color32::from_rgb(255, 120, 60),
            KeyframeInterpolation::Bezier(_) => Color32::from_rgb(60, 200, 255),
        };
        painter.add(egui::epaint::PathShape::convex_polygon(diamond, color, Stroke::new(1.0, Color32::BLACK)));
    }
}
