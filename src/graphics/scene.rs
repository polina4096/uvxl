use std::marker::PhantomData;
use glam::Mat4;

use crate::graphics::{camera::{Projection, Camera, Transformation}, uniform::Uniform, bindable::Bindable};

use super::layout::Layout;

pub struct Scene<P: Projection, T> {
  pub projection: P,
  pub camera: Camera<T>,
  pub uniform: Uniform<Mat4>,
  pub _1: PhantomData<T>,
}

impl<P: Projection, T> Scene<P, T> {
  pub fn new(device: &wgpu::Device, projection: P, camera: Camera<T>) -> Self {
    return Self {
      projection,
      camera,
      uniform: Uniform::new(device),
      _1: Default::default(),
    };
  }
}

impl<P: Projection, T> Scene<P, T> {
  pub fn update(&self, queue: &wgpu::Queue)
    where Camera<T>: Transformation
  {
    self.uniform.update(queue, &self.apply());
  }
}

impl<P: Projection, T> Bindable for Scene<P, T> {
  fn bind<'pass, 'uniform: 'pass>(&'uniform self, render_pass: &mut wgpu::RenderPass<'pass>, index: u32) {
    self.uniform.bind(render_pass, index);
  }

  fn group(&self) -> &wgpu::BindGroup {
    return self.uniform.group();
  }
}

impl<P: Projection, T> Layout for Scene<P, T> {
  fn layout(&self) -> &wgpu::BindGroupLayout {
    return self.uniform.layout();
  }
}

impl<P: Projection, T> Transformation for Scene<P, T>
  where Camera<T>: Transformation
{
  fn apply(&self) -> Mat4 {
    return self.projection.apply() * self.camera.apply();
  }
}