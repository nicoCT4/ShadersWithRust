use nalgebra_glm::{Vec2, Vec3};
use crate::color::Color;

pub struct Fragment {
   pub position: Vec2,
   pub color: Color,
   pub depth: f32,
   pub normal: Vec3,
   pub vertex_position: Vec3,
   pub intensity: f32,
}

impl Fragment {
   pub fn new(x: f32, y: f32, color: Color, depth: f32) -> Self {
      Fragment {
         position: Vec2::new(x, y),
         color,
         depth,
         normal: Vec3::new(0.0, 0.0, 0.0),
         vertex_position: Vec3::new(0.0, 0.0, 0.0),
         intensity: 1.0,
      }
   }

   pub fn new_with_data(
      x: f32,
      y: f32,
      color: Color,
      depth: f32,
      normal: Vec3,
      vertex_position: Vec3,
      intensity: f32,
   ) -> Self {
      Fragment {
         position: Vec2::new(x, y),
         color,
         depth,
         normal,
         vertex_position,
         intensity,
      }
   }
}