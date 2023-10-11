use glam::{ivec3, Quat, vec3};
use log::info;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use crate::app::App;
use crate::game::client::graphics::entity_model::EntityModel;
use crate::game::client::graphics::world_renderer::WorldRenderer;
use crate::game::entity::{Entity, EntityState};
use crate::game::entity::player::EntityPlayer;
use crate::game::network::packet::{InitialChunkDataServerPacket, ClientJoinServerPacket, ServerPacket, ClientPacket, ClientMovePacket, PlayerJoinServerPacket, PlayerMoveServerPacket};
use crate::game::player::Player;
use crate::game::world::chunk::ChunkVec3Ext;
use crate::game::world::world::World;
use crate::graphics::instance::Instance;
use crate::input::camera_controller::CameraController;

pub struct Client {
  pub world_renderer : WorldRenderer,

  pub camera_controller : CameraController,

  pub world  : World,
  pub player : Player, // later we might want to have a client player which holds addition client information such as auth or other stuff
}

impl Client {
  pub fn new(app: &mut App) -> Self {
    let world_renderer = WorldRenderer::new(app);
    let camera_controller = CameraController::new(20.0, 1.0);

    return Self {
      world_renderer,
      camera_controller,

      world: Default::default(),
      player: Default::default(),
    };
  }

  pub fn render(&mut self, app: &mut App, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
    self.camera_controller.update_camera(&mut self.world_renderer.scene.camera, app.delta);
    self.player.entity.state_mut().position = self.world_renderer.scene.camera.position;
    self.world_renderer.render(app, view, encoder);
  }

  pub fn update(&mut self, app: &mut App) {
    if let Some(connection) = &mut app.connection {
      connection.send(ClientPacket::ClientMovePacket(ClientMovePacket {
        position: self.player.entity.state().position,
      })).unwrap();
    }
  }

  pub fn resize(&mut self, app: &mut App, size: PhysicalSize<u32>) {
    self.world_renderer.resize(app, size);
  }

  pub fn window_event(&mut self, app: &mut App, event: &WindowEvent) {
    match event {
      WindowEvent::KeyboardInput { input, .. } => {
        if let Some(keycode) = input.virtual_keycode {
          self.camera_controller.on_keyboard(keycode, input.state);
        }
      }

      _ => { }
    }
  }

  pub fn device_event(&mut self, app: &mut App, event: &DeviceEvent) {
    match event {
      DeviceEvent::MouseMotion { delta } => {
        self.camera_controller.on_mouse(delta.0, delta.1);
      }

      _ => {}
    }
  }

  pub fn packet(&mut self, app: &mut App, packet: &ServerPacket) {
    match packet {
      ServerPacket::ClientJoinServerPacket(ClientJoinServerPacket { success, reason, players }) => {
        for player in players {
          let entity = EntityPlayer::new(
            EntityState {
              title: Some(player.0.clone()),
              position: player.1,
              velocity: vec3(0.0, 0.0, 0.0),
              rotation: Quat::IDENTITY,
            }
          );

          self.world.players.push(Player { name: player.0.clone(), entity });
          self.world_renderer.entity_renderer.entities_mesh.instances = self.world.players.iter()
            .map(|x| EntityModel { position: x.entity.state().position }).collect();

          self.world_renderer.entity_renderer.entities_mesh.bake_instances(&app.graphics);
          self.world_renderer.entity_renderer.entities_mesh.update_instances(
            bytemuck::cast_slice(self.world_renderer.entity_renderer.entities_mesh.instances.iter().map(Instance::bake).collect::<Vec<_>>().as_slice()), &app.graphics.queue);
        }
      }

      ServerPacket::InitialChunkDataServerPacket(InitialChunkDataServerPacket { chunk, position }) => {
        self.world.chunk_manager.chunks.insert(*position, chunk.clone());
        // self.world_renderer.chunk_renderer.chunk_meshes.clear();

        let vertical_render_distance = 4;
        let horizontal_render_distance = 2;
        let chunk_pos = self.player.entity.state().position.to_chunk_pos();
        for x in -horizontal_render_distance ..= horizontal_render_distance {
          for y in -vertical_render_distance ..= vertical_render_distance {
            for z in -horizontal_render_distance ..= horizontal_render_distance {
              let chunk_pos = ivec3(chunk_pos.x + x, chunk_pos.y + y, chunk_pos.z + z);
              let Some(chunk) = self.world.chunk_manager.chunks.get(&chunk_pos) else { continue };
              if !self.world_renderer.chunk_renderer.chunk_meshes.contains_key(&chunk_pos) {
                self.world_renderer.chunk_renderer.chunk_sender.send((chunk_pos, chunk.blocks.clone())).unwrap();
              }

              // self.world_renderer.chunk_renderer.add_chunk(chunk_pos, chunk.clone(), &app.graphics);
            }
          }
        }
      }

      ServerPacket::PlayerJoinServerPacket(PlayerJoinServerPacket { name, position }) => {
        dbg!(&name);

        let entity = EntityPlayer::new(
          EntityState {
            title: Some(name.clone()),
            position: *position,
            velocity: vec3(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
          }
        );

        self.world.players.push(Player { name: name.clone(), entity });

        self.world_renderer.entity_renderer.entities_mesh.instances = self.world.players.iter()
          .map(|x| EntityModel { position: x.entity.state().position }).collect();

        self.world_renderer.entity_renderer.entities_mesh.bake_instances(&app.graphics);
        self.world_renderer.entity_renderer.entities_mesh.update_instances(
          bytemuck::cast_slice(self.world_renderer.entity_renderer.entities_mesh.instances.iter().map(Instance::bake).collect::<Vec<_>>().as_slice()), &app.graphics.queue);
      }

      ServerPacket::PlayerMoveServerPacket(PlayerMoveServerPacket { name, position }) => {
        for player in &mut self.world.players {
          if player.name == *name {
            player.entity.state_mut().position = *position;
          }
        }

        self.world_renderer.entity_renderer.entities_mesh.instances = self.world.players.iter()
          .map(|x| EntityModel { position: x.entity.state().position }).collect();

        self.world_renderer.entity_renderer.entities_mesh.bake_instances(&app.graphics);
        self.world_renderer.entity_renderer.entities_mesh.update_instances(
          bytemuck::cast_slice(self.world_renderer.entity_renderer.entities_mesh.instances.iter().map(Instance::bake).collect::<Vec<_>>().as_slice()), &app.graphics.queue);
      }
    }
  }
}