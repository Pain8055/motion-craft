use motion_render::MotionRenderer;
use motion_ui::MotionAppUi;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct MotionApp {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    renderer: Option<MotionRenderer>,
    egui_ctx: Option<egui::Context>,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    app_ui: MotionAppUi,
    last_frame_time: Instant,
}

impl Default for MotionApp {
    fn default() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            queue: None,
            surface_config: None,
            renderer: None,
            egui_ctx: None,
            egui_state: None,
            egui_renderer: None,
            app_ui: MotionAppUi::default(),
            last_frame_time: Instant::now(),
        }
    }
}

impl ApplicationHandler for MotionApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let win_attr = Window::default_attributes()
            .with_title("Motion Craft — After Effects in Rust (wgpu / Android)")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

        let window = match event_loop.create_window(win_attr) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                log::error!("Failed to create window: {:?}", e);
                return;
            }
        };
        self.window = Some(window.clone());

        log::info!("Creating wgpu Instance with all backends (Vulkan, GLES)...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = match instance.create_surface(window.clone()) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to create wgpu surface: {:?}", e);
                return;
            }
        };

        log::info!("Requesting compatible GPU adapter...");
        let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })) {
            Some(a) => a,
            None => {
                log::warn!("No adapter found with compatible_surface constraint, trying without surface...");
                match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })) {
                    Some(a) => a,
                    None => {
                        log::warn!("No default adapter found, trying fallback adapter...");
                        match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                            power_preference: wgpu::PowerPreference::default(),
                            compatible_surface: None,
                            force_fallback_adapter: true,
                        })) {
                            Some(a) => a,
                            None => {
                                log::error!("CRITICAL: No compatible GPU adapter found on this device!");
                                return;
                            }
                        }
                    }
                }
            }
        };

        log::info!("Selected GPU Adapter: {:?}", adapter.get_info());

        let (device, queue) = match pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Motion Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                memory_hints: Default::default(),
            },
            None,
        )) {
            Ok(pair) => pair,
            Err(e) => {
                log::error!("Failed to create GPU device: {:?}", e);
                return;
            }
        };

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let surface_caps = surface.get_capabilities(&adapter);
        if surface_caps.formats.is_empty() {
            log::error!("Surface reported 0 supported formats!");
            return;
        }

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = if surface_caps.present_modes.contains(&wgpu::PresentMode::AutoVsync) {
            wgpu::PresentMode::AutoVsync
        } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else if !surface_caps.present_modes.is_empty() {
            surface_caps.present_modes[0]
        } else {
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = if !surface_caps.alpha_modes.is_empty() {
            surface_caps.alpha_modes[0]
        } else {
            wgpu::CompositeAlphaMode::Auto
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = MotionRenderer::new(&device, surface_format);

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            None,
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_format,
            None,
            1,
            false,
        );

        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface_config = Some(config);
        self.renderer = Some(renderer);
        self.egui_ctx = Some(egui_ctx);
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);

        log::info!("Motion Craft initialization completed successfully. Triggering first frame redraw.");
        window.request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Application suspended: releasing surface and window handles");
        self.surface = None;
        self.window = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let window = match self.window.as_ref() {
            Some(w) => w,
            None => return,
        };

        if let Some(egui_state) = self.egui_state.as_mut() {
            let res = egui_state.on_window_event(window, &event);
            if res.consumed {
                window.request_redraw();
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(surface), Some(device), Some(config)) = (
                    self.surface.as_ref(),
                    self.device.as_ref(),
                    self.surface_config.as_mut(),
                ) {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(device, config);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_seconds = (now - self.last_frame_time).as_secs_f32().min(0.1);
                self.last_frame_time = now;

                let (surface, device, queue, config, renderer, egui_ctx, egui_state, egui_renderer) =
                    match (
                        self.surface.as_ref(),
                        self.device.as_ref(),
                        self.queue.as_ref(),
                        self.surface_config.as_ref(),
                        self.renderer.as_ref(),
                        self.egui_ctx.as_ref(),
                        self.egui_state.as_mut(),
                        self.egui_renderer.as_mut(),
                    ) {
                        (
                            Some(s),
                            Some(d),
                            Some(q),
                            Some(c),
                            Some(r),
                            Some(ec),
                            Some(es),
                            Some(er),
                        ) => (s, d, q, c, r, ec, es, er),
                        _ => return,
                    };

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Lost) => {
                        log::warn!("Surface lost, reconfiguring surface...");
                        surface.configure(device, config);
                        return;
                    }
                    Err(wgpu::SurfaceError::Outdated) => {
                        log::warn!("Surface outdated, reconfiguring surface...");
                        surface.configure(device, config);
                        return;
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface acquire timeout, skipping frame");
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Surface out of memory, exiting...");
                        event_loop.exit();
                        return;
                    }
                    Err(e) => {
                        log::warn!("Surface texture error: {:?}", e);
                        return;
                    }
                };

                let view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Frame Encoder"),
                    });

                // 1. Render Composition Viewport
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Composition Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.08,
                                    g: 0.09,
                                    b: 0.11,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    renderer.render_composition(
                        queue,
                        &mut render_pass,
                        &self.app_ui.comp,
                        self.app_ui.timeline.current_time,
                    );
                }

                // 2. Render UI (egui)
                let raw_input = egui_state.take_egui_input(window);
                let full_output = egui_ctx.run(raw_input, |ctx| {
                    self.app_ui.update(ctx, delta_seconds);
                });

                egui_state.handle_platform_output(window, full_output.platform_output);
                let tris = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                for (id, delta) in &full_output.textures_delta.set {
                    egui_renderer.update_texture(device, queue, *id, delta);
                }

                let screen_desc = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [config.width, config.height],
                    pixels_per_point: full_output.pixels_per_point,
                };

                egui_renderer.update_buffers(device, queue, &mut encoder, &tris, &screen_desc);

                {
                    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Egui Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    egui_renderer.render(&mut render_pass.forget_lifetime(), &tris, &screen_desc);
                }

                for id in &full_output.textures_delta.free {
                    egui_renderer.free_texture(id);
                }

                queue.submit(Some(encoder.finish()));
                output_frame.present();

                window.request_redraw();
            }
            _ => {}
        }
    }
}
