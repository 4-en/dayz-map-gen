use crate::config::{BiomeConfig, MapConfig};
use eframe::egui;
use image::{ImageBuffer, Rgba};
use noise::{NoiseFn, Perlin, Seedable};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Ocean,
    Beach,
    Plains,
    Forest,
    Mountain,
    Snow,
    Desert,
    Swamp,
    Tundra,
    Jungle,
}

pub fn get_biome_color(biome: Biome) -> (u8, u8, u8) {
    match biome {
        Biome::Ocean => (0, 0, 100),
        Biome::Beach => (238, 214, 175),
        Biome::Plains => (50, 205, 50),
        Biome::Forest => (34, 139, 34),
        Biome::Mountain => (139, 137, 137),
        Biome::Snow => (255, 250, 250),
        Biome::Desert => (255, 228, 181),
        Biome::Swamp => (0, 100, 0),
        Biome::Tundra => (255, 228, 196),
        Biome::Jungle => (0, 128, 0),
    }
}

pub fn choose_biome(temp: f64, humidity: f64, elev: f32, sea_level: f32, slope: f32) -> Biome {
    // TODO: this is so messy, please fix ^^
    if elev < sea_level * 0.8 {
        Biome::Ocean
    } else if elev < sea_level {
        Biome::Beach
    } else if humidity > 0.7 && temp > 0.7 {
        if elev > 0.8 {
            Biome::Mountain
        } else {
            Biome::Jungle
        }
    } else if temp < 0.2 {
        Biome::Snow
    } else if slope > 0.5 {
        Biome::Mountain
    } else if elev < sea_level * 1.2 {
        if humidity > 0.7 {
            if temp > 0.5 {
                Biome::Jungle
            } else {
                Biome::Swamp
            }
        } else if humidity > 0.4 {
            if temp > 0.5 {
                Biome::Forest
            } else {
                Biome::Plains
            }
        } else {
            if temp > 0.7 {
                Biome::Desert
            } else {
                Biome::Plains
            }
        }
    } else if elev < sea_level * 1.5 {
        if humidity > 0.5 {
            if temp > 0.5 {
                Biome::Mountain
            } else {
                Biome::Tundra
            }
        } else {
            if temp > 0.7 {
                Biome::Desert
            } else {
                Biome::Forest
            }
        }
    } else {
        if temp < 0.3 {
            Biome::Snow
        } else if temp < 0.5 {
            Biome::Mountain
        } else if temp < 0.7 {
            Biome::Forest
        } else {
            Biome::Desert
        }
    }
}

pub fn generate_biome_map(
    map_config: &MapConfig,
    biome_config: &BiomeConfig,
    heightmap: &[f32],
    seed: u32,
) -> (egui::ColorImage, ImageBuffer<Rgba<u8>, Vec<u8>>, Vec<u8>) {
    let width = map_config.width;
    let height = map_config.height;
    let size = (width * height) as usize;

    let sea_level = map_config.sea_level.clamp(0.0, 1.0) as f32;

    let perlin_temp: Perlin = Perlin::new().set_seed(seed);
    let perlin_hum: Perlin = Perlin::new().set_seed(seed + 2000);

    let avg_temp = ((biome_config.base_temperature + 10.0) / 50.0).clamp(0.0, 1.0);
    let avg_hum = (biome_config.base_humidity / 100.0).clamp(0.0, 1.0);
    let temp_variation = (biome_config.temperature_variation / 100.0).clamp(0.0, 1.0);
    let hum_variation = (biome_config.humidity_variation / 100.0).clamp(0.0, 1.0);

    let min_temp = (avg_temp - temp_variation) as f64;
    let max_temp = (avg_temp + temp_variation) as f64;
    let min_hum = (avg_hum - hum_variation) as f64;
    let max_hum = (avg_hum + hum_variation) as f64;

    // Move ownership of the preview image and biome IDs into the mutex.
    let preview_buf = std::sync::Mutex::new(ImageBuffer::new(width, height));
    let biome_ids_buf = std::sync::Mutex::new(vec![0u8; size]);

    (0..height).into_par_iter().for_each(|y| {
        let mut row_biomes = Vec::with_capacity(width as usize);
        let mut row_colors = Vec::with_capacity(width as usize);
        let ny = y as f64;

        for x in 0..width {
            let idx = (y * width + x) as usize;
            let h = heightmap[idx];
            let nx = x as f64;

            // Calculate slope with neighboring pixels.
            let mut slope = 0.0;
            if false && x > 0 && y > 0 && x < width - 1 && y < height - 1 {
                let left = heightmap[idx - 1];
                let right = heightmap[idx + 1];
                let up = heightmap[idx - width as usize];
                let down = heightmap[idx + width as usize];

                slope = ((left - h).abs() + (right - h).abs() + (up - h).abs() + (down - h).abs())
                    / 4.0;
                slope *= 1000.0;

                let angle_rad = slope.atan2(1.0);
                slope = (angle_rad / std::f32::consts::PI * 2.0).clamp(0.0, 1.0);
            }

            // Generate temperature and humidity based on perlin noise.
            let mut temp =
                (perlin_temp.get([nx / biome_config.scale, ny / biome_config.scale]) + 1.0) / 2.0;
            let mut humidity =
                (perlin_hum.get([nx / biome_config.scale, ny / biome_config.scale]) + 1.0) / 2.0;

            temp = temp * (max_temp - min_temp) + min_temp;
            humidity = humidity * (max_hum - min_hum) + min_hum;

            let biome = choose_biome(temp, humidity, h, sea_level, slope);
            let color = get_biome_color(biome); // Returns (u8, u8, u8)

            row_biomes.push(biome);
            row_colors.push(color);
        }

        // Lock and update the preview image and biome IDs.
        let mut preview_lock = preview_buf.lock().unwrap();
        let mut biome_ids_lock = biome_ids_buf.lock().unwrap();
        for x in 0..width {
            let i = (y * width + x) as usize;
            biome_ids_lock[i] = row_biomes[x as usize] as u8;
            preview_lock.put_pixel(
                x,
                y,
                Rgba([
                    row_colors[x as usize].0,
                    row_colors[x as usize].1,
                    row_colors[x as usize].2,
                    255,
                ]),
            );
        }
    });

    // Extract the values from the mutexes.
    let preview = preview_buf.into_inner().unwrap();
    let biome_ids = biome_ids_buf.into_inner().unwrap();

    let pixels = preview
        .pixels()
        .map(|p| egui::Color32::from_rgb(p[0], p[1], p[2]))
        .collect();

    let size_arr = [width as usize, height as usize];
    (
        egui::ColorImage {
            size: size_arr,
            pixels,
        },
        preview,
        biome_ids,
    )
}
