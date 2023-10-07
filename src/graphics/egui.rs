use egui::ClippedPrimitive;
use winit::event_loop::EventLoop;
use winit::window::Window;
use crate::graphics::context::Graphics;

pub struct EGuiContext {
  pub context     : egui::Context,
  pub renderer    : egui_wgpu::Renderer,
  pub screen_desc : egui_wgpu::renderer::ScreenDescriptor,
  pub winit_state : egui_winit::State,
}

impl EGuiContext {
  pub fn new<T>(event_loop: &EventLoop<T>, graphics: &Graphics) -> Self {
    let context = egui::Context::default();
    let renderer = egui_wgpu::Renderer::new(&graphics.device, graphics.format, None, 1);
    let screen_desc = egui_wgpu::renderer::ScreenDescriptor {
      size_in_pixels   : [graphics.size.width, graphics.size.height],
      pixels_per_point : graphics.scale as f32,
    };

    #[allow(unused_mut)]
    let mut winit_state = egui_winit::State::new(event_loop);
    winit_state.set_max_texture_side(graphics.device.limits().max_texture_dimension_2d as usize);
    winit_state.set_pixels_per_point(graphics.scale as f32);

    return Self {
      context,
      renderer,
      screen_desc,
      winit_state,
    }
  }

  pub fn begin_frame(&mut self, window: &Window) {
    let new_input = self.winit_state.take_egui_input(window);
    self.context.begin_frame(new_input);
  }

  pub fn end_frame(&mut self, graphics: &Graphics, encoder: &mut wgpu::CommandEncoder) -> (Vec<ClippedPrimitive>, Vec<wgpu::CommandBuffer>) {
    let egui_output = self.context.end_frame();

    // Free textures
    for id in &egui_output.textures_delta.free {
      self.renderer.free_texture(id);
    }

    // Upload textures
    for (id, image_delta) in &egui_output.textures_delta.set {
      self.renderer.update_texture(
        &graphics.device,
        &graphics.queue,
        *id,
        image_delta,
      );
    }

    // Generate vertices and render commands
    let clipped_primitives = self.context.tessellate(egui_output.shapes);
    let commands = self.renderer.update_buffers(
      &graphics.device,
      &graphics.queue,
      encoder,
      &clipped_primitives,
      &self.screen_desc,
    );

    return (clipped_primitives, commands);
  }

  pub fn render<'this: 'pass, 'pass>(&'this mut self, graphics: &Graphics, render_pass: &mut wgpu::RenderPass<'pass>, clipped_primitives: &'pass [ClippedPrimitive], commands: Vec<wgpu::CommandBuffer>) {
    self.renderer.render(render_pass, clipped_primitives, &self.screen_desc);
    graphics.queue.submit(commands);
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    self.screen_desc.size_in_pixels = [new_size.width, new_size.height];
  }

  pub fn scale(&mut self, scale_factor: f64) {
    self.screen_desc.pixels_per_point = scale_factor as f32;
  }
}