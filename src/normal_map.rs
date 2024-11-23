use std::sync::Arc;
use once_cell::sync::OnceCell;
use crate::color::Color;
use nalgebra_glm::Vec3;

static NORMAL_MAP: OnceCell<Arc<NormalMap>> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct NormalMap {
    width: u32,
    height: u32,
    data: Vec<Vec3>,
}

impl NormalMap {
    pub fn new(path: &str) -> Result<Self, image::ImageError> {
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();
        
        // Convert RGB colors to normal vectors (assuming tangent-space normal map)
        let data = img.pixels()
            .map(|p| {
                // Convert from [0,255] to [-1,1] range
                let x = (p[0] as f32 / 255.0) * 2.0 - 1.0;
                let y = (p[1] as f32 / 255.0) * 2.0 - 1.0;
                let z = (p[2] as f32 / 255.0) * 2.0 - 1.0;
                Vec3::new(x, y, z).normalize()
            })
            .collect();

        Ok(NormalMap { width, height, data })
    }

    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let u = u.fract().abs();
        let v = v.fract().abs();
        
        let x = (u * (self.width as f32)) as u32;
        let y = (v * (self.height as f32)) as u32;
        
        let index = (y * self.width + x) as usize;
        self.data[index]
    }
}

pub fn init_normal_map(path: &str) -> Result<(), image::ImageError> {
    let normal_map = NormalMap::new(path)?;
    NORMAL_MAP.set(Arc::new(normal_map))
        .expect("Normal map already initialized");
    Ok(())
}

pub fn with_normal_map(f: impl FnOnce(&NormalMap) -> Vec3) -> Vec3 {
    let normal_map = NORMAL_MAP.get()
        .expect("Normal map not initialized");
    f(normal_map)
}

pub fn render_earth(u: f32, v: f32) -> Vec3 {
    // Obtener el color base de la Tierra
    let base_color = Vec3::new(0.0, 0.5, 1.0); // Color azul para la Tierra

    // Obtener el vector normal del mapa normal
    let normal = with_normal_map(|normal_map| normal_map.sample(u, v));

    // Combinar el color base con el normal (esto es solo un ejemplo)
    let final_color = base_color + normal * 0.1; // Ajusta la intensidad según sea necesario

    final_color
}

