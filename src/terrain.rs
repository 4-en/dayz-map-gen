use image::{ImageBuffer, Rgba};
use noise::{NoiseFn, Perlin, Seedable};
use rayon::prelude::*;
use eframe::egui;
use crate::config::MapConfig;
use crate::preview::get_color_for_height;

pub fn generate_map(config: &MapConfig, seed: u32, previous_map: &Option<Vec<f32>>) -> (egui::ColorImage, ImageBuffer<Rgba<u8>, Vec<u8>>, Vec<f32>) {
    let perlin = Perlin::new().set_seed(seed);
    let width = config.width;
    let height = config.height;
    let mut preview = ImageBuffer::new(width, height);
    let mut heightmap = vec![0.0f32; (width * height) as usize];

    let overlay_strength = (config.overlay / 100.0).clamp(0.0, 1.0);
    let overlay_old = 1.0 - overlay_strength;
    // check if strength is below 1 and if old map is provided and same size
    let overlay: bool = overlay_strength < 0.999 && previous_map.is_some() && previous_map.as_ref().unwrap().len() == (width * height) as usize;
    let previous_ref = previous_map.as_ref();


    let max_mountainous = 1.5_f64.powf(config.mountainous) - 0.5;
    let max_amp = max_mountainous * config.amp_base + config.amp_mid + config.amp_detail;

    let preview_buf = std::sync::Mutex::new(&mut preview);
    let heightmap_buf = std::sync::Mutex::new(&mut heightmap);

    (0..height).into_par_iter().for_each(|y| {
        let mut row_data = Vec::with_capacity(width as usize);
        for x in 0..width {
            let nx = x as f64;
            let ny = y as f64;

            let base = (perlin.get([nx / config.scale_base, ny / config.scale_base]) + 1.0) / 2.0;
            let mid = (perlin.get([nx / config.scale_mid, ny / config.scale_mid]) + 1.0) / 2.0;
            let detail = (perlin.get([nx / config.scale_detail, ny / config.scale_detail]) + 1.0) / 2.0;

            let mut h = base;
            h = (h + 0.5).powf(config.mountainous);
            h = (h - 0.5) * config.amp_base;
            h += config.amp_mid * mid;
            h += config.amp_detail * detail;
            h = (h / max_amp).clamp(0.0, 1.0);

            if config.island_mode {
                let border = config.island_border.clamp(0.01, 0.5);
                let curve = config.island_curve.clamp(1.0, 10.0);
                let xf = x as f64 / width as f64;
                let yf = y as f64 / height as f64;

                let mut edge_strength_x = 0.0;
                let mut edge_strength_y = 0.0;
                if xf < border {
                    edge_strength_x = 1.0 - (xf / border);
                } else if xf > 1.0 - border {
                    edge_strength_x = (xf - (1.0 - border)) / border;
                }
                if yf < border {
                    edge_strength_y = 1.0 - (yf / border);
                } else if yf > 1.0 - border {
                    edge_strength_y = (yf - (1.0 - border)) / border;
                }

                let edge_strength = edge_strength_x + edge_strength_y;
                let falloff = 1.0 - edge_strength.powf(curve);
                h *= falloff;
            }

            if overlay {
                let old_height = previous_ref.unwrap()[(y * width + x) as usize] as f64;
                h = h * overlay_strength + old_height * overlay_old;
            }

            row_data.push((h as f32, get_color_for_height(h as f64, config.sea_level)));
        }

        let mut preview_lock = preview_buf.lock().unwrap();
        let mut heightmap_lock = heightmap_buf.lock().unwrap();
        for x in 0..width {
            let i = (y * width + x) as usize;
            heightmap_lock[i] = row_data[x as usize].0;
            preview_lock.put_pixel(
                x, y,
                Rgba([
                    row_data[x as usize].1.0,
                    row_data[x as usize].1.1,
                    row_data[x as usize].1.2,
                    255,
                ])
            );
        }
    });

    let pixels = preview
        .pixels()
        .map(|p| egui::Color32::from_rgb(p[0], p[1], p[2]))
        .collect();

    let size = [width as usize, height as usize];
    (egui::ColorImage { size, pixels }, preview, heightmap)
}
