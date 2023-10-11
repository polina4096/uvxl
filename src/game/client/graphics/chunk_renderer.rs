use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use glam::{IVec3, Vec2, vec3, Vec4};
use wgpu::RenderPass;
use crate::game::client::graphics::chunk_model::ChunkModel;
use crate::game::world::BlockId;
use crate::game::world::chunk::Chunk;
use crate::graphics::atlas::Atlas;
use crate::graphics::drawable::Drawable;
use crate::graphics::mesh::InstancedMesh;
use crate::graphics::vertex::Vertex;
use crate::game::world::chunk::{CHUNK_SIZE};
use crate::graphics::bindable::Bindable;
use crate::graphics::context::Graphics;
use crate::util::side::Side;

pub type ChunkMesh = InstancedMesh<Vertex, ChunkModel>;

pub struct ChunkRenderer {
  pub atlas        : &'static Atlas<BlockId>,
  pub chunk_meshes : HashMap<IVec3, ChunkMesh>,
  pub chunk_sender : Sender<(IVec3, Vec<BlockId>)>,
}

impl ChunkRenderer {
  pub fn new(atlas: Atlas<BlockId>, sender: impl Fn(IVec3, Vec<Vertex>) + Send + 'static) -> Self {
    let (chunk_sender, receiver) = channel::<(IVec3, Vec<BlockId>)>();

    let atlas: &_ = Box::leak(Box::new(atlas));
    std::thread::spawn(move || {
      while let Ok((position, chunk)) = receiver.recv() {
        let vertices = mesher::culled::<CHUNK_SIZE>(&chunk, atlas);
        sender(position, vertices);
      }
    });

    return Self {
      atlas,
      chunk_meshes: HashMap::new(),
      chunk_sender,
    };
  }

  pub fn render<'this: 'pass, 'pass>(&'this self, render_pass: &mut RenderPass<'pass>, index: u32) {
    self.atlas.bind(render_pass, index);
    for mesh in self.chunk_meshes.values() {
      mesh.draw(render_pass);
    }
  }

  pub fn add_chunk(&mut self, chunk_pos: IVec3, chunk: Chunk, graphics: &Graphics) {
    let vertices = mesher::culled::<CHUNK_SIZE>(&chunk.blocks, self.atlas);
    let chunk_mesh = InstancedMesh::new(graphics, vertices, vec![ChunkModel { position: (chunk_pos * CHUNK_SIZE as i32).as_vec3() }]);
    self.chunk_meshes.insert(chunk_pos, chunk_mesh);
  }

  pub fn remove_chunk(&mut self, chunk_pos: IVec3) {
    self.chunk_meshes.remove(&chunk_pos);
  }
}

#[allow(dead_code)]
pub mod mesher {
  use super::*;

  // Meshing algorithms
  // Creates 6 faces for each voxel
  pub fn simple<const CHUNK_SIZE: usize>(data: &[BlockId], texture_atlas: &Atlas<BlockId>) -> Vec<Vertex> {
    let mut vertices = vec![];
    for i in 0 .. CHUNK_SIZE as isize {
      for j in 0 .. CHUNK_SIZE as isize {
        for k in 0 .. CHUNK_SIZE as isize {
          let block_state = data[index::<CHUNK_SIZE>(i, j, k)];
          if block_state != BlockId::AIR {
            let uv = texture_atlas.uv(&block_state);
            vertices.extend(block_face(Side::Top,    i, j, k, uv));
            vertices.extend(block_face(Side::Bottom, i, j, k, uv));
            vertices.extend(block_face(Side::Right,  i, j, k, uv));
            vertices.extend(block_face(Side::Left,   i, j, k, uv));
            vertices.extend(block_face(Side::Front,  i, j, k, uv));
            vertices.extend(block_face(Side::Back,   i, j, k, uv));
          }
        }

      }

    }

    return vertices;
  }

  // Creates only the faces visible from outside
  pub fn culled<const CHUNK_SIZE: usize>(data: &[BlockId], texture_atlas: &Atlas<BlockId>) -> Vec<Vertex> {
    let mut vertices = vec![];
    for i in 0 .. CHUNK_SIZE as isize {
      for j in 0 .. CHUNK_SIZE as isize {
        for k in 0 .. CHUNK_SIZE as isize {
          let block_state = data[index::<CHUNK_SIZE>(i, j, k)];
          if block_state != BlockId::AIR {
            let uv = texture_atlas.uv(&block_state);
            let chunk_volume: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
            let v = index::<CHUNK_SIZE>(i, j + 1, k); if v != chunk_volume { if unsafe { *data.get_unchecked(v) == BlockId::AIR } { vertices.extend(block_face(Side::Top,    i, j, k, uv)); } } else { vertices.extend(block_face(Side::Top,    i, j, k, uv)); }
            let v = index::<CHUNK_SIZE>(i, j - 1, k); if v != chunk_volume { if unsafe { *data.get_unchecked(v) == BlockId::AIR } { vertices.extend(block_face(Side::Bottom, i, j, k, uv)); } } else { vertices.extend(block_face(Side::Bottom, i, j, k, uv)); }
            let v = index::<CHUNK_SIZE>(i, j, k + 1); if v != chunk_volume { if unsafe { *data.get_unchecked(v) == BlockId::AIR } { vertices.extend(block_face(Side::Right,  i, j, k, uv)); } } else { vertices.extend(block_face(Side::Right,  i, j, k, uv)); }
            let v = index::<CHUNK_SIZE>(i, j, k - 1); if v != chunk_volume { if unsafe { *data.get_unchecked(v) == BlockId::AIR } { vertices.extend(block_face(Side::Left,   i, j, k, uv)); } } else { vertices.extend(block_face(Side::Left,   i, j, k, uv)); }
            let v = index::<CHUNK_SIZE>(i + 1, j, k); if v != chunk_volume { if unsafe { *data.get_unchecked(v) == BlockId::AIR } { vertices.extend(block_face(Side::Front,  i, j, k, uv)); } } else { vertices.extend(block_face(Side::Front,  i, j, k, uv)); }
            let v = index::<CHUNK_SIZE>(i - 1, j, k); if v != chunk_volume { if unsafe { *data.get_unchecked(v) == BlockId::AIR } { vertices.extend(block_face(Side::Back,   i, j, k, uv)); } } else { vertices.extend(block_face(Side::Back,   i, j, k, uv)); }
          }
        }

      }

    }

    return vertices;
  }

  const fn index<const CHUNK_SIZE: usize>(x: isize, y: isize, z: isize) -> usize {
    let chunk_size = CHUNK_SIZE as isize;
    let chunk_volume = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as isize;
    return if x < 0 || y < 0 || z < 0 || x >= chunk_size || y >= chunk_size || z >= chunk_size { chunk_volume as usize }
    else { ((z * chunk_size * chunk_size) + (y * chunk_size) + x) as usize };
  }
}

pub const fn block_face(side: Side, i: isize, j: isize, k: isize, uv: Vec4) -> [Vertex; 6] {
  let [ox, oy, sx, sy] = uv.to_array();

  #[allow(clippy::identity_op)]
  return match side {
    Side::Top => [
      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, oy) },
      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, oy) },
      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },

      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },
      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, sy) },
      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, oy) },
    ],
    Side::Bottom => [
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, oy) },
      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, sy) },
      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },

      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, oy) },
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, oy) },
    ],
    Side::Right => [
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(ox, oy) },
      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, oy) },
      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },

      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },
      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(ox, sy) },
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(ox, oy) },
    ],
    Side::Left => [
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(sx, oy) },
      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(sx, sy) },
      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, sy) },

      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, sy) },
      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, oy) },
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(sx, oy) },
    ],
    Side::Front => [
      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(sx, oy) },
      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(sx, sy) },
      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(ox, sy) },

      Vertex { pos: vec3((1 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(ox, sy) },
      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(ox, oy) },
      Vertex { pos: vec3((1 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(sx, oy) },
    ],
    Side::Back => [
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, oy) },
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, oy) },
      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },

      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (1 + k) as f32), uv: Vec2::new(sx, sy) },
      Vertex { pos: vec3((0 + i) as f32, (1 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, sy) },
      Vertex { pos: vec3((0 + i) as f32, (0 + j) as f32, (0 + k) as f32), uv: Vec2::new(ox, oy) },
    ],
  };
}