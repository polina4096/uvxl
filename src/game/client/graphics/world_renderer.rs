use glam::{UVec2};
use wgpu::{CommandEncoder, TextureView};
use winit::dpi::PhysicalSize;
use crate::app::App;
use crate::game::client::graphics::chunk_renderer::ChunkRenderer;
use crate::game::client::graphics::chunk_model::ChunkModel;
use crate::game::world::BlockId;
use crate::graphics::atlas::Atlas;
use crate::graphics::bindable::Bindable;
use crate::graphics::camera::{Camera3D, Projection, ProjectionPerspective, TagCamera3D};
use crate::graphics::context::Graphics;
use crate::graphics::depth_buffer::DepthBuffer;
use crate::graphics::instance::Instance;
use crate::graphics::layout::Layout;
use crate::graphics::scene::Scene;
use crate::graphics::vertex::Vertex;

pub struct WorldRenderer {
  pub scene        : Scene<ProjectionPerspective, TagCamera3D>,
  pub pipeline     : wgpu::RenderPipeline,
  pub depth_buffer : DepthBuffer,

  pub chunk_renderer    : ChunkRenderer,
}

impl WorldRenderer {
  pub fn new(graphics: &Graphics) -> Self {

    let atlas = Atlas::new(&[
      (BlockId::TEST, image::load_from_memory(include_bytes!("../../../../res/test.png")).unwrap().flipv()),
      (BlockId::PANEL, image::load_from_memory(include_bytes!("../../../../res/panel.png")).unwrap().flipv()),
    ], graphics);

    let depth_buffer = DepthBuffer::new(graphics, UVec2::new(graphics.size.width, graphics.config.height));

    let scene = Scene::new(
      &graphics.device,
      ProjectionPerspective::new(
        graphics.size.width as f32,
        graphics.size.height as f32,
        85.0,
        0.1,
        1000.0
      ),
      Camera3D::default()
    );

    let shader = graphics.device.create_shader_module(wgpu::include_wgsl!("chunk.wgsl"));
    let render_pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[
        scene.layout(),
        atlas.layout(),
      ],
      push_constant_ranges: &[],
    });

    let pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label  : Some("Render Pipeline"),
      layout : Some(&render_pipeline_layout),

      vertex: wgpu::VertexState {
        module      : &shader,
        entry_point : "vertex_main",
        buffers     : &[
          Vertex::describe(),
          ChunkModel::describe(),
        ],
      },
      fragment: Some(wgpu::FragmentState {
        module      : &shader,
        entry_point : "fragment_main",
        targets     : &[Some(wgpu::ColorTargetState {
          format     : graphics.config.format,
          blend      : Some(wgpu::BlendState::REPLACE),
          write_mask : wgpu::ColorWrites::ALL,
        })],
      }),

      primitive: wgpu::PrimitiveState {
        topology           : wgpu::PrimitiveTopology::TriangleList,
        front_face         : wgpu::FrontFace::Ccw,
        cull_mode          : Some(wgpu::Face::Back),
        polygon_mode       : wgpu::PolygonMode::Fill, // Others require Features::NON_FILL_POLYGON_MODE
        unclipped_depth    : false,                   // Requires Features::DEPTH_CLIP_CONTROL
        conservative       : false,                   // Requires Features::CONSERVATIVE_RASTERIZATION
        strip_index_format : None,
      },

      multisample: wgpu::MultisampleState {
        count                     : 1,
        mask                      : !0,
        alpha_to_coverage_enabled : false,
      },

      depth_stencil: Some(wgpu::DepthStencilState {
        format              : DepthBuffer::DEPTH_FORMAT,
        depth_write_enabled : true,
        depth_compare       : wgpu::CompareFunction::Less,
        stencil             : wgpu::StencilState::default(),
        bias                : wgpu::DepthBiasState::default(),
      }),
      multiview: None,
    });

    let chunk_renderer = ChunkRenderer::new(atlas);

    return Self {
      pipeline,
      scene,
      depth_buffer,

      chunk_renderer,
    };
  }
}

impl WorldRenderer {
  pub fn render(&mut self, app: &mut App, view: &TextureView, encoder: &mut CommandEncoder) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Render Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: view,
        resolve_target: None,
        ops: wgpu::Operations {

          load: wgpu::LoadOp::Clear(wgpu::Color {
            r: 0.005,
            g: 0.005,
            b: 0.005,
            a: 1.000,
          }),
          store: true,
        },
      })],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &self.depth_buffer.view,
        depth_ops: Some(wgpu::Operations {
          load: wgpu::LoadOp::Clear(1.0),
          store: true,
        }),
        stencil_ops: None,
      }),
    });

    render_pass.set_pipeline(&self.pipeline);

    self.scene.update(&app.graphics.queue);
    self.scene.bind(&mut render_pass, 0);

    self.chunk_renderer.render(&mut render_pass, 1);
  }

  pub fn resize(&mut self, app: &mut App, size: PhysicalSize<u32>) {
    self.depth_buffer = DepthBuffer::new(&app.graphics, UVec2::new(size.width, size.height));
    self.scene.projection.resize(size.width as f32, size.height as f32);
  }
}