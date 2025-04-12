use crate::config::{MapConfig, RefinerConfig};

pub fn refine_heightmap(
    heightmap: &Vec<f32>,
    config: &RefinerConfig,
    map_config: &MapConfig,
) -> Vec<f32> {
    let width = map_config.width as usize;
    let height = map_config.height as usize;
    let size = width * height;

    // clone the heightmap to avoid modifying the original
    let mut heightmap = heightmap.clone();

    // Apply height offset, coefficient, and exponent
    for i in 0..size {
        heightmap[i] = (heightmap[i] as f32 + config.height_offset) * config.height_coeff;
        heightmap[i] = heightmap[i].powf(config.height_exponent);
    }

    // Smooth the heightmap
    // TODO: Implement smoothing logic
    // This could involve averaging neighboring pixels or applying a Gaussian blur
    // probably something more advanced taking into account cliffs and other features
    // would be nice to have a preview of the smoothed heightmap

    // Apply curve points if provided
    // TODO: Implement curve points logic

    // Apply paint map overlay if provided
    // TODO: Implement paint map overlay logic

    // Normalize the heightmap to the range [0.0, 1.0]
    let min_height = *heightmap.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max_height = *heightmap.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let range = max_height - min_height;
    if range > 0.0 {
        for i in 0..size {
            heightmap[i] = (heightmap[i] - min_height) / range;
        }
    }

    // Return the refined heightmap
    heightmap
}


