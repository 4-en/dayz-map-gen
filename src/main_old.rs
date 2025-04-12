use eframe::egui;
use image::{ImageBuffer, Rgba};
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;
use rayon::prelude::*;
use rfd::FileDialog;

/// Configuration for the map generator.
#[derive(Debug)]
struct MapConfig {
    width: u32, // in pixels (1 pixel = 1 meter)
    height: u32,
    scale_base: f64,
    amp_base: f64,
    scale_mid: f64,
    amp_mid: f64,
    scale_detail: f64,
    amp_detail: f64,
    seed: u32,
    use_random_seed: bool,
    island_mode: bool,
    island_border: f64, // e.g. 0.1 = outer 10%
    island_curve: f64,  // e.g. 2.0 = quadratic
    sea_level: f64,     // normalized threshold [0,1] for water
    mountainous: f64,   // heigher means more extreme mountains
}

enum GenerationStep {
    Terrain,
    Water,
    Biomes,
    Objects,
    Export,
}

impl Clone for GenerationStep {
    fn clone(&self) -> Self {
        match self {
            GenerationStep::Terrain => GenerationStep::Terrain,
            GenerationStep::Water => GenerationStep::Water,
            GenerationStep::Biomes => GenerationStep::Biomes,
            GenerationStep::Objects => GenerationStep::Objects,
            GenerationStep::Export => GenerationStep::Export,
        }
    }
}

impl Copy for GenerationStep {}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            seed: 12345,
            use_random_seed: true,
            island_mode: true,
            island_border: 0.1,
            island_curve: 2.0,
            sea_level: 0.4,
            scale_base: 400.0,
            amp_base: 1.0,
            scale_mid: 100.0,
            amp_mid: 0.5,
            scale_detail: 25.0,
            amp_detail: 0.15,
            mountainous: 1.5,
        }
    }
}

/// Returns a color based on the normalized height value `h` and sea level.
fn get_color_for_height(h: f64, sea_level: f64) -> (u8, u8, u8) {
    if h < sea_level * 0.6 {
        (0, 0, 100) // Deep water
    } else if h < sea_level {
        (64, 164, 223) // Shallow water
    } else if h < 0.5 {
        (34, 139, 34) // Grassland
    } else if h < 0.65 {
        (160, 82, 45) // Hills
    } else if h < 0.85 {
        (139, 137, 137) // Mountains
    } else {
        (255, 250, 250) // Snowy peaks
    }
}

/// Generates a colored map as an egui::ColorImage using Perlin noise.
/// The process uses the provided configuration, optionally applying an island mask.
fn generate_map(
    config: &MapConfig,
    seed: u32,
) -> (egui::ColorImage, ImageBuffer<Rgba<u8>, Vec<u8>>, Vec<f32>) {
    let perlin = Perlin::new().set_seed(seed);
    let width = config.width;
    let height = config.height;

    // Allocate data structures
    let mut preview = ImageBuffer::new(width, height);
    let mut heightmap = vec![0.0f32; (width * height) as usize];

    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let max_mountainous = 1.5_f64.powf(config.mountainous) - 0.5;
    let max_amp = max_mountainous * config.amp_base + config.amp_mid + config.amp_detail;

    // Wrap shared resources
    let preview_buf = std::sync::Mutex::new(&mut preview);
    let heightmap_buf = std::sync::Mutex::new(&mut heightmap);

    // Process rows in parallel
    (0..height).into_par_iter().for_each(|y| {
        let mut row_data = Vec::with_capacity(width as usize);

        for x in 0..width {
            let nx = x as f64;
            let ny = y as f64;

            let base = (perlin.get([nx / config.scale_base, ny / config.scale_base]) + 1.0) / 2.0;
            let mid = (perlin.get([nx / config.scale_mid, ny / config.scale_mid]) + 1.0) / 2.0;
            let detail =
                (perlin.get([nx / config.scale_detail, ny / config.scale_detail]) + 1.0) / 2.0;

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

            row_data.push((h as f32, get_color_for_height(h as f64, config.sea_level)));
        }

        // Write data back with locking
        let mut preview_lock = preview_buf.lock().unwrap();
        let mut heightmap_lock = heightmap_buf.lock().unwrap();
        for x in 0..width {
            let i = (y * width + x) as usize;
            heightmap_lock[i] = row_data[x as usize].0;
            preview_lock.put_pixel(
                x,
                y,
                Rgba([
                    row_data[x as usize].1.0,
                    row_data[x as usize].1.1,
                    row_data[x as usize].1.2,
                    255,
                ]),
            );
        }
    });

    // Convert to egui::ColorImage
    let pixels = preview
        .pixels()
        .map(|p| egui::Color32::from_rgb(p[0], p[1], p[2]))
        .collect();

    let size = [width as usize, height as usize];
    (egui::ColorImage { size, pixels }, preview, heightmap)
}

/// The main application structure holding the configuration and preview texture.
struct DayZMapApp {
    current_step: GenerationStep,
    config: MapConfig,
    preview_texture: Option<egui::TextureHandle>,
    preview_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    heightmap_data: Option<Vec<f32>>,
}

impl Default for DayZMapApp {
    fn default() -> Self {
        Self {
            current_step: GenerationStep::Terrain,
            config: MapConfig::default(),
            preview_texture: None,
            preview_image: None,
            heightmap_data: None,
        }
    }
}

impl DayZMapApp {
    fn render_terrain_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Map Settings");
        ui.separator();

        // Width / Height as text fields
        ui.horizontal(|ui| {
            ui.label("Width (px):");
            let mut width_str = self.config.width.to_string();
            if ui.text_edit_singleline(&mut width_str).changed() {
                if let Ok(w) = width_str.parse() {
                    self.config.width = w;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Height (px):");
            let mut height_str = self.config.height.to_string();
            if ui.text_edit_singleline(&mut height_str).changed() {
                if let Ok(h) = height_str.parse() {
                    self.config.height = h;
                }
            }
        });

        ui.checkbox(&mut self.config.use_random_seed, "Use Random Seed");

        if !self.config.use_random_seed {
            ui.label("Seed:");
            ui.add(egui::DragValue::new(&mut self.config.seed).speed(1));
        } else {
            ui.label(format!("Random Seed: {}", self.config.seed));
        }

        ui.label("Sea Level:");
        ui.add(egui::Slider::new(&mut self.config.sea_level, 0.0..=1.0));

        ui.separator();
        ui.heading("Island Shaping");

        ui.checkbox(&mut self.config.island_mode, "Enable Island Mode");
        ui.add(
            egui::Slider::new(&mut self.config.island_border, 0.01..=0.5).text("Island Border %"),
        );
        ui.add(egui::Slider::new(&mut self.config.island_curve, 1.0..=10.0).text("Falloff Curve"));

        ui.separator();
        ui.label("Terrain Contrast (Mountains)");
        ui.add(egui::Slider::new(&mut self.config.mountainous, 0.3..=3.0).text("Mountainous"));

        ui.separator();
        ui.heading("Noise Layers");

        ui.label("Base Noise");
        ui.add(egui::Slider::new(&mut self.config.scale_base, 10.0..=10000.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.config.amp_base, 0.0..=2.0).text("Amp"));

        ui.label("Mid Noise");
        ui.add(egui::Slider::new(&mut self.config.scale_mid, 10.0..=1000.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.config.amp_mid, 0.0..=2.0).text("Amp"));

        ui.label("Detail Noise");
        ui.add(egui::Slider::new(&mut self.config.scale_detail, 5.0..=100.0).text("Scale"));
        ui.add(egui::Slider::new(&mut self.config.amp_detail, 0.0..=2.0).text("Amp"));

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Generate Map").clicked() {
                let seed = if self.config.use_random_seed {
                    let new_seed = rand::random::<u32>();
                    self.config.seed = new_seed;
                    new_seed
                } else {
                    self.config.seed
                };

                let (color_image, preview_img, heightmap_data) = generate_map(&self.config, seed);
                self.preview_texture =
                    Some(ctx.load_texture("preview", color_image, egui::TextureOptions::default()));
                self.preview_image = Some(preview_img);
                self.heightmap_data = Some(heightmap_data);
            }

            if ui.button("Load Map").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                    .set_title("Select a heightmap image")
                    .pick_file()
                {
                    if let Ok(img) = image::open(&path) {
                        let gray = img.to_luma8();
                        let (w, h) = gray.dimensions();

                        self.config.width = w;
                        self.config.height = h;

                        let heightmap: Vec<f32> =
                            gray.pixels().map(|p| p[0] as f32 / 255.0).collect();

                        let mut preview = ImageBuffer::new(w, h);
                        for y in 0..h {
                            for x in 0..w {
                                let i = (y * w + x) as usize;
                                let h = heightmap[i];
                                let (r, g, b) =
                                    get_color_for_height(h as f64, self.config.sea_level);
                                preview.put_pixel(x, y, Rgba([r, g, b, 255]));
                            }
                        }

                        self.heightmap_data = Some(heightmap);
                        self.preview_image = Some(preview.clone());

                        let color_image = egui::ColorImage {
                            size: [w as usize, h as usize],
                            pixels: preview
                                .pixels()
                                .map(|p| egui::Color32::from_rgb(p[0], p[1], p[2]))
                                .collect(),
                        };
                        self.preview_texture = Some(ctx.load_texture(
                            "preview",
                            color_image,
                            egui::TextureOptions::default(),
                        ));
                    }
                }
            }
        });
    }

    fn render_water_settings(&mut self, _ui: &mut egui::Ui) { /* lake threshold, river toggles */
    }
    fn render_biome_settings(&mut self, _ui: &mut egui::Ui) { /* biome slider ranges */
    }
    fn render_object_settings(&mut self, _ui: &mut egui::Ui) { /* trees, building densities */
    }

    fn render_export_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("Export Options");

        if ui.button("Export Preview").clicked() {
            if let Some(preview) = &self.preview_image {
                let _ = preview.save("export_preview.png");
            }
        }

        if ui.button("Export Heightmap").clicked() {
            if let (Some(data), w, h) =
                (&self.heightmap_data, self.config.width, self.config.height)
            {
                use image::{GrayImage, Luma};
                let mut img = GrayImage::new(w, h);
                for (i, val) in data.iter().enumerate() {
                    let x = (i as u32) % w;
                    let y = (i as u32) / w;
                    let intensity = (val * 255.0).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Luma([intensity]));
                }
                let _ = img.save("export_heightmap.png");
            }
        }
    }
}

impl eframe::App for DayZMapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading(format!(
                    "Step {}:",
                    match self.current_step {
                        GenerationStep::Terrain => "1: Terrain",
                        GenerationStep::Water => "2: Water",
                        GenerationStep::Biomes => "3: Biomes",
                        GenerationStep::Objects => "4: Objects",
                        GenerationStep::Export => "5: Export",
                    }
                ));

                ui.separator();

                match self.current_step {
                    GenerationStep::Terrain => {
                        self.render_terrain_settings(ui, ctx);
                    }

                    GenerationStep::Water => {
                        self.render_water_settings(ui);
                    }

                    GenerationStep::Biomes => {
                        self.render_biome_settings(ui);
                    }

                    GenerationStep::Objects => {
                        self.render_object_settings(ui);
                    }

                    GenerationStep::Export => {
                        self.render_export_panel(ui);
                    }
                }

                egui::TopBottomPanel::bottom("nav_bar").show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        if !matches!(self.current_step, GenerationStep::Terrain) {
                            if ui.button("← Back").clicked() {
                                self.current_step = match self.current_step {
                                    GenerationStep::Water => GenerationStep::Terrain,
                                    GenerationStep::Biomes => GenerationStep::Water,
                                    GenerationStep::Objects => GenerationStep::Biomes,
                                    GenerationStep::Export => GenerationStep::Objects,
                                    _ => self.current_step,
                                };
                            }
                        }

                        if !matches!(self.current_step, GenerationStep::Export) {
                            if ui.button("Next →").clicked() {
                                self.current_step = match self.current_step {
                                    GenerationStep::Terrain => GenerationStep::Water,
                                    GenerationStep::Water => GenerationStep::Biomes,
                                    GenerationStep::Biomes => GenerationStep::Objects,
                                    GenerationStep::Objects => GenerationStep::Export,
                                    _ => self.current_step,
                                };
                            }
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.preview_texture {
                let available_size = ui.available_size();
                let image_size = texture.size_vec2();
                let scale = {
                    let w_ratio = available_size.x / image_size.x;
                    let h_ratio = available_size.y / image_size.y;
                    w_ratio.min(h_ratio).min(1.0)
                };
                ui.image(texture, image_size * scale);
            } else {
                ui.label("Press 'Generate Map' to create a new map preview.");
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "DayZ Map Generator",
        options,
        Box::new(|_cc| Box::new(DayZMapApp::default())),
    )
}
