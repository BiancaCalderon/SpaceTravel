use nalgebra_glm::{Vec3};
use std::f32::consts::PI;

pub struct Camera {
  pub eye: Vec3,
  pub center: Vec3,
  pub up: Vec3,
  pub has_changed: bool,
  pub bird_eye_active: bool,
  pub previous_state: Option<(Vec3, Vec3, f32, f32, f32)>,
  pub yaw: f32,
  pub roll: f32,
  pub pitch: f32,
}

impl Camera {
  pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
    Camera {
      eye,
      center,
      up,
      has_changed: true,
      bird_eye_active: false,
      previous_state: None,
      yaw: 0.0,
      roll: 0.0,
      pitch: 0.0,
    }
  }

  pub fn zoom(&mut self, delta: f32) {
    let direction = (self.center - self.eye).normalize();
    self.eye += direction * delta;
    self.has_changed = true;
  }

  pub fn move_center(&mut self, movement: Vec3) {
    self.center += movement;
    self.eye += movement;
  }

  pub fn update_center(&mut self) {
    let forward = self.get_forward();
    self.center = self.eye + forward;
  }

  pub fn rotate_pitch(&mut self, angle: f32) {
    self.pitch = (self.pitch + angle).clamp(-PI/2.0 + 0.1, PI/2.0 - 0.1);
    self.update_center();
  }

  pub fn set_bird_eye_view(&mut self) {
    self.eye = Vec3::new(0.0, 1200.0, 800.0);
    self.center = Vec3::new(0.0, 0.0, 0.0);
    self.up = Vec3::new(0.0, 1.0, 0.0);
    self.bird_eye_active = true;
    self.has_changed = true;
  }

  pub fn get_forward(&self) -> Vec3 {
    Vec3::new(
      self.yaw.cos() * self.pitch.cos(),
      self.pitch.sin(),
      self.yaw.sin() * self.pitch.cos(),
    ).normalize()
  }
}
