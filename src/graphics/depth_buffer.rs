use glam::{UVec2};
use crate::graphics::context::Graphics;

pub struct DepthBuffer {
  pub texture : wgpu::Texture,
  pub view    : wgpu::TextureView,
}

impl DepthBuffer {
  pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

  pub fn new(graphics: &Graphics, size: UVec2) -> Self {
    let size = wgpu::Extent3d {
      width: size.x,
      height: size.y,
      depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
      label: None,
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::DEPTH_FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT
           | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    };

    let texture = graphics.device.create_texture(&desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    return Self {
      texture,
      view
    };
  }
}