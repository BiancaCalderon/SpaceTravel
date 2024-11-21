use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod planet;
//mod normal_map;
mod skybox;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use planet::PlanetType;
//use normal_map::init_normal_map;
use skybox::Skybox;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite
}

pub struct CelestialBody {
    position: Vec3,
    scale: f32,
    rotation: Vec3,
    shader_type: PlanetType,
}

fn create_noise() -> FastNoiseLite {
    create_cloud_noise() 
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_cell_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_frequency(Some(0.1));
    noise
}

fn create_ground_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    
    // Use FBm fractal type to layer multiple octaves of noise
    noise.set_noise_type(Some(NoiseType::Cellular)); // Cellular noise for cracks
    noise.set_fractal_type(Some(FractalType::FBm));  // Fractal Brownian Motion
    noise.set_fractal_octaves(Some(5));              // More octaves = more detail
    noise.set_fractal_lacunarity(Some(2.0));         // Lacunarity controls frequency scaling
    noise.set_fractal_gain(Some(0.5));               // Gain controls amplitude scaling
    noise.set_frequency(Some(0.05));                 // Lower frequency for larger features

    noise
}

fn create_lava_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(42);
    
    // Use FBm for multi-layered noise, giving a "turbulent" feel
    noise.set_noise_type(Some(NoiseType::Perlin));  // Perlin noise for smooth, natural texture
    noise.set_fractal_type(Some(FractalType::FBm)); // FBm for layered detail
    noise.set_fractal_octaves(Some(6));             // High octaves for rich detail
    noise.set_fractal_lacunarity(Some(2.0));        // Higher lacunarity = more contrast between layers
    noise.set_fractal_gain(Some(0.5));              // Higher gain = more influence of smaller details
    noise.set_frequency(Some(0.002));                // Low frequency = large features
    
    noise
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
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
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


fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
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

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], planet_type: &PlanetType) {
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
            let shaded_color = fragment_shader(&fragment, &uniforms, planet_type);
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

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Renderer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    )
        .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x000000);

    // model position
    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Vec3::new(0.0, 0.0, 0.0);
    let scale = 1.0f32;

    // camera parameters
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    let obj = Obj::load("assets/models/smooth_sphere.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array(); 
    let mut time = 0;
    let skybox = Skybox::new(1000);

    let noise = create_noise();
    let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
    let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
    let mut uniforms = Uniforms { 
        model_matrix: Mat4::identity(), 
        view_matrix: Mat4::identity(), 
        projection_matrix, 
        viewport_matrix, 
        time: 0, 
        noise
    };

    let mut celestial_bodies = vec![
        CelestialBody {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: 2.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Sun,
        },
        CelestialBody {
            position: Vec3::new(-4.0, 0.0, 0.0),
            scale: 0.3,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Asteroid,
        },
        CelestialBody {
            position: Vec3::new(6.0, 0.0, 0.0),
            scale: 0.4,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::RockyPlanet,
        },
        CelestialBody {
            position: Vec3::new(12.0, 0.0, 0.0),
            scale: 0.6,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Earth,
        },
        CelestialBody {
            position: Vec3::new(18.0, 0.0, 0.0),
            scale: 0.5,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::CrystalPlanet,
        },
        CelestialBody {
            position: Vec3::new(24.0, 0.0, 0.0),
            scale: 0.7,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::FirePlanet,
        },
        CelestialBody {
            position: Vec3::new(30.0, 0.0, 0.0),
            scale: 1.0,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::WaterPlanet,
        },
        CelestialBody {
            position: Vec3::new(36.0, 0.0, 0.0),
            scale: 0.8,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::CloudPlanet,
        },
        CelestialBody {
            position: Vec3::new(12.0, 0.0, 2.0),
            scale: 0.2,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            shader_type: PlanetType::Moon,
        },
    ];

    // Definir los radios de órbita para cada planeta
    let planet_orbit_radii = vec![
        8.0, // Radio para el primer planeta
        12.0, // Radio para el segundo planeta
        16.0, // Radio para el tercer planeta
        20.0, // Radio para el cuarto planeta (Tierra)
        24.0, // Radio para el quinto planeta
        28.0, // Radio para el sexto planeta
        32.0, // Radio para el séptimo planeta
        36.0, // Radio para el octavo planeta
        6.0,  // Radio para el asteroide (más cerca del sol)
    ];

    // Velocidad de órbita base
    let base_orbit_speed = 0.01; // Velocidad base para el planeta más cercano

    let mut planet_angles: Vec<f32> = vec![0.0; celestial_bodies.len()]; // Ángulos iniciales de los planetas

    // Definir un ángulo para la luna
    let mut moon_angle: f32 = 0.0; // Ángulo inicial de la luna
    let moon_orbit_radius = 0.5; // Radio de órbita de la luna alrededor de la Tierra

    // Definir colores para cada cuerpo celeste (sin contar el sol)
    let colors = vec![
        0xFF0000, // Rojo para el primer planeta
        0x00FF00, // Verde para el segundo planeta
        0x0000FF, // Azul para el tercer planeta
        0xFFFF00, // Amarillo para el cuarto planeta (Tierra)
        0xFF00FF, // Magenta para el quinto planeta
        0x00FFFF, // Cian para el sexto planeta
        0xFFA500, // Naranja para el séptimo planeta
        0x800080, // Púrpura para el octavo planeta
        0xFFFFFF, // Blanco para el asteroide
    ];

    // Crear un vector para almacenar las posiciones anteriores de cada cuerpo celeste
    let mut previous_positions: Vec<Vec<Vec3>> = vec![vec![]; celestial_bodies.len()];

    // Obtener la posición de la Tierra antes de modificar celestial_bodies
    let earth_position = celestial_bodies.iter()
        .find(|b| b.shader_type == PlanetType::Earth)
        .map(|b| b.position)
        .unwrap_or(Vec3::new(0.0, 0.0, 0.0)); // Valor por defecto en caso de que no se encuentre

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;

        handle_input(&window, &mut camera);

        framebuffer.clear();

        skybox.render(&mut framebuffer, &uniforms, camera.eye);

        // Actualizar la posición de los planetas en órbita
        for (i, body) in celestial_bodies.iter_mut().enumerate() {
            if body.shader_type == PlanetType::Sun {
                continue; // El sol no se mueve
            }

            // Calcular la posición en órbita
            let orbit_radius = planet_orbit_radii[i]; // Usar el radio de órbita correspondiente
            let angle = planet_angles[i]; // Usar el ángulo correspondiente

            // Calcular la velocidad de órbita en función del radio
            let orbit_speed = base_orbit_speed / orbit_radius; // Planetas más lejanos se mueven más lento

            // Guardar la posición anterior
            previous_positions[i].push(body.position);

            body.position.x = orbit_radius * angle.cos(); // Posición en X
            body.position.z = orbit_radius * angle.sin(); // Posición en Z

            // Incrementar el ángulo para simular la órbita
            planet_angles[i] += orbit_speed; // Incrementar el ángulo de órbita

            // Si el cuerpo es la luna, ajustar su posición respecto a la Tierra
            if body.shader_type == PlanetType::Moon {
                // Calcular la posición de la luna en función de la posición de la Tierra
                body.position = earth_position + Vec3::new(moon_orbit_radius * moon_angle.cos(), 0.0, moon_orbit_radius * moon_angle.sin());
            }

            // Limitar el número de posiciones almacenadas para evitar un uso excesivo de memoria
            if previous_positions[i].len() > 100 {
                previous_positions[i].remove(0);
            }
        }

        // Actualizar el ángulo de la luna
        moon_angle += 0.05; // Incrementar el ángulo de la luna para simular su órbita

        // Renderizar cada cuerpo celeste
        for (i, body) in celestial_bodies.iter().enumerate() {
            uniforms.model_matrix = create_model_matrix(
                body.position,
                body.scale,
                body.rotation + Vec3::new(0.0, time as f32 * 0.01, 0.0)
            );
            uniforms.view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
            uniforms.time = time;

            render(&mut framebuffer, &uniforms, &vertex_arrays, &body.shader_type);

            // Dibujar la estela
            let color = colors[i]; // Obtener el color correspondiente
            for j in 0..previous_positions[i].len() - 1 {
                if j + 1 < previous_positions[i].len() {
                    framebuffer.line(previous_positions[i][j], previous_positions[i][j + 1]);
                }
            }
        }

        // Renderizar las órbitas de los planetas
        for (i, body) in celestial_bodies.iter().enumerate() {
            if body.shader_type == PlanetType::Sun {
                continue; // No renderizar la órbita del sol
            }
            let orbit_radius = planet_orbit_radii[i]; // Usar el radio de órbita correspondiente
            let color = colors[i]; // Obtener el color correspondiente para la órbita
            render_orbit(&mut framebuffer, orbit_radius, 100, color); // Asegúrate de que esta línea esté correcta
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}

fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 0.5;
    let rotation_speed = PI / 50.0;
    let zoom_speed = 1.0;

    // Rotación de la cámara (mirando arriba/abajo)
    if window.is_key_down(Key::Up) {
        camera.rotate_pitch(-rotation_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.rotate_pitch(rotation_speed);
    }

    // Movimiento WASD (adelante, izquierda, atrás, derecha)
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::W) {
        movement.z -= movement_speed; // Mover hacia adelante
    }
    if window.is_key_down(Key::S) {
        movement.z += movement_speed; // Mover hacia atrás
    }
    if window.is_key_down(Key::A) {
        movement.x -= movement_speed; // Mover a la izquierda
    }
    if window.is_key_down(Key::D) {
        movement.x += movement_speed; // Mover a la derecha
    }

    // Aplicar movimiento solo si hay entrada
    if movement.magnitude() > 0.0 {
        camera.move_center(movement);
    }

    // Movimiento vertical (Q para subir, E para bajar)
    if window.is_key_down(Key::Q) {
        camera.move_up(movement_speed);
    }
    if window.is_key_down(Key::E) {
        camera.move_up(-movement_speed);
    }

    // Zoom (1 para acercar, 2 para alejar)
    if window.is_key_down(Key::Key1) {
        camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Key2) {
        camera.zoom(-zoom_speed);
    }

    // Activar vista de pájaro (tecla B)
    if window.is_key_down(Key::B) {
        camera.set_bird_eye_view();
    }
}

// Función para renderizar la órbita
fn render_orbit(framebuffer: &mut Framebuffer, radius: f32, segments: usize, color: u32) {
    let mut points = Vec::new();
    for i in 0..segments {
        let angle = 2.0 * PI * (i as f32 / segments as f32);
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        points.push(Vec3::new(x, 0.0, z));
    }

    for i in 0..points.len() {
        let next_index = (i + 1) % points.len();
        framebuffer.line(points[i], points[next_index]);
    }
}