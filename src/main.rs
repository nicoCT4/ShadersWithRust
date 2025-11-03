use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod celestial_body;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use camera::Camera;
use shaders::{vertex_shader, fragment_shader, Uniforms};
use celestial_body::{CelestialBody, ShaderType};


pub struct RenderContext {
    framebuffer: Framebuffer,
    camera: Camera,
    bodies: Vec<CelestialBody>,
    current_body_index: usize,
    time: f32,
}

impl RenderContext {
    fn new(width: usize, height: usize) -> Self {
        let mut bodies = Vec::new();
        
        // Sol en el centro
        bodies.push(
            CelestialBody::new(
                Vec3::new(0.0, 0.0, 0.0),
                1.5,
                ShaderType::Sun,
            )
            .with_rotation_speed(Vec3::new(0.0, 0.1, 0.0))
        );
        
        // Planeta rocoso (tipo Tierra)
        bodies.push(
            CelestialBody::new(
                Vec3::new(3.0, 0.0, 0.0),
                0.5,
                ShaderType::RockyPlanet,
            )
            .with_orbit(3.0, 0.5)
            .with_rotation_speed(Vec3::new(0.0, 0.5, 0.0))
        );
        
        // Luna del planeta rocoso
        bodies.push(
            CelestialBody::new(
                Vec3::new(3.8, 0.0, 0.0), // Cerca del planeta rocoso
                0.15, // Más pequeña que el planeta
                ShaderType::Moon,
            )
            .with_orbit(0.8, 1.2) // Órbita alrededor del planeta rocoso
            .with_rotation_speed(Vec3::new(0.0, 0.3, 0.0))
        );
        
        // Gigante gaseoso (tipo Júpiter)
        bodies.push(
            CelestialBody::new(
                Vec3::new(6.0, 0.0, 0.0),
                0.8, 
                ShaderType::GasGiant,
            )
            .with_orbit(6.0, 0.25)
            .with_rotation_speed(Vec3::new(0.0, 0.8, 0.0))
        );
        
        // Temporalmente removido el campo de estrellas para debug

        RenderContext {
            framebuffer: Framebuffer::new(width, height),
            camera: Camera::new(
                Vec3::new(0.0, 3.0, 8.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            bodies,
            current_body_index: 0,
            time: 0.0,
        }
    }
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,   1.0, 0.0,
        0.0,    0.0,   0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

fn create_view_matrix(camera: &Camera) -> Mat4 {
    look_at(&camera.eye, &camera.center, &camera.up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    shader_type: &ShaderType,
) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Processing Stage
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        
        if x < framebuffer.width && y < framebuffer.height {
            // Apply fragment shader
            let shaded_color = fragment_shader(&fragment, uniforms, shader_type);
            let color = shaded_color.to_hex();
            
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut window = Window::new(
        "Space Renderer - Solar System",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    let mut context = RenderContext::new(framebuffer_width, framebuffer_height);
    context.framebuffer.set_background_color(0x000011);

    // Load sphere model
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load sphere model");
    let vertex_arrays = obj.get_vertex_array();

    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

    let mut last_frame_time = std::time::Instant::now();

    println!("Controls:");
    println!("  Arrow Keys: Orbit camera");
    println!("  W/S: Zoom in/out");
    println!("  1: Focus on Sun");
    println!("  2: Focus on Mars (Rocky Planet)");
    println!("  3: Focus on Moon");
    println!("  4: Focus on Jupiter (Gas Giant)");
    println!("  Space: Toggle orbit animation");
    println!("  ESC: Exit");

    let mut orbit_enabled = true;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let current_time = std::time::Instant::now();
        let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = current_time;

        // Handle input
        handle_input(&window, &mut context, &mut orbit_enabled);

        // Update bodies
        if orbit_enabled {
            context.time += delta_time;
            
            // Guardamos las posiciones que necesitamos antes de modificar
            let planet_pos = if context.bodies.len() > 1 {
                context.bodies[1].position
            } else {
                Vec3::new(0.0, 0.0, 0.0)
            };
            

            
            // Actualizamos los cuerpos
            for i in 0..context.bodies.len() {
                match i {
                    0 => { // Sol - solo rotación
                        let rotation_speed = context.bodies[i].rotation_speed;
                        context.bodies[i].rotation += rotation_speed * delta_time;
                        context.bodies[i].time += delta_time;
                    },
                    1 => { // Planeta rocoso - órbita normal
                        context.bodies[i].update(delta_time);
                    },
                    2 => { // Luna - orbita alrededor del planeta rocoso
                        let orbit_speed = context.bodies[i].orbit_speed;
                        let orbit_radius = context.bodies[i].orbit_radius;
                        let rotation_speed = context.bodies[i].rotation_speed;
                        
                        context.bodies[i].orbit_angle += orbit_speed * delta_time;
                        context.bodies[i].position.x = planet_pos.x + context.bodies[i].orbit_angle.cos() * orbit_radius;
                        context.bodies[i].position.z = planet_pos.z + context.bodies[i].orbit_angle.sin() * orbit_radius;
                        context.bodies[i].rotation += rotation_speed * delta_time;
                        context.bodies[i].time += delta_time;
                    },
                    3 => { // Gigante gaseoso - órbita normal
                        context.bodies[i].update(delta_time);
                    },
                    _ => {
                        context.bodies[i].update(delta_time);
                    }
                }
            }
        }

        context.framebuffer.clear();

        let view_matrix = create_view_matrix(&context.camera);

        // Render all bodies
        for body in &context.bodies {
            let model_matrix = create_model_matrix(
                body.position,
                body.scale,
                body.rotation,
            );

            let uniforms = Uniforms::new(
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                body.time,
            );

            render(
                &mut context.framebuffer,
                &uniforms,
                &vertex_arrays,
                &body.shader_type,
            );
        }

        window
            .update_with_buffer(
                &context.framebuffer.buffer,
                framebuffer_width,
                framebuffer_height,
            )
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn handle_input(window: &Window, context: &mut RenderContext, orbit_enabled: &mut bool) {
    let _movement_speed = 0.5;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 0.5;

    // Camera orbit
    if window.is_key_down(Key::Left) {
        context.camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        context.camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Up) {
        context.camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::Down) {
        context.camera.orbit(0.0, rotation_speed);
    }

    // Camera zoom
    if window.is_key_down(Key::W) {
        context.camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::S) {
        context.camera.zoom(-zoom_speed);
    }

    // Switch between bodies (just for camera focus, render all)
    if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
        context.current_body_index = 0;
        context.camera.center = context.bodies[0].position;
        println!("Focusing on: Sun");
    }
    if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
        context.current_body_index = 1;
        context.camera.center = context.bodies[1].position;
        println!("Focusing on: Rocky Planet (Mars)");
    }
    if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
        context.current_body_index = 2;
        context.camera.center = context.bodies[2].position;
        println!("Focusing on: Moon");
    }
    if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) {
        context.current_body_index = 3;
        context.camera.center = context.bodies[3].position;
        println!("Focusing on: Gas Giant (Jupiter)");
    }

    // Toggle orbit animation
    if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
        *orbit_enabled = !*orbit_enabled;
        println!("Orbit animation: {}", if *orbit_enabled { "ON" } else { "OFF" });
    }
}