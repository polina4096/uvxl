use std::fmt::Debug;
use glam::IVec3;
use log::warn;
use winit::window::Window;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow, EventLoopProxy};
use winit::dpi::PhysicalSize;
use crate::game::client::client::Client;
use crate::game::client::graphics::chunk_model::ChunkModel;
use crate::game::client::graphics::world_renderer::WorldRenderer;
use crate::game::network::packet::{ClientPacket, ClientJoinClientPacket, ServerPacket};
use crate::game::client::window::WindowStack;
use crate::game::client::window::server_join::ServerJoinWindow;
use crate::game::world::chunk::CHUNK_SIZE;
use crate::graphics::context::Graphics;
use crate::graphics::egui::EGuiContext;
use crate::graphics::mesh::InstancedMesh;
use crate::graphics::vertex::Vertex;
use crate::input::input::Input;
use crate::network::connection::Connection;

pub enum UVxlEvent {
  ConnectionReady,
  IncomingPacket(ServerPacket),
  MesherChunkDone(IVec3, Vec<Vertex>),
  MutateWindowStack(Box<dyn FnOnce(&mut App, &mut WindowStack)>),

  SetClientName(String),
}

impl Debug for UVxlEvent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::ConnectionReady => f.write_str("ConnectionReady"),
      Self::IncomingPacket(..) => f.write_str("IncomingPacket"),
      Self::MutateWindowStack(..) => f.write_str("MutateWindowStack"),
      Self::MesherChunkDone(..) => f.write_str("MesherChunkDone"),
      Self::SetClientName(..) => f.write_str("SetClientName"),
    }
  }
}

pub struct App {
  pub event_proxy : EventLoopProxy<UVxlEvent>,
  pub window      : Window,
  pub input       : Input,
  pub graphics    : Graphics,
  pub egui_ctx    : EGuiContext,
  pub connection  : Option<Connection>,

  pub last_update : instant::Instant,
  pub last_render : instant::Instant,
  pub delta       : instant::Duration,
}

impl App {
  pub async fn run(window: Window, event_loop: EventLoop<UVxlEvent>) {
    let input = Input::default();
    let graphics = Graphics::new(&window).await;
    let egui_ctx = EGuiContext::new(&event_loop, &graphics);
    let event_proxy = event_loop.create_proxy();

    let mut window_stack: WindowStack = vec![
      Box::<ServerJoinWindow>::default()
    ];

    let now = instant::Instant::now();
    let mut app = App {
      event_proxy,
      window,
      input,
      graphics,

      egui_ctx,

      connection : None,

      last_update : now,
      last_render : now,
      delta       : instant::Duration::ZERO,
    };

    let mut client = {
      let world_renderer = WorldRenderer::new(&app);

      Client::new(
        &mut app,
      )
    };

    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;

      let now = instant::Instant::now();
      const NANOS_PER_SECOND: u64 = 1_000_000_000;
      const TICK_RATE: usize = 20;
      const TICK_RATE_DURATION: instant::Duration = instant::Duration::from_nanos(NANOS_PER_SECOND / TICK_RATE as u64);
      if now - app.last_update > TICK_RATE_DURATION {
        app.last_update = now;
        client.update(&mut app);
      }

      match event {
        Event::RedrawRequested(window_id) if window_id == app.window.id() => {
          let now = instant::Instant::now();
          app.delta = now - app.last_render;
          app.last_render = now;

          match app.render(&mut client, &mut window_stack) {
            Ok(_) => { }
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => app.resize(&mut client, app.graphics.size),
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => warn!("{:?}", e),
          }
        }

        Event::MainEventsCleared => {
          app.window.request_redraw();
        }

        Event::DeviceEvent { event, .. } => {
          app.device_event(&mut client, &event);
        }

        Event::WindowEvent { event, window_id } if window_id == app.window.id() => {
          // If not consumed
          if !app.window_event(&mut client, &event) {
            match event {
              WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit
              }

              WindowEvent::Resized(physical_size) => {
                app.resize(&mut client, physical_size);
              }

              WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } => {
                app.resize(&mut client, *new_inner_size);
                app.scale(scale_factor);
              }

              _ => {}
            }
          }
        }

        Event::UserEvent(event) => {
          match event {
            UVxlEvent::ConnectionReady => {
              dbg!(&client.player.name);
              app.connection.as_mut().unwrap().send(ClientPacket::ClientJoinClientPacket(ClientJoinClientPacket {
                name: client.player.name.clone(),
              })).unwrap();
            }

            UVxlEvent::MutateWindowStack(closure) => {
              closure(&mut app, &mut window_stack);
            }

            UVxlEvent::IncomingPacket(packet) => {
              client.packet(&mut app, &packet);
            }

            UVxlEvent::MesherChunkDone(position, data) => {
              let chunk_mesh = InstancedMesh::new(&app.graphics, data, vec![ChunkModel { position: (position * CHUNK_SIZE as i32).as_vec3() }]);
              client.world_renderer.chunk_renderer.chunk_meshes.insert(position, chunk_mesh);
            }

            UVxlEvent::SetClientName(name) => {
              client.player.name = name;
              dbg!(&client.player.name);
            }
          }
        }

        _ => {}
      }
    });
  }
}

impl App {
  fn render(&mut self, client: &mut Client, window_stack: &mut WindowStack) -> Result<(), wgpu::SurfaceError> {
    let output = self.graphics.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render encoder"),
    });

    client.render(self, &view, &mut encoder);

    self.egui_ctx.begin_frame(&self.window);
    for window in window_stack { window.draw(self); }
    let (clipped_primitives, commands) = self.egui_ctx.end_frame(&self.graphics, &mut encoder);

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: true,
          },
        })],
        depth_stencil_attachment: None,
      });

      self.egui_ctx.render(&self.graphics, &mut render_pass, &clipped_primitives, commands);
    }

    self.graphics.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    return Ok(());
  }

  fn device_event(&mut self, client: &mut Client, event: &DeviceEvent) {
    client.device_event(self, event);
  }

  fn window_event(&mut self, client: &mut Client, event: &WindowEvent) -> bool {
    client.window_event(self, event);

    return self.egui_ctx.winit_state.on_event(&self.egui_ctx.context, event).consumed;
  }

  fn resize(&mut self, client: &mut Client, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.graphics.size = new_size;
      self.graphics.config.width = new_size.width;
      self.graphics.config.height = new_size.height;
      self.graphics.surface.configure(&self.graphics.device, &self.graphics.config);

      self.egui_ctx.resize(new_size);

      client.resize(self, new_size);
    }
  }

  fn scale(&mut self, scale_factor: f64) {
    self.graphics.scale = scale_factor;

    self.egui_ctx.scale(scale_factor);
  }
}