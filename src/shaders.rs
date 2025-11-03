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

// ============================================
// ROCKY PLANET SHADER - Planeta tipo Tierra/Marte
// ============================================
fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
   let position = fragment.vertex_position;
   let time = uniforms.time;
   
   // Capa 1: Continentes y océanos base
   let continent_zoom = 4.0;
   let continent_noise = uniforms.noise.get_noise_3d(
      position.x * continent_zoom,
      position.y * continent_zoom,
      position.z * continent_zoom,
   );
   
   // Colores base
   let ocean_color = Color::from_hex(0x1a4d7a);      // Azul océano
   let land_color = Color::from_hex(0x3d8c40);       // Verde tierra
   let mountain_color = Color::from_hex(0x8b7355);   // Marrón montañas
   let snow_color = Color::from_hex(0xf0f0f0);       // Blanco nieve
   
   let mut base_color = if continent_noise > 0.0 {
      // Tierra
      if continent_noise > 0.5 {
         // Montañas
         let t = (continent_noise - 0.5) * 2.0;
         lerp_color(&land_color, &mountain_color, t)
      } else {
         land_color
      }
   } else {
      // Océano
      let depth = continent_noise.abs();
      let deep_ocean = Color::from_hex(0x0a2342);
      lerp_color(&ocean_color, &deep_ocean, depth)
   };
   
   // Capa 2: Detalles de terreno (vegetación, desiertos)
   let detail_zoom = 10.0;
   let detail_noise = uniforms.noise.get_noise_3d(
      position.x * detail_zoom + 100.0,
      position.y * detail_zoom,
      position.z * detail_zoom,
   );
   
   if continent_noise > 0.0 {
      let detail_color = if detail_noise > 0.3 {
         Color::from_hex(0xc2b280) // Desierto
      } else {
         Color::from_hex(0x228b22) // Bosque
      };
      base_color = blend_colors(&base_color, &detail_color, detail_noise.abs() * 0.3);
   }
   
   // Capa 3: Casquetes polares
   let polar_threshold = 0.7;
   if position.y.abs() > polar_threshold {
      let polar_factor = (position.y.abs() - polar_threshold) / (1.0 - polar_threshold);
      base_color = blend_colors(&base_color, &snow_color, polar_factor);
   }
   
   // Capa 4: Nubes animadas
   let mut cloud_noise = create_cloud_noise();
   let cloud_zoom = 6.0;
   let cloud_speed = 0.2;
   let cloud_value = cloud_noise.get_noise_3d(
      position.x * cloud_zoom + time * cloud_speed,
      position.y * cloud_zoom,
      position.z * cloud_zoom,
   );
   
   if cloud_value > 0.5 {
      let cloud_factor = (cloud_value - 0.5) * 2.0;
      let cloud_color = Color::from_hex(0xffffff);
      base_color = blend_colors(&base_color, &cloud_color, cloud_factor * 0.6);
   }
   
   // Aplicar iluminación
   let light_intensity = fragment.intensity;
   base_color * light_intensity
}

// ============================================
// GAS GIANT SHADER - Planeta tipo Júpiter/Saturno
// ============================================
fn gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
   let position = fragment.vertex_position;
   let time = uniforms.time;
   
   // Capa 1: Bandas horizontales base
   let band_frequency = 15.0;
   let band_position = position.y * band_frequency;
   let band_noise = uniforms.noise.get_noise_2d(
      band_position,
      time * 0.1,
   );
   
   // Colores de las bandas
   let color1 = Color::from_hex(0xd4a574); // Beige claro
   let color2 = Color::from_hex(0xc17f4a); // Marrón claro
   let color3 = Color::from_hex(0x8b6239); // Marrón oscuro
   let color4 = Color::from_hex(0xe6c9a8); // Crema
   
   let band_value = (band_position.sin() + 1.0) * 0.5;
   let base_color = if band_value < 0.25 {
      let t = band_value * 4.0;
      lerp_color(&color1, &color2, t)
   } else if band_value < 0.5 {
      let t = (band_value - 0.25) * 4.0;
      lerp_color(&color2, &color3, t)
   } else if band_value < 0.75 {
      let t = (band_value - 0.5) * 4.0;
      lerp_color(&color3, &color4, t)
   } else {
      let t = (band_value - 0.75) * 4.0;
      lerp_color(&color4, &color1, t)
   };
   
   // Capa 2: Turbulencias en las bandas
   let turbulence_zoom = 8.0;
   let turbulence_noise = uniforms.noise.get_noise_3d(
      position.x * turbulence_zoom + time * 0.3,
      position.y * turbulence_zoom * 0.5,
      position.z * turbulence_zoom,
   );
   
   let turbulent_offset = turbulence_noise * 0.3;
   let turbulent_band = ((band_position + turbulent_offset).sin() + 1.0) * 0.5;
   
   let turbulence_color = if turbulent_band > 0.6 {
      Color::from_hex(0xa0785a)
   } else {
      Color::from_hex(0xe8d4b8)
   };
   
   let with_turbulence = blend_colors(&base_color, &turbulence_color, turbulence_noise.abs() * 0.4);
   
   // Capa 3: Gran Mancha Roja (o equivalente)
   let spot_center_x = 0.3;
   let spot_center_y = 0.2;
   let distance_to_spot = ((position.x - spot_center_x).powi(2) + 
                           (position.y - spot_center_y).powi(2)).sqrt();
   
   if distance_to_spot < 0.3 {
      let spot_noise = uniforms.noise.get_noise_3d(
         position.x * 5.0 + time * 0.05,
         position.y * 5.0,
         position.z * 5.0,
      );
      
      let spot_factor = (1.0 - distance_to_spot / 0.3) * ((spot_noise + 1.0) * 0.5);
      let spot_color = Color::from_hex(0xc74440); // Rojo
      let with_spot = blend_colors(&with_turbulence, &spot_color, spot_factor * 0.7);
      
      // Capa 4: Detalles finos y remolinos
      let detail_zoom = 20.0;
      let detail_noise = uniforms.noise.get_noise_3d(
         position.x * detail_zoom - time * 0.2,
         position.y * detail_zoom,
         position.z * detail_zoom,
      );
      
      let detail_color = Color::from_hex(0xf5e6d3);
      let final_color = blend_colors(&with_spot, &detail_color, detail_noise.abs() * 0.2);
      
      final_color * fragment.intensity
   } else {
      with_turbulence * fragment.intensity
   }
}

// ============================================
// MOON SHADER - Luna con cráteres
// ============================================
fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
   let position = fragment.vertex_position;
   
   // Capa 1: Color base grisáceo
   let base_color = Color::from_hex(0x9b9b9b); // Gris medio
   let dark_color = Color::from_hex(0x6b6b6b);  // Gris oscuro
   let light_color = Color::from_hex(0xc5c5c5); // Gris claro
   
   // Capa 2: Variaciones de terreno
   let terrain_zoom = 8.0;
   let terrain_noise = uniforms.noise.get_noise_3d(
      position.x * terrain_zoom,
      position.y * terrain_zoom,
      position.z * terrain_zoom,
   );
   
   let terrain_color = if terrain_noise > 0.0 {
      lerp_color(&base_color, &light_color, terrain_noise)
   } else {
      lerp_color(&base_color, &dark_color, -terrain_noise)
   };
   
   // Capa 3: Cráteres
   let crater_zoom = 10.0;
   let crater_noise = uniforms.noise.get_noise_3d(
      position.x * crater_zoom + 500.0,
      position.y * crater_zoom,
      position.z * crater_zoom,
   );
   
   let mut final_color = terrain_color;
   
   if crater_noise > 0.7 {
      let crater_depth = (crater_noise - 0.7) / 0.3;
      let crater_color = Color::from_hex(0x4a4a4a); // Muy oscuro
      final_color = blend_colors(&final_color, &crater_color, crater_depth * 0.8);
   }
   
   // Capa 4: Detalles de superficie
   let detail_zoom = 25.0;
   let detail_noise = uniforms.noise.get_noise_3d(
      position.x * detail_zoom,
      position.y * detail_zoom,
      position.z * detail_zoom,
   );
   
   let detail_color = Color::from_hex(0xb0b0b0);
   final_color = blend_colors(&final_color, &detail_color, detail_noise.abs() * 0.15);
   
   // Aplicar iluminación más dramática para la luna
   let light_intensity = fragment.intensity.powf(0.8); // Más contraste
   final_color * light_intensity
}