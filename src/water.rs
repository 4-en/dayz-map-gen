use crate::config::{MapConfig, WaterConfig};
use eframe::egui;
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use rand::{
    Rng,
    SeedableRng,
    rngs::StdRng,
};


pub fn generate_water_map(
    map_config: &MapConfig,
    water_config: &WaterConfig,
    heightmap: &[f32],
    biome_map: &[u8],
    seed: u32,
) -> (egui::ColorImage, ImageBuffer<Rgba<u8>, Vec<u8>>, Vec<f32>, Vec<f32>) {
    let width = map_config.width;
    let height = map_config.height;
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let mut lake_map = vec![0.0f32; (width * height) as usize];
    let mut river_map = vec![0.0f32; (width * height) as usize];
    let mut adjusted_height_map = heightmap.iter().map(|&h| h).collect::<Vec<f32>>();

    let water_map_buf = std::sync::Mutex::new(&mut lake_map);
    let river_map_buf = std::sync::Mutex::new(&mut river_map);
    let adjusted_height_map_buf = std::sync::Mutex::new(&mut adjusted_height_map);

    // TODO: generate lakes

    // TODO: generate rivers


    // placeholders
    let preview = ImageBuffer::new(width, height);
    let preview_image = egui::ColorImage::new(
        [width as usize, height as usize],
        egui::Color32::from_black_alpha(0),
    );

    

    (
        preview_image,
        preview,
        lake_map,
        river_map,
    )
}