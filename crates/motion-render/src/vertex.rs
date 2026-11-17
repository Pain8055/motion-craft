use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

impl QuadVertex {
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ],
    };

    pub fn unit_quad() -> [Self; 4] {
        [
            Self { position: [0.0, 0.0], uv: [0.0, 0.0] },
            Self { position: [1.0, 0.0], uv: [1.0, 0.0] },
            Self { position: [1.0, 1.0], uv: [1.0, 1.0] },
            Self { position: [0.0, 1.0], uv: [0.0, 1.0] },
        ]
    }

    pub const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
}
