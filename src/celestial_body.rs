use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderType {
   Sun,
   RockyPlanet,
   GasGiant,
   Moon,
   RingedPlanet,
   Starfield,
}

pub struct CelestialBody {
   pub position: Vec3,
   pub rotation: Vec3,
   pub scale: f32,
   pub shader_type: ShaderType,
   pub rotation_speed: Vec3,
   pub orbit_speed: f32,
   pub orbit_radius: f32,
   pub orbit_angle: f32,
   pub time: f32,
}

impl CelestialBody {
   pub fn new(
      position: Vec3,
      scale: f32,
      shader_type: ShaderType,
   ) -> Self {
      CelestialBody {
         position,
         rotation: Vec3::new(0.0, 0.0, 0.0),
         scale,
         shader_type,
         rotation_speed: Vec3::new(0.0, 0.5, 0.0),
         orbit_speed: 0.0,
         orbit_radius: 0.0,
         orbit_angle: 0.0,
         time: 0.0,
      }
   }

   pub fn with_orbit(mut self, radius: f32, speed: f32) -> Self {
      self.orbit_radius = radius;
      self.orbit_speed = speed;
      self
   }

   pub fn with_rotation_speed(mut self, speed: Vec3) -> Self {
      self.rotation_speed = speed;
      self
   }

   pub fn update(&mut self, delta_time: f32) {
      // Update rotation
      self.rotation += self.rotation_speed * delta_time;

      // Update orbit
      if self.orbit_radius > 0.0 {
         self.orbit_angle += self.orbit_speed * delta_time;
         self.position.x = self.orbit_angle.cos() * self.orbit_radius;
         self.position.z = self.orbit_angle.sin() * self.orbit_radius;
      }

      // Update internal time for shader animations
      self.time += delta_time;
   }
}