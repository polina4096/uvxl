use glam::{Vec3, Vec2, vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos : Vec3,
    pub uv  : Vec2,
}

impl Vertex {
    pub fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }

    pub fn vertices_quad(min: f32, max: f32) -> Vec<Self> {
        return vec![
            Vertex { pos: vec3(min, min, 1.0), uv: Vec2::new(0.0, 0.0) },
            Vertex { pos: vec3(min, max, 1.0), uv: Vec2::new(0.0, 1.0) },
            Vertex { pos: vec3(max, max, 1.0), uv: Vec2::new(1.0, 1.0) },
            Vertex { pos: vec3(max, max, 1.0), uv: Vec2::new(1.0, 1.0) },
            Vertex { pos: vec3(max, min, 1.0), uv: Vec2::new(1.0, 0.0) },
            Vertex { pos: vec3(min, min, 1.0), uv: Vec2::new(0.0, 0.0) },
        ];
    }
}