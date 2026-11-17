use crate::graph_view::render_graph_editor;
use crate::inspector_view::render_inspector;
use crate::timeline_view::render_timeline;
use crate::viewport_view::render_viewport;
use egui::Context;
use glam::Vec4;
use motion_core::{Composition, CubicBezier, Layer, Timeline};

pub struct MotionAppUi {
    pub comp: Composition,
    pub timeline: Timeline,
    pub selected_layer_id: Option<u64>,
    pub graph_bezier: CubicBezier,
    pub show_graph_editor: bool,
}

impl Default for MotionAppUi {
    fn default() -> Self {
        let mut comp = Composition::new("Main Comp", 1920, 1080, 60.0, 5.0);

        // Add default sample layers
        comp.add_layer(Layer::new_solid(
            1,
            "Background Solid",
            Vec4::new(0.08, 0.09, 0.12, 1.0),
            1920.0,
            1080.0,
            5.0,
        ));

        let mut title_layer = Layer::new_text(
            2,
            "Motion Title",
            "MOTION CRAFT",
            72.0,
            Vec4::new(1.0, 0.8, 0.2, 1.0),
            5.0,
        );
        // Animate position
        title_layer.transform.position.add_keyframe(
            0.0,
            glam::Vec2::new(960.0, 640.0),
            motion_core::KeyframeInterpolation::Bezier(CubicBezier::ease_out()),
        );
        title_layer.transform.position.add_keyframe(
            1.2,
            glam::Vec2::new(960.0, 500.0),
            motion_core::KeyframeInterpolation::Linear,
        );
        comp.add_layer(title_layer);

        Self {
            comp,
            timeline: Timeline::default(),
            selected_layer_id: Some(2),
            graph_bezier: CubicBezier::ease_in_out(),
            show_graph_editor: false,
        }
    }
}

impl MotionAppUi {
    pub fn update(&mut self, ctx: &Context, delta_seconds: f32) {
        // Advance timeline if playing
        self.timeline.advance(delta_seconds, self.comp.duration);

        // Top Menu Bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Composition").clicked() {
                        self.comp = Composition::default();
                        ui.close_menu();
                    }
                    if ui.button("Export MP4 Video").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Layer", |ui| {
                    if ui.button("New Solid Layer").clicked() {
                        self.comp.add_layer(Layer::new_solid(
                            0,
                            "New Solid",
                            Vec4::new(0.2, 0.4, 0.8, 1.0),
                            400.0,
                            400.0,
                            self.comp.duration,
                        ));
                        ui.close_menu();
                    }
                    if ui.button("New Text Layer").clicked() {
                        self.comp.add_layer(Layer::new_text(
                            0,
                            "Text Layer",
                            "New Text",
                            48.0,
                            Vec4::ONE,
                            self.comp.duration,
                        ));
                        ui.close_menu();
                    }
                });

                ui.toggle_value(&mut self.show_graph_editor, "Graph Editor");
            });
        });

        // Bottom Timeline Panel
        egui::TopBottomPanel::bottom("timeline_panel")
            .resizable(true)
            .min_height(160.0)
            .default_height(240.0)
            .show(ctx, |ui| {
                render_timeline(ui, &mut self.comp, &mut self.timeline, &mut self.selected_layer_id);
            });

        // Right Inspector Panel
        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_inspector(ui, &mut self.comp, self.selected_layer_id, self.timeline.current_time);

                    if self.show_graph_editor {
                        ui.separator();
                        render_graph_editor(ui, &mut self.graph_bezier);
                    }
                });
            });

        // Central Composition Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            render_viewport(ui, &self.comp, self.selected_layer_id, self.timeline.current_time);
        });
    }
}
