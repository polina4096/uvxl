use std::marker::PhantomData;
use glam::{Mat4, Vec3, Quat, vec3};

pub trait Transformation {
  fn apply(&self) -> Mat4;
}

// Camera
pub struct TagCamera2D;
pub struct TagCamera3D;

pub type Camera2D = Camera<TagCamera2D>;
pub type Camera3D = Camera<TagCamera3D>;

pub struct Camera<T> {
  pub position: Vec3,
  pub scale: Vec3,
  // pub rotation: Quat,
  pub pitch: f32,
  pub yaw: f32,
  pub _1: PhantomData<T>,
}

impl<T> Camera<T> {
  pub fn new(position: Vec3, scale: Vec3, rotation: Quat) -> Self {
    return Self {
      position,
      scale,
      // rotation,
      pitch: 0.0,
      yaw: 0.0,

      _1: Default::default(),
    };
  }
}

impl<T> Default for Camera<T> {
  fn default() -> Self {
    return Self {
      position: vec3(0.0, 0.0, 0.0),
      scale: vec3(1.0, 1.0, 1.0),
      // rotation: Quat::default(),
      pitch: 0.0,
      yaw: 0.0,
      _1: Default::default(),
    };
  }
}

impl Transformation for Camera<TagCamera2D> {
  fn apply(&self) -> Mat4 {
    let model = Mat4::from_scale(vec3(self.scale.x, self.scale.y, self.scale.z))
              * Mat4::from_translation(self.position);
              // * Mat4::from_quat(self.rotation);

    return model;
  }
}

impl Transformation for Camera<TagCamera3D> {
  fn apply(&self) -> Mat4 {
    let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
    let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

    return Mat4::look_to_rh(
      self.position,
      vec3(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
      Vec3::Y
    );
  }
}


// Projection
pub trait Projection: Transformation {
  fn resize(&mut self, width: f32, height: f32);
}

/* Orthographic projection matrix */
pub struct ProjectionOrthographic {
  width: f32,
  height: f32,
  znear: f32,
  zfar: f32,
}

impl ProjectionOrthographic {
  pub fn new(width: u32, height: u32, znear: f32, zfar: f32) -> Self {
    return Self {
      width: width as f32,
      height: height as f32,
      znear: znear,
      zfar: zfar,
    };
  }
}

impl Projection for ProjectionOrthographic {
  fn resize(&mut self, width: f32, height: f32) {
    self.width = width;
    self.height = height;
  }
}

impl Transformation for ProjectionOrthographic {
  fn apply(&self) -> Mat4 {
    return Mat4::orthographic_lh(0.0, self.width, self.height, 0.0, self.znear, self.zfar);
  }
}

/* Perspective projection matrix */
pub struct ProjectionPerspective {
  aspect: f32,
  fov: f32,
  znear: f32,
  zfar: f32,
}

impl ProjectionPerspective {
  pub fn new(width: f32, height: f32, fov: f32, znear: f32, zfar: f32) -> Self {
    return Self {
      aspect: width / height,
      fov: fov,
      znear: znear,
      zfar: zfar,
    };
  }
}

impl Projection for ProjectionPerspective {
  fn resize(&mut self, width: f32, height: f32) {
    self.aspect = width / height;
  }
}

impl Transformation for ProjectionPerspective {
  fn apply(&self) -> Mat4 {
    return Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.znear, self.zfar);
  }
}