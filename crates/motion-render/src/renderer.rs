use crate::vertex::QuadVertex;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use motion_core::{Composition, LayerContent};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ShaderUniforms {
    view_proj: [f32; 16],
    model: [f32; 16],
    color: [f32; 4],
    opacity: f32,
    blend_mode: u32,
    _pad0: u32,
    _pad1: u32,
}

pub struct MotionRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl MotionRenderer {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Motion Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Motion Uniform Buffer"),
            size: std::mem::size_of::<ShaderUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Motion Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Motion Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Motion Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Motion Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[QuadVertex::DESC],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&QuadVertex::unit_quad()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(&QuadVertex::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn render_composition(
        &self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        comp: &Composition,
        time: f32,
    ) {
        // Orthographic projection matching composition pixels (0,0 is top-left)
        let view_proj = Mat4::orthographic_lh(
            0.0,
            comp.width as f32,
            comp.height as f32,
            0.0,
            -1000.0,
            1000.0,
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Render back-to-front (reverse order)
        for layer in comp.layers.iter().rev() {
            if !layer.is_active_at(time) {
                continue;
            }

            let world_mat = comp.compute_world_matrix(layer.id, time);
            let opacity = layer.transform.compute_opacity(time);

            let (color, quad_scale) = match &layer.content {
                LayerContent::Solid { color, width, height } => (*color, glam::Vec2::new(*width, *height)),
                LayerContent::Text { color, font_size, .. } => (*color, glam::Vec2::new(font_size * 5.0, *font_size)),
                LayerContent::Image { width, height, .. } => (glam::Vec4::ONE, glam::Vec2::new(*width, *height)),
                LayerContent::Adjustment => (glam::Vec4::ONE, glam::Vec2::new(comp.width as f32, comp.height as f32)),
            };

            let model = world_mat * Mat4::from_scale(quad_scale.extend(1.0));

            let uniforms = ShaderUniforms {
                view_proj: view_proj.to_cols_array(),
                model: model.to_cols_array(),
                color: color.to_array(),
                opacity,
                blend_mode: layer.blend_mode.shader_id(),
                _pad0: 0,
                _pad1: 0,
            };

            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}
