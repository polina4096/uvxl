use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use crate::graphics::context::Graphics;
use crate::graphics::drawable::Drawable;
use crate::graphics::instance::Instance;

pub fn make_buffer<Vertex: Pod + Zeroable>(graphics: &Graphics, vertices: &[Vertex]) -> wgpu::Buffer {
  return graphics.device.create_buffer_init(
    &wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(vertices),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    }
  );
}

pub struct InstancedMesh<Vertex: Pod + Zeroable, InstanceImpl: Instance> {
  buffer          : wgpu::Buffer,
  instance_buffer : wgpu::Buffer,

  pub vertices        : Vec<Vertex>,
  pub instances       : Vec<InstanceImpl>,
}

impl<Vertex: Pod + Zeroable, InstanceImpl: Instance> InstancedMesh<Vertex, InstanceImpl> {
  pub fn new(graphics: &Graphics, vertices: Vec<Vertex>, instances: Vec<InstanceImpl>) -> Self {
    let instance_data = instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let instance_buffer = make_buffer(graphics, &instance_data);
    let buffer = make_buffer(graphics, &vertices);

    return Self {
      buffer,
      instance_buffer,

      vertices,
      instances,
    };
  }

  pub fn bake(&mut self, graphics: &Graphics) {
    self.buffer = make_buffer(graphics, &self.vertices);
  }

  pub fn bake_instances(&mut self, graphics: &Graphics) {
    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    self.instance_buffer = make_buffer(graphics, &instance_data);
  }

  pub fn update(&mut self, data: &[Vertex], queue: &wgpu::Queue) {
    queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
  }

  pub fn update_instances(&mut self, data: &[InstanceImpl::Baked], queue: &wgpu::Queue) {
    queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(data));
  }

  fn make_buffer<T: Pod + Zeroable>(device: &wgpu::Device, data: &[T]) -> wgpu::Buffer {
    return device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(data),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      }
    );
  }
}

impl<Vertex: Pod + Zeroable, InstanceImpl: Instance> Drawable for InstancedMesh<Vertex, InstanceImpl> {
  fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
    if !self.instances.is_empty() {
      render_pass.set_vertex_buffer(0, self.buffer.slice(..));
      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      render_pass.draw(0 .. self.vertices.len() as u32, 0 .. self.instances.len() as u32);
    }
  }
}