use crate::config::{MapConfig, WaterConfig};
use eframe::egui;
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use rand::{
    Rng,
    SeedableRng,
    rngs::StdRng,
};

pub fn get_color_for_water(value: f32) -> (u8, u8, u8) {
    let blue = (value * 255.0).clamp(0.0, 255.0) as u8;
    let green = ((1.0 - value) * 255.0).clamp(0.0, 255.0) as u8;
    (blue, green, 255)
}

pub fn generate_lake_at(
    map_config: &MapConfig,
    water_config: &WaterConfig,
    heightmap: &[f32],
    lake_map: &mut [f32],
    x: f32,
    y: f32
) -> bool {
    let width = map_config.width as f32;
    let height = map_config.height as f32;
    
    // try to generate a lake by searching for bowl at given coordinates
    // basically a flood fill algorithm

    // 1. find a local minimum in the heightmap
    let mut center_x = x as i32;
    let mut center_y = y as i32;
    // TODO: set center to nearest local minimum in heightmap

    // abort if one of the following conditions is met:
    // - center is out of bounds
    // - center is too low or too high (check config)
    // - center is already part of a lake

    // 2. flood fill the area around the center
    // - find next lowest neighbor (4 directions)
    // - set new lake height to the new neighbor height if heigher than last lake height
    // - add all neighbors lower than the lake height
    // - repeat until stop condition is met

    // stop on one of the following conditions:
    // - the area is too big (radius)
    // - the area is too deep
    // when stopped, go back to last valid lake

    // 3. lower lake height by a random amount (config)

    // 4. check all points of the lake

    // 5. apply heightmap modification (slightly lower the heightmap in the lake area)

    // 6. expand lake area by 1 pixel in all directions (since height map precision is 1m/pixel, the actual intersection is somewhere between the pixel below and above the lake)

    // 7. add lake to the lake map (using lake height for aal pixels in the lake area)
    
    // 8. return true if lake was generated, false if not

    
    false
}



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