use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use crate::PlanetType;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  // Transform position
  let position = Vec4::new(
    vertex.position.x,
    vertex.position.y,
    vertex.position.z,
    1.0
  );
  let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

  // Perform perspective division
  let w = transformed.w;
  let ndc_position = Vec4::new(
    transformed.x / w,
    transformed.y / w,
    transformed.z / w,
    1.0
  );

  // apply viewport matrix
  let screen_position = uniforms.viewport_matrix * ndc_position;

  // Transform normal
  let model_mat3 = mat4_to_mat3(&uniforms.model_matrix); 
  let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

  let transformed_normal = normal_matrix * vertex.normal;

  // Create a new Vertex with transformed attributes
  Vertex {
    position: vertex.position,
    normal: vertex.normal,
    tex_coords: vertex.tex_coords,
    color: vertex.color,
    transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
    transformed_normal,
  }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, planet_type: &PlanetType) -> Color {
    match planet_type {
        PlanetType::Sun => sun_shader(fragment, uniforms),
        PlanetType::RockyPlanet => rocky_planet_shader(fragment, uniforms),
        PlanetType::Earth => {
            let earth_color = earth_shader(fragment, uniforms);
            let cloud_color = cloud_shader(fragment, uniforms);
            blend_layers(earth_color, cloud_color)
        },
        PlanetType::CrystalPlanet => crystal_planet_shader(fragment, uniforms),
        PlanetType::FirePlanet => fire_planet_shader(fragment, uniforms),
        PlanetType::WaterPlanet => water_planet_shader(fragment, uniforms),
        PlanetType::CloudPlanet => cloud_planet_shader(fragment, uniforms),
        PlanetType::Moon => moon_shader(fragment, uniforms),
        PlanetType::Asteroid => asteroid_shader(fragment, uniforms),
        PlanetType::Trail => {
            let base_color = Color::new(100, 100, 255); // Color base para la estela (puedes personalizar)
            let trail_effect = calculate_trail_effect(fragment, uniforms); // Efecto dinámico
            blend_layers(base_color, trail_effect)
        },
        PlanetType::Spaceship => {
            // Color o shader específico para la nave
            Color::new(192, 192, 192) // Color gris para la nave
        }
        _ => Color::new(0, 0, 0),
    }
}

// Implementación de la función de cálculo para la estela
fn calculate_trail_effect(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Un ejemplo de cálculo para un efecto dinámico de estela
    let intensity = (fragment.position.y * (uniforms.time as f32).sin()).abs(); // Variación con el tiempo
    Color::new(
        (intensity * 255.0) as u8,
        (intensity * 100.0) as u8,
        200, // Azul fijo para contraste
    )
}


// Función para mezclar capas de color
fn blend_layers(base_color: Color, overlay_color: Color) -> Color {
    base_color.lerp(&overlay_color, 0.5) // Mezcla 50% de cada color
}

fn cloud_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 100.0;  // to move our values 
  let ox = 100.0; // offset x in the noise map
  let oy = 100.0;
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;
  let t = uniforms.time as f32 * 0.5;

  let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox + t, y * zoom + oy);

  // Define cloud threshold and colors
  let cloud_threshold = 0.5; // Adjust this value to change cloud density
  let cloud_color = Color::new(255, 255, 255); // White for clouds
  let sky_color = Color::new(30, 97, 145); // Sky blue

  // Determine if the pixel is part of a cloud or sky
  let noise_color = if noise_value > cloud_threshold {
    cloud_color
  } else {
    sky_color
  };

  noise_color * fragment.intensity
}

fn gaseous_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Generar ruido para simular nubes gaseosas
    let noise_value = uniforms.noise.get_noise_2d(x * zoom, y * zoom);
    
    // Colores base para las franjas de Júpiter (más oscuros)
    let stripe_color1 = Color::new(200, 153, 0); // Color amarillo oscuro
    let stripe_color2 = Color::new(200, 102, 0); // Color naranja oscuro
    let stripe_color3 = Color::new(153, 51, 0);  // Color marrón oscuro
    let stripe_color4 = Color::new(200, 200, 200); // Color blanco para las nubes (más transparente)
    let stripe_color5 = Color::new(150, 150, 150); // Color gris claro para variaciones

    // Crear un patrón de franjas utilizando ruido y la posición y
    let stripe_pattern = (y * 3.0).sin() * 0.5 + 0.5; // Aumentar la frecuencia para más franjas

    // Mezclar colores según el patrón de franjas
    let base_color = if stripe_pattern < 0.2 {
        stripe_color1
    } else if stripe_pattern < 0.4 {
        stripe_color2
    } else if stripe_pattern < 0.6 {
        stripe_color3
    } else if stripe_pattern < 0.8 {
        stripe_color4
    } else {
        stripe_color5 // Añadir un color gris claro para más variación
    };

    // Interpolación suave entre colores
    let smooth_color = if stripe_pattern < 0.2 {
        stripe_color1.lerp(&stripe_color2, stripe_pattern * 5.0)
    } else if stripe_pattern < 0.4 {
        stripe_color2.lerp(&stripe_color3, (stripe_pattern - 0.2) * 5.0)
    } else if stripe_pattern < 0.6 {
        stripe_color3.lerp(&stripe_color4, (stripe_pattern - 0.4) * 5.0)
    } else if stripe_pattern < 0.8 {
        stripe_color4.lerp(&stripe_color5, (stripe_pattern - 0.6) * 5.0)
    } else {
        stripe_color5.lerp(&stripe_color1, (stripe_pattern - 0.8) * 5.0)
    };

    // Ajustar el tamaño de la mancha blanca y hacerla más difusa
    let white_spot_size = 0.05; // Tamaño de la mancha blanca más pequeña
    let white_spot = if (y.abs() - 0.5).abs() < white_spot_size {
        stripe_color4 * 0.5 // Hacer la nube más transparente
    } else {
        smooth_color
    };

    // Aplicar un efecto de ruido adicional para variación
    let final_color = if noise_value > 0.5 { 
        stripe_color4.lerp(&smooth_color, 0.5) // Mezclar con el color suave
    } else { 
        white_spot 
    };

    // Ajustar la intensidad del color final
    final_color * fragment.intensity * 0.8 // Reducir la intensidad para un efecto más sutil
}

fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 30.0;
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;

  // Generar múltiples capas de ruido para textura rocosa detallada
  let noise_value = uniforms.noise.get_noise_2d(x * zoom, y * zoom);                    // Ruido grande para formaciones rocosas
  let small_noise_value = uniforms.noise.get_noise_2d(x * zoom * 2.0, y * zoom * 2.0);    // Ruido de alta frecuencia para detalles finos
  let medium_noise_value = uniforms.noise.get_noise_2d(x * zoom * 0.5, y * zoom * 0.5);    // Ruido de escala media para variabilidad
  let crater_noise = uniforms.noise.get_noise_2d(x * zoom * 3.0, y * zoom * 3.0);         // Ruido para simular los cráteres
  let very_small_noise_value = uniforms.noise.get_noise_2d(x * zoom * 4.0, y * zoom * 4.0); // Ruido extra fino para detalles muy pequeños

  // Colores base para las rocas (variaciones de grises, marrones, y toques de óxido)
  let base_rock_color = Color::new(156, 156, 156);    // Gris base
  let lighter_rock_color = Color::new(200, 200, 200); // Gris más claro
  let dark_rock_color = Color::new(100, 100, 100);    // Gris oscuro
  let brown_rock_color = Color::new(140, 120, 60);    // Marrón terracota
  let rust_rock_color = Color::new(120, 80, 40);      // Rojo oxidado
  let rocky_surface_color = Color::new(90, 70, 50);   // Superficie rocoso oscura
  let crater_color = Color::new(50, 50, 50);         // Color oscuro para los cráteres

  // Mezcla de colores según el ruido de las formaciones rocosas
  let base_color = if noise_value > 0.5 {
      lighter_rock_color
  } else {
      base_rock_color
  };

  // Crear una mezcla de colores más variados para las capas
  let detailed_color = if medium_noise_value > 0.5 {
      brown_rock_color
  } else {
      dark_rock_color
  };

  // Mezclar con el color de oxidación para crear texturas rocosas
  let oxide_layer_color = if very_small_noise_value > 0.5 {
      rust_rock_color
  } else {
      rocky_surface_color
  };

  // Agregar textura fina con detalles de pequeñas formaciones
  let texture_color = base_color.lerp(&detailed_color, small_noise_value.abs());

  // Añadir detalles de óxido o desgaste en la superficie
  let final_color = texture_color.lerp(&oxide_layer_color, small_noise_value.abs());

  // Crear un efecto de capas para simular formaciones rocosas más grandes
  let layer_effect = (noise_value * 0.5 + small_noise_value * 0.5).clamp(0.0, 1.0);
  let layered_color = detailed_color;

  // Crear un efecto de textura punteada o rugosa con ruido de alta frecuencia
  let dot_noise = uniforms.noise.get_noise_2d(x * zoom * 10.0, y * zoom * 10.0); // Ruido para puntos pequeños
  let dotted_effect = (dot_noise * 2.0).abs().clamp(0.0, 1.0);
  let dotted_color = layered_color.lerp(&Color::new(120, 120, 120), dotted_effect); // Mezcla con gris claro para los puntos

  // Crear un efecto de superficie rugosa
  let roughness_effect = (noise_value * 0.3 + small_noise_value * 0.7).clamp(0.2, 1.0); // Rugosidad
  let rough_color = dotted_color * roughness_effect;

  // Ajustar la intensidad de la luz para suavizar las sombras
  // Usar una luz ambiental o suavizar la iluminación basada en la posición
  let ambient_light = 0.7; // Luz ambiental suave
  let light_intensity = 1.0; // Suavizar la luz según la posición 'y' sin hacerla demasiado oscura

  // Combinar la luz ambiental con la intensidad de la luz calculada
  let illuminated_color = rough_color * (ambient_light + light_intensity * 0.8);


  // Ajustar la intensidad final de la textura
  illuminated_color * fragment.intensity * 1.95 // Reducir un poco la intensidad general para un acabado más equilibrado
}

fn sun_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Base colors for the lava effect
  let bright_color = Color::new(255, 240, 0); // yellow
  let dark_color = Color::new(211, 84, 0);   //Burnt orange

  // Get fragment position
  let position = Vec3::new(
    fragment.vertex_position.x,
    fragment.vertex_position.y,
    fragment.depth
  );

  // Base frequency and amplitude for the pulsating effect
  let base_frequency = 0.2;
  let pulsate_amplitude = 0.5;
  let t = uniforms.time as f32 * 0.01;

  // Pulsate on the z-axis to change spot size
  let pulsate = (t * base_frequency).sin() * pulsate_amplitude;

  // Apply noise to coordinates with subtle pulsating on z-axis
  let zoom = 1000.0; // Constant zoom factor
  let noise_value1 = uniforms.noise.get_noise_3d(
    position.x * zoom,
    position.y * zoom,
    (position.z + pulsate) * zoom
  );
  let noise_value2 = uniforms.noise.get_noise_3d(
    (position.x + 1000.0) * zoom,
    (position.y + 1000.0) * zoom,
    (position.z + 1000.0 + pulsate) * zoom
  );
  let noise_value = (noise_value1 + noise_value2) * 0.5;  // Averaging noise for smoother transitions

  // Use lerp for color blending based on noise value
  let color = dark_color.lerp(&bright_color, noise_value);

  color * fragment.intensity
}

fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 100.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Generar ruido para simular la superficie lunar
    let noise_value = uniforms.noise.get_noise_2d(x * zoom, y * zoom);
    let moon_color = Color::new(200, 200, 200); // Color gris
    let crater_color = Color::new(150, 150, 150); // Color más oscuro para los cráteres

    // Mezclar colores según el ruido
    let final_color = if noise_value > 0.5 {
        crater_color
    } else {
        moon_color
    };

    // Simular rotación de la luna
    let rotation_effect = (uniforms.time as f32 * 0.1).sin() * 0.1;
    let rotated_color = final_color.lerp(&Color::new(255, 255, 255), rotation_effect);

    rotated_color * fragment.intensity
}

fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 30.0; // Zoom para la textura de la Tierra
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Generar ruido para simular la textura de la Tierra
    let noise_value = uniforms.noise.get_noise_2d(x * zoom, y * zoom);
    let land_noise = uniforms.noise.get_noise_2d(x * zoom * 3.0, y * zoom * 3.0); // Ruido para el continente

    // Colores base para la tierra y el agua
    let land_color = Color::new(34, 139, 34); // Color verde para la tierra
    let water_color = Color::new(0, 0, 255); // Color azul para el agua
    let island_color = Color::new(0, 255, 0); // Color verde brillante para la isla

    // Mezclar colores según el ruido para simular tierra y agua
    let base_color = if noise_value > 0.5 {
        land_color
    } else {
        water_color
    };

    // Determinar si hay una isla o continente adicional
    let island_effect = if land_noise > 0.5 {
        island_color // Si el ruido es alto, usar el color de la isla
    } else {
        Color::new(0, 0, 0) // Sin isla
    };

    // Aplicar el shader de nubes
    let cloud_color = cloud_shader(fragment, uniforms);

    // Mezclar el color base con el color de las nubes y la isla
    let final_color = base_color.lerp(&cloud_color, 0.5).lerp(&island_effect, 0.5); // Mezcla 50% de nubes y 50% de isla

    final_color * fragment.intensity
}


fn cloud_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0; // Controla la escala del ruido
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.5; // Tiempo para animar las nubes

    // Generar múltiples capas de ruido para simular nubes
    let noise_value1 = uniforms.noise.get_noise_2d(x * zoom + t, y * zoom + t);
    let noise_value2 = uniforms.noise.get_noise_2d(x * zoom * 0.5 + t, y * zoom * 0.5);
    let noise_value3 = uniforms.noise.get_noise_2d(x * zoom * 2.0 + t, y * zoom * 2.0);

    // Colores base para las nubes y el cielo
    let cloud_color = Color::new(255, 255, 255); // Blanco para las nubes
    let sky_color = Color::new(135, 206, 235); // Azul cielo
    let cloud_shadow_color = Color::new(200, 200, 200); // Sombra de nubes

    // Definir umbrales para determinar la densidad de las nubes
    let cloud_threshold1 = 0.4; // Umbral para la primera capa de nubes
    let cloud_threshold2 = 0.6; // Umbral para la segunda capa de nubes
    let cloud_threshold3 = 0.8; // Umbral para la tercera capa de nubes

    // Determinar el color de las nubes basado en el ruido
    let mut noise_color = sky_color; // Comenzar con el color del cielo

    if noise_value1 > cloud_threshold1 {
        noise_color = noise_color.lerp(&cloud_color, (noise_value1 - cloud_threshold1) / (1.0 - cloud_threshold1));
    }
    if noise_value2 > cloud_threshold2 {
        noise_color = noise_color.lerp(&cloud_shadow_color, (noise_value2 - cloud_threshold2) / (1.0 - cloud_threshold2));
    }
    if noise_value3 > cloud_threshold3 {
        noise_color = noise_color.lerp(&cloud_color, (noise_value3 - cloud_threshold3) / (1.0 - cloud_threshold3));
    }

    // Ajustar la intensidad del color final
    noise_color * fragment.intensity
}

fn crystal_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 30.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Generar ruido para simular cristales brillantes
    let noise_value = uniforms.noise.get_noise_2d(x * zoom + uniforms.time as f32 * 0.2, y * zoom);
    
    // Colores base para los cristales
    let crystal_color1 = Color::new(0, 255, 255); // Cian
    let crystal_color2 = Color::new(255, 0, 255); // Magenta

    // Interpolación suave para simular el brillo de los cristales
    let color = crystal_color1.lerp(&crystal_color2, noise_value);

    // Aumentar el brillo
    let bright_color = color * 1.5; // Aumentar el brillo

    // Ajustar la intensidad del color final
    bright_color * fragment.intensity * 2.8 // Reducir la intensidad para un efecto más sutil
}

fn fire_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 80.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Generar ruido para simular fuego con movimiento
    let noise_value = uniforms.noise.get_noise_2d(x * zoom + uniforms.time as f32 * 0.5, y * zoom);
    
    // Colores base para el fuego
    let fire_color1 = Color::new(255, 140, 0); // Naranja
    let fire_color2 = Color::new(255, 0, 0);   // Rojo

    // Interpolación suave para simular el movimiento del fuego
    let color = fire_color1.lerp(&fire_color2, noise_value);

    // Aplicar el shader de franjas como capa extra
    let stripe_color = striped_planet_shader(fragment, uniforms);
    
    // Ajustar la opacidad del shader de franjas
    let opacity = 0.5; // Ajustar la opacidad según sea necesario
    let final_color = color.lerp(&stripe_color, opacity);

    // Ajustar la intensidad del color final
    final_color * fragment.intensity
}

fn water_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 40.0;
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;

  // Crear un seed basado en la posición
  let seed = (x * 1000.0 + y * 1000.0) as u64; 
  let mut rng = StdRng::seed_from_u64(seed);
  let random_offset = rng.gen_range(0.0..=1.0); // Generar un desplazamiento aleatorio

  // Generar ruido para simular agua con movimiento
  let noise_value = uniforms.noise.get_noise_2d(x * zoom + random_offset, y * zoom + (uniforms.time as f32 * 0.1).sin());

  // Generar ondas con mayor amplitud utilizando una función seno controlada
  let wave_effect = ((x + uniforms.time as f32 * 0.1).sin() * 0.5 + (y + uniforms.time as f32 * 0.1).cos() * 0.5).sin() * 1.0; // Aumentar la amplitud de la ola

  // Colores base para el agua, con un celeste más saturado y profundo
  let water_color1 = Color::new(0, 0, 255);     // Azul profundo
  let water_color2 = Color::new(0, 150, 255);   // Celeste intenso

  // Interpolación suave para simular el movimiento del agua
  let wave_intensity = wave_effect * noise_value.abs() * 1.5; // Aumentar la intensidad de la ola

  // Lerp entre los dos colores usando la intensidad de la ola
  let color = water_color1.lerp(&water_color2, wave_intensity);

  // Ajustar la intensidad del color final
  color * fragment.intensity * 0.9 // Aumentar ligeramente la intensidad para resaltar más el celeste
}


fn striped_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 10.0; // Controla la frecuencia de las franjas
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Colores base para las franjas
    let stripe_color1 = Color::new(255, 100, 0); // Naranja intenso
    let stripe_color2 = Color::new(255, 200, 0); // Amarillo brillante
    let stripe_color3 = Color::new(255, 50, 0);  // Naranja rojizo

    // Crear un patrón de franjas utilizando una función seno
    let stripe_pattern = (y * zoom + uniforms.time as f32 * 0.1).sin() + (x * zoom * 0.5).sin(); // Movimiento más lento

    // Determinar el color basado en el patrón de franjas
    let color = if stripe_pattern > 0.0 {
        stripe_color1
    } else {
        stripe_color3 // Cambiar a naranja rojizo
    };

    // Ajustar la intensidad del color final y aplicar la opacidad
    let opacity = (stripe_pattern.abs() * 0.5).clamp(0.0, 1.0); // Controlar la opacidad
    color * opacity * fragment.intensity
}

pub fn asteroid_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 20.0; // Controla la escala del ruido
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    // Generar múltiples capas de ruido para textura detallada
    let base_noise = uniforms.noise.get_noise_2d(x * zoom, y * zoom); // Ruido base
    let small_noise = uniforms.noise.get_noise_2d(x * zoom * 2.0, y * zoom * 2.0); // Ruido más pequeño
    let medium_noise = uniforms.noise.get_noise_2d(x * zoom * 0.5, y * zoom * 0.5); // Ruido medio
    let lava_noise = uniforms.noise.get_noise_2d(x * zoom * 4.0, y * zoom * 4.0); // Ruido para las piscinas de lava

    // Colores base para el asteroide
    let base_color = Color::new(150, 150, 150); // Gris base
    let lighter_color = Color::new(200, 200, 200); // Gris más claro
    let dark_color = Color::new(100, 100, 100); // Gris oscuro
    let rust_color = Color::new(120, 80, 40); // Color oxidado
    let lava_color1 = Color::new(255, 100, 0); // Color de lava (brillante)
    let lava_color2 = Color::new(255, 50, 0);  // Color de lava (más oscuro)

    // Mezcla de colores según el ruido
    let color_variation = if base_noise > 0.5 {
        lighter_color.lerp(&base_color, small_noise.abs())
    } else {
        dark_color.lerp(&rust_color, medium_noise.abs())
    };

    // Efecto de lava dinámico
    let time = uniforms.time as f32 * 0.5; // Controlar la velocidad de pulsación
    let blend_factor = time.sin() * 0.5 + 0.5; // Oscilar entre 0 y 1

    // Determinar si hay lava en la superficie
    let lava_effect = if lava_noise > 0.5 {
        lava_color1.lerp(&lava_color2, blend_factor) // Mezclar colores de lava
    } else {
        Color::new(0, 0, 0) // Sin lava
    };

    // Combinar el color base con el efecto de lava
    let final_color = color_variation.lerp(&lava_effect, lava_noise.abs());

    // Ajustar la intensidad del color final
    final_color * fragment.intensity
}
