use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use glam::Vec3;
use winit::event::{ElementState, VirtualKeyCode};
use crate::graphics::camera::Camera;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct CameraController {
  amount_left: f32,
  amount_right: f32,
  amount_forward: f32,
  amount_backward: f32,
  amount_up: f32,
  amount_down: f32,
  rotate_horizontal: f32,
  rotate_vertical: f32,
  speed: f32,
  sensitivity: f32,
}

impl CameraController {
  pub fn new(speed: f32, sensitivity: f32) -> Self {
    Self {
      amount_left: 0.0,
      amount_right: 0.0,
      amount_forward: 0.0,
      amount_backward: 0.0,
      amount_up: 0.0,
      amount_down: 0.0,
      rotate_horizontal: 0.0,
      rotate_vertical: 0.0,
      speed,
      sensitivity,
    }
  }

  pub fn on_keyboard(&mut self, key: VirtualKeyCode, state: ElementState){
    let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
    match key {
      VirtualKeyCode::W => { self.amount_forward = amount; }
      VirtualKeyCode::S => { self.amount_backward = amount; }
      VirtualKeyCode::A => { self.amount_left = amount; }
      VirtualKeyCode::D => { self.amount_right = amount; }
      VirtualKeyCode::Space => { self.amount_up = amount; }
      VirtualKeyCode::LShift => { self.amount_down = amount; }
      _ => {},
    }
  }

  pub fn on_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
    self.rotate_horizontal += mouse_dx as f32;
    self.rotate_vertical   += mouse_dy as f32;
  }

  pub fn update_camera<T>(&mut self, camera: &mut Camera<T>, dt: Duration) {
    let dt = dt.as_secs_f32();

    // move forward/backward and left/right
    let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
    let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
    camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

    // move up/down; since we don't use roll, we can just modify the y coordinate directly
    camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

    // rotate
    camera.yaw   += ( self.rotate_horizontal) * self.sensitivity * dt;
    camera.pitch += (-self.rotate_vertical  ) * self.sensitivity * dt;

    // if process_mouse isn't called every frame, these values
    // will not get set to zero, and the camera will rotate
    // when moving in a non cardinal direction
    self.rotate_horizontal = 0.0;
    self.rotate_vertical = 0.0;

    // keep the camera's angle from going too high/low
    if camera.pitch < -SAFE_FRAC_PI_2 {
      camera.pitch = -SAFE_FRAC_PI_2;
    } else if camera.pitch > SAFE_FRAC_PI_2 {
      camera.pitch = SAFE_FRAC_PI_2;
    }
  }
}