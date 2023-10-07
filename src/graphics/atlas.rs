use std::collections::BTreeMap;
use std::hash::Hash;
use std::fmt::Debug;
use glam::{Vec2, Vec4};

use image::{DynamicImage, GenericImage, RgbaImage};
use rectangle_pack::{GroupedRectsToPlace, RectToInsert, TargetBin, volume_heuristic, contains_smallest_box, pack_rects, PackedLocation};
use wgpu::BindGroup;
use crate::graphics::context::Graphics;
use crate::graphics::layout::Layout;

use super::{texture::Texture, bindable::Bindable};

pub struct Atlas<RectToPlaceId: Debug + Hash + Clone + Eq + Ord + PartialOrd> {
  locations: BTreeMap<RectToPlaceId, (i32, PackedLocation)>,
  texture: Texture,
}

impl<RectToPlaceId: Debug + Hash + Clone + Copy + Eq + Ord + PartialOrd> Atlas<RectToPlaceId> {
  const ATLAS_SIZE: u32 = 2048;

  pub fn new(images: &[(RectToPlaceId, DynamicImage)], graphics: &Graphics) -> Self {
    let mut rects_to_place = GroupedRectsToPlace::<RectToPlaceId, i32>::new();
    for (id, image) in images {
      rects_to_place.push_rect(*id, None, RectToInsert::new(image.width(), image.height(), 255));
    }

    let mut target_bins = BTreeMap::new(); // TODO: use multiple atlases if needed
    target_bins.insert(0, TargetBin::new(Self::ATLAS_SIZE, Self::ATLAS_SIZE, 255));

    let rectangle_placements = pack_rects(
      &rects_to_place,
      &mut target_bins,
      &volume_heuristic,
      &contains_smallest_box
    ).unwrap();

    let mut texture_image = RgbaImage::new(Self::ATLAS_SIZE, Self::ATLAS_SIZE);
    let locations = rectangle_placements.packed_locations().clone();
    for (id, image) in images {
      let loc = locations[&id].1;
      texture_image.copy_from(image, loc.x(), loc.y()).unwrap();
    }

    let texture = Texture::from_image(DynamicImage::from(texture_image), graphics);
    return Self {
      texture,
      locations,
    };
  }

  pub fn uv(&self, id: &RectToPlaceId) -> Vec4 {
    let atlas_size = Self::ATLAS_SIZE as f32;
    let location = self.locations[id].1;

    let pos = Vec2::new(location.x() as f32 / atlas_size,
                        location.y() as f32 / atlas_size);

    let size = Vec2::new((location.x() + location.width())  as f32 / atlas_size,
                         (location.y() + location.height()) as f32 / atlas_size);

    return Vec4::new(pos.x, pos.y, size.x, size.y);
  }
}

impl<RectToPlaceId: Debug + Hash + Clone + Copy + Eq + Ord + PartialOrd> Bindable for Atlas<RectToPlaceId> {
  fn bind<'pass, 'uniform: 'pass>(&'uniform self, render_pass: &mut wgpu::RenderPass<'pass>, index: u32) {
    self.texture.bind(render_pass, index);
  }

  fn group(&self) -> &BindGroup {
    return self.texture.group();
  }
}

impl<RectToPlaceId: Debug + Hash + Clone + Copy + Eq + Ord + PartialOrd> Layout for Atlas<RectToPlaceId> {
  fn layout(&self) -> &wgpu::BindGroupLayout {
    return self.texture.layout();
  }
}