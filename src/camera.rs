use nalgebra_glm::{Vec3, rotate_vec3};
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

  pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
    let forward = (self.center - self.eye).normalize();
    let right = forward.cross(&self.up).normalize();
    let up = right.cross(&forward).normalize();

    let rotated = 
    vector.x * right +
    vector.y * up +
    - vector.z * forward;

    rotated.normalize()
  }

  pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
    let radius_vector = self.eye - self.center;
    let radius = radius_vector.magnitude();

    let current_yaw = radius_vector.z.atan2(radius_vector.x);

    let radius_xz = (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
    let current_pitch = (-radius_vector.y).atan2(radius_xz);

    let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
    let new_pitch = (current_pitch + delta_pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

    let new_eye = self.center + Vec3::new(
      radius * new_yaw.cos() * new_pitch.cos(),
      -radius * new_pitch.sin(),
      radius * new_yaw.sin() * new_pitch.cos()
    );

    self.eye = new_eye;
    self.has_changed = true;
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

  pub fn check_if_changed(&mut self) -> bool {
    if self.has_changed {
      self.has_changed = false;
      true
    } else {
      false
    }
  }

  fn update_center(&mut self) {
    let forward = self.get_forward();
    self.center = self.eye + forward;
  }

  pub fn move_forward(&mut self, amount: f32) {
    let forward = self.get_forward();
    self.eye += forward * amount;
    self.update_center();
  }

  pub fn move_up(&mut self, amount: f32) {
    self.eye += self.get_up() * amount;
    self.update_center();
  }

  pub fn rotate_yaw(&mut self, angle: f32) {
    self.yaw += angle;
    self.update_center();
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

  pub fn get_up(&self) -> Vec3 {
    self.up
  }

  pub fn get_right(&self) -> Vec3 {
    self.get_forward().cross(&self.up).normalize()
  }

  pub fn set_normal_view(&mut self) {
    self.eye = Vec3::new(0.0, 0.0, 10.0);
    self.center = Vec3::new(0.0, 0.0, 0.0);
    self.up = Vec3::new(0.0, 1.0, 0.0);
  }

  pub fn update_rotation(&mut self, delta_roll: f32, delta_pitch: f32, delta_yaw: f32) {
    self.roll += delta_roll;
    self.pitch += delta_pitch;
    self.yaw += delta_yaw;

    self.roll = self.roll.clamp(-PI / 4.0, PI / 4.0);
    self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);
  }

  pub fn reset_rotation(&mut self) {
    self.roll = 0.0;
    self.pitch = 0.0;
    self.yaw = 0.0;
  }

  pub fn move_camera(&mut self, delta_time: f32, delta_roll: f32, delta_pitch: f32, delta_yaw: f32) {
    // Evitar movimiento si está en vista de pájaro
    if self.bird_eye_active {
        return; // No hacer nada si está en vista de pájaro
    }

    let forward = self.get_forward();
    let right = self.get_right();
    
    self.eye += forward * delta_time;
    self.eye += right * self.roll * delta_time;
    self.eye.y += self.pitch * delta_time;

    self.yaw += delta_yaw * delta_time;
    self.pitch += delta_pitch * delta_time;

    if delta_roll == 0.0 && delta_pitch == 0.0 && delta_yaw == 0.0 {
        self.reset_rotation();
    } else {
        self.update_rotation(delta_roll, delta_pitch, delta_yaw);
    }
  }
  pub fn move_and_rotate(&mut self, delta_time: f32, inputs: (f32, f32, f32, f32, f32, f32)) {
    // inputs: (forward, right, up, roll, pitch, yaw)
    let (forward_input, right_input, up_input, delta_roll, delta_pitch, delta_yaw) = inputs;

    // Actualiza rotaciones
    self.yaw += delta_yaw * delta_time;
    self.pitch = (self.pitch + delta_pitch * delta_time).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
    self.roll = (self.roll + delta_roll * delta_time).clamp(-PI / 4.0, PI / 4.0);

    // Calcula nuevos ejes locales
    let (forward, right, up) = self.get_local_axes();

    // Aplica movimiento basado en los ejes locales
    self.eye += forward * forward_input * delta_time;
    self.eye += right * right_input * delta_time;
    self.eye += up * up_input * delta_time;

    // Movimiento adicional mientras rota
    self.eye += forward * (self.roll * delta_time); // Desplazamiento adicional basado en la rotación

    // Actualiza la posición del centro
    self.update_center();
}

pub fn get_local_axes(&self) -> (Vec3, Vec3, Vec3) {
    let forward = self.get_forward();
    let right = forward.cross(&self.up).normalize();
    let up = right.cross(&forward).normalize();
    (forward, right, up)
}
}
