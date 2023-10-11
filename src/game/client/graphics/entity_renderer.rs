use glam::Vec4;
use wgpu::RenderPass;
use crate::game::client::graphics::chunk_renderer::block_face;
use crate::game::client::graphics::entity_model::EntityModel;
use crate::graphics::drawable::Drawable;
use crate::graphics::mesh::InstancedMesh;
use crate::graphics::vertex::Vertex;
use crate::graphics::bindable::Bindable;
use crate::graphics::context::Graphics;
use crate::graphics::texture::Texture;
use crate::util::side::Side;

pub type EntityMesh = InstancedMesh<Vertex, EntityModel>;

pub struct EntityRenderer {
  pub texture: Texture,
  pub entities_mesh : EntityMesh,
}

impl EntityRenderer {
  pub fn new(graphics: &Graphics) -> Self {
    return Self {
      texture: Texture::from_image(image::load_from_memory(include_bytes!("../../../../res/player.png")).unwrap().flipv(), graphics),
      entities_mesh: InstancedMesh::new(graphics, [
        block_face(Side::Top, 0, 0, 0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        block_face(Side::Bottom, 0, 0, 0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        block_face(Side::Front, 0, 0, 0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        block_face(Side::Back, 0, 0, 0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        block_face(Side::Left, 0, 0, 0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
        block_face(Side::Right, 0, 0, 0, Vec4::new(0.0, 0.0, 1.0, 1.0)),
      ].concat(), vec![]),
    };
  }

  pub fn render<'this: 'pass, 'pass>(&'this self, render_pass: &mut RenderPass<'pass>, index: u32) {
    self.texture.bind(render_pass, index);
    self.entities_mesh.draw(render_pass);
  }
}
