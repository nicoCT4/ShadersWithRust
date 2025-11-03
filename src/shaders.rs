use nalgebra_glm::Vec3;
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;
use crate::celestial_body::ShaderType;
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use std::f32::consts::PI;

// Estructura de Uniforms actualizada
pub struct Uniforms {
   pub model_matrix: nalgebra_glm::Mat4,
   pub view_matrix: nalgebra_glm::Mat4,
   pub projection_matrix: nalgebra_glm::Mat4,
   pub viewport_matrix: nalgebra_glm::Mat4,
   pub time: f32,
   pub noise: FastNoiseLite,
}

impl Uniforms {
   pub fn new(
      model_matrix: nalgebra_glm::Mat4,
      view_matrix: nalgebra_glm::Mat4,
      projection_matrix: nalgebra_glm::Mat4,
      viewport_matrix: nalgebra_glm::Mat4,
      time: f32,
   ) -> Self {
      let mut noise = FastNoiseLite::new();
      noise.set_noise_type(Some(NoiseType::OpenSimplex2));
      
      Uniforms {
         model_matrix,
         view_matrix,
         projection_matrix,
         viewport_matrix,
         time,
         noise,
      }
   }
}

// Vertex shader
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
   let position = nalgebra_glm::Vec4::new(
      vertex.position.x,
      vertex.position.y,
      vertex.position.z,
      1.0
   );

   let transformed = uniforms.projection_matrix 
      * uniforms.view_matrix 
      * uniforms.model_matrix 
      * position;

   let w = transformed.w;
   let ndc_position = nalgebra_glm::Vec4::new(
      transformed.x / w,
      transformed.y / w,
      transformed.z / w,
      1.0
   );

   let screen_position = uniforms.viewport_matrix * ndc_position;

   let model_mat3 = nalgebra_glm::Mat3::new(
      uniforms.model_matrix[0], uniforms.model_matrix[1], uniforms.model_matrix[2],
      uniforms.model_matrix[4], uniforms.model_matrix[5], uniforms.model_matrix[6],
      uniforms.model_matrix[8], uniforms.model_matrix[9], uniforms.model_matrix[10]
   );
   
   let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(nalgebra_glm::Mat3::identity());
   let transformed_normal = normal_matrix * vertex.normal;

   Vertex {
      position: vertex.position,
      normal: vertex.normal,
      tex_coords: vertex.tex_coords,
      color: vertex.color,
      transformed_position: nalgebra_glm::Vec3::new(
         screen_position.x,
         screen_position.y,
         screen_position.z
      ),
      transformed_normal,
   }
}

// Fragment shader dispatcher
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, shader_type: &ShaderType) -> Color {
   match shader_type {
      ShaderType::Sun => sun_shader(fragment, uniforms),
      ShaderType::RockyPlanet => rocky_planet_shader(fragment, uniforms),
      ShaderType::GasGiant => gas_giant_shader(fragment, uniforms),
      ShaderType::Moon => moon_shader(fragment, uniforms),
      ShaderType::RingedPlanet => gas_giant_shader(fragment, uniforms),
   }
}

// Utility functions for shaders
fn create_noise() -> FastNoiseLite {
   let mut noise = FastNoiseLite::new();
   noise.set_noise_type(Some(NoiseType::OpenSimplex2));
   noise
}

fn create_cloud_noise() -> FastNoiseLite {
   let mut noise = FastNoiseLite::new();
   noise.set_noise_type(Some(NoiseType::OpenSimplex2));
   noise.set_fractal_type(Some(FractalType::FBm));
   noise.set_fractal_octaves(Some(4));
   noise.set_fractal_lacunarity(Some(2.0));
   noise.set_fractal_gain(Some(0.5));
   noise
}

fn lerp_color(a: &Color, b: &Color, t: f32) -> Color {
   let t = t.clamp(0.0, 1.0);
   Color::new(
      (a.to_hex() as f32 * (1.0 - t) + (b.to_hex() >> 16) as f32 * t) as u8,
      ((a.to_hex() >> 8) as f32 * (1.0 - t) + ((b.to_hex() >> 8) & 0xFF) as f32 * t) as u8,
      ((a.to_hex() & 0xFF) as f32 * (1.0 - t) + (b.to_hex() & 0xFF) as f32 * t) as u8,
   )
}

fn blend_colors(base: &Color, overlay: &Color, factor: f32) -> Color {
   let factor = factor.clamp(0.0, 1.0);
   let base_hex = base.to_hex();
   let overlay_hex = overlay.to_hex();
   
   let r1 = ((base_hex >> 16) & 0xFF) as f32;
   let g1 = ((base_hex >> 8) & 0xFF) as f32;
   let b1 = (base_hex & 0xFF) as f32;
   
   let r2 = ((overlay_hex >> 16) & 0xFF) as f32;
   let g2 = ((overlay_hex >> 8) & 0xFF) as f32;
   let b2 = (overlay_hex & 0xFF) as f32;
   
   Color::new(
      (r1 * (1.0 - factor) + r2 * factor) as u8,
      (g1 * (1.0 - factor) + g2 * factor) as u8,
      (b1 * (1.0 - factor) + b2 * factor) as u8,
   )
}

// ============================================
// SUN SHADER - Estrella con efecto de plasma
// ============================================
fn sun_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
   let position = fragment.vertex_position;
   let time = uniforms.time;
   
   // Capa 1: Base de colores cálidos con gradiente radial
   let distance_from_center = (position.x * position.x + 
                              position.y * position.y + 
                              position.z * position.z).sqrt();
   
   let core_color = Color::from_hex(0xFFFF00);  // Amarillo brillante
   let mid_color = Color::from_hex(0xFF6600);   // Naranja
   let edge_color = Color::from_hex(0xFF0000);  // Rojo
   
   let base_color = if distance_from_center < 0.5 {
      let t = distance_from_center * 2.0;
      lerp_color(&core_color, &mid_color, t)
   } else {
      let t = (distance_from_center - 0.5) * 2.0;
      lerp_color(&mid_color, &edge_color, t)
   };
   
   // Capa 2: Plasma animado usando noise
   let plasma_zoom = 8.0;
   let plasma_speed = 0.3;
   let plasma_noise = uniforms.noise.get_noise_3d(
      position.x * plasma_zoom + time * plasma_speed,
      position.y * plasma_zoom,
      position.z * plasma_zoom + time * plasma_speed * 0.5,
   );
   
   let plasma_intensity = (plasma_noise + 1.0) * 0.5;
   let plasma_color = Color::from_hex(0xFFAA00);
   let with_plasma = blend_colors(&base_color, &plasma_color, plasma_intensity * 0.3);
   
   // Capa 3: Manchas solares (áreas más oscuras)
   let spot_zoom = 3.0;
   let spot_noise = uniforms.noise.get_noise_3d(
      position.x * spot_zoom,
      position.y * spot_zoom + time * 0.1,
      position.z * spot_zoom,
   );
   
   if spot_noise > 0.5 {
      let spot_factor = (spot_noise - 0.5) * 2.0;
      let dark_spot = Color::from_hex(0x994400);
      let with_spots = blend_colors(&with_plasma, &dark_spot, spot_factor * 0.4);
      
      // Capa 4: Brillo en los bordes (efecto corona)
      let edge_glow = (1.0 - distance_from_center).powf(3.0);
      let glow_color = Color::from_hex(0xFFFFAA);
      blend_colors(&with_spots, &glow_color, edge_glow * 0.3)
   } else {
      with_plasma
   }
}