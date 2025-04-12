use crate::{preview::get_color_for_height, terrain::generate_map, refiner::refine_heightmap};
use crate::config::{BiomeConfig, MapConfig, RefinerConfig, WaterConfig};
use crate::biomes::generate_biome_map;
use eframe::egui;
use image::{ImageBuffer, Rgba};

enum GenerationStep {
    Terrain,
    Refinement,
    Water,
    Biomes,
    Objects,
    Export,
}

impl Clone for GenerationStep {
    fn clone(&self) -> Self {
        match self {
            GenerationStep::Terrain => GenerationStep::Terrain,
            GenerationStep::Refinement => GenerationStep::Refinement,
            GenerationStep::Water => GenerationStep::Water,
            GenerationStep::Biomes => GenerationStep::Biomes,
            GenerationStep::Objects => GenerationStep::Objects,
            GenerationStep::Export => GenerationStep::Export,
        }
    }
}

impl Copy for GenerationStep {}

/// The main application structure holding the configuration and preview texture.
pub struct DayZMapApp {
    current_step: GenerationStep,
    config: MapConfig,
    refiner_config: RefinerConfig,
    biome_config: BiomeConfig,
    water_config: WaterConfig,
    preview_texture: Option<egui::TextureHandle>,
    preview_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    heightmap_data: Option<Vec<f32>>,
    biome_map: Option<Vec<u8>>,
}

impl Default for DayZMapApp {
    fn default() -> Self {
        Self {
            current_step: GenerationStep::Terrain,
            config: MapConfig::default(),
            refiner_config: RefinerConfig::default(),
            biome_config: BiomeConfig::default(),
            water_config: WaterConfig::default(),
            preview_texture: None,
            preview_image: None,
            heightmap_data: None,
            biome_map: None,
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

        ui.horizontal(|ui| {
            ui.label("Quick Resize:");
            for &size in [0.25, 0.5, 2.0, 4.0].iter() {
                if ui.button(format!("{:.2}x", size)).clicked() {
                    self.config.width = (self.config.width as f32 * size) as u32;
                    self.config.height = (self.config.height as f32 * size) as u32;
                    self.config.scale_base = (self.config.scale_base as f32 * size) as f64;
                    self.config.scale_mid = (self.config.scale_mid as f32 * size) as f64;
                    self.config.scale_detail = (self.config.scale_detail as f32 * size) as f64;


                }
            }
        });

        ui.separator();

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
        ui.add(egui::Slider::new(&mut self.config.scale_base, 10.0..=10000.0).text("Scale").clamp_to_range(false));
        ui.add(egui::Slider::new(&mut self.config.amp_base, 0.0..=2.0).text("Amp").clamp_to_range(false));

        ui.label("Mid Noise");
        ui.add(egui::Slider::new(&mut self.config.scale_mid, 10.0..=1000.0).text("Scale").clamp_to_range(false));
        ui.add(egui::Slider::new(&mut self.config.amp_mid, 0.0..=2.0).text("Amp").clamp_to_range(false));

        ui.label("Detail Noise");
        ui.add(egui::Slider::new(&mut self.config.scale_detail, 5.0..=100.0).text("Scale").clamp_to_range(false));
        ui.add(egui::Slider::new(&mut self.config.amp_detail, 0.0..=2.0).text("Amp").clamp_to_range(false));

        ui.separator();
        ui.label("Overlay Generation");
        ui.add(egui::Slider::new(&mut self.config.overlay, 0.0..=100.0).text("Overlay Strength").clamp_to_range(false));

        ui.horizontal(|ui| {
            if ui.button("Generate Map").clicked() {
                let seed = if self.config.use_random_seed {
                    let new_seed = rand::random::<u32>();
                    self.config.seed = new_seed;
                    new_seed
                } else {
                    self.config.seed
                };

                let (color_image, preview_img, heightmap_data) = generate_map(&self.config, seed, &self.heightmap_data);
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

    fn render_refine_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {

        // Curve controls
        ui.collapsing("Height Curve", |ui| {
            // Control points UI here
            // Presets: Linear, Steep Peaks, Flatlands, etc.
        });

        ui.label("Sea Level:");
        ui.add(egui::Slider::new(&mut self.config.sea_level, 0.0..=1.0).text("Sea Level"));

        ui.label("Height Offset:");
        ui.add(egui::Slider::new(&mut self.refiner_config.height_offset, -1.0..=1.0).text("Height Offset"));

        // coeff for height (height * coeff + offset)
        ui.label("Height Coefficient:");
        ui.add(egui::Slider::new(&mut self.refiner_config.height_coeff, 0.0..=10.0).text("Height Coefficient"));

        // exp for height (height ^ exp + offset)
        ui.label("Height Exponent:");
        ui.add(egui::Slider::new(&mut self.refiner_config.height_exponent, 0.0..=10.0).text("Height Exponent"));

        // smoothness of the heightmap (0.0 = no smoothing, 1.0 = full smoothing)
        ui.label("Smoothing Factor:");
        ui.add(egui::Slider::new(&mut self.refiner_config.smoothness, 0.0..=1.0).text("Smoothing Factor"));

        // TODO: connect this and add following features:
        // - smoothing factor (taking into account cliffs and other features)
        // - Curve points (add/remove points, adjust curve shape, similar to photoshop/gimp curves)
        // - Paint map overlay (load a texture and use it to modify the heightmap using "sculpting" tools like "raise/lower, smooth, etc.)"
        // - "live" preview using smaller texture (512x512) and a "preview" button to generate the full heightmap
        // - "Apply" button to apply the changes to the heightmap and update the preview


        if ui.button("Apply Refinement").clicked() {
            let refined_heightmap = refine_heightmap(
                self.heightmap_data.as_ref().unwrap(),
                &self.refiner_config,
                &self.config,
            );
            let (w, h) = (self.config.width, self.config.height);
            let mut preview = ImageBuffer::new(w, h);
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) as usize;
                    let h = refined_heightmap[i];
                    let (r, g, b) = get_color_for_height(h as f64, self.config.sea_level);
                    preview.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
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
            self.heightmap_data = Some(refined_heightmap);

        }
    }


    fn render_biome_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) { /* biome slider ranges */


        ui.checkbox(&mut self.biome_config.use_random_seed, "Use Random Seed");

        if !self.biome_config.use_random_seed {
            ui.label("Seed:");
            ui.add(egui::DragValue::new(&mut self.biome_config.seed).speed(1));
        } else {
            ui.label(format!("Random Seed: {}", self.biome_config.seed));
        }

        ui.separator();
        ui.label("Biome Scale:");
        ui.add(egui::Slider::new(&mut self.biome_config.scale, 0.0..=20000.0).text("Biome Scale").clamp_to_range(false));

        ui.label("Base Temperature:");
        ui.add(egui::Slider::new(&mut self.biome_config.base_temperature, -10.0..=40.0).text("Base Temperature"));

        ui.label("Temperature Variation:");
        ui.add(egui::Slider::new(&mut self.biome_config.temperature_variation, 0.0..=100.0).text("Temperature Variation"));

        ui.label("Base Humidity:");
        ui.add(egui::Slider::new(&mut self.biome_config.base_humidity, 0.0..=100.0).text("Base Humidity"));

        ui.label("Humidity Variation:");
        ui.add(egui::Slider::new(&mut self.biome_config.humidity_variation, 0.0..=100.0).text("Humidity Variation"));

        ui.label("Biome Blend Factor:");
        ui.add(egui::Slider::new(&mut self.biome_config.biome_blend_factor, 0.0..=100.0).text("Biome Blend Factor"));


        if ui.button("Generate Biome Map").clicked() {
            if let Some(heightmap) = &self.heightmap_data {

                let mut seed = self.biome_config.seed;
                if self.biome_config.use_random_seed {
                    seed = rand::random::<u32>();
                    self.biome_config.seed = seed;
                }

                let (color_image, preview, biome)  = generate_biome_map(&self.config, &self.biome_config, heightmap, seed);
                self.biome_map = Some(biome);
                self.preview_texture = Some(ctx.load_texture("preview", color_image, egui::TextureOptions::default()));
                self.preview_image = Some(preview);


            } else {
                ui.label("Please load a heightmap first.");
            }
        }

    }

    fn render_water_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) { /* water slider ranges */
        ui.checkbox(&mut self.water_config.use_random_seed, "Use Random Seed");

        if !self.water_config.use_random_seed {
            ui.label("Seed:");
            ui.add(egui::DragValue::new(&mut self.water_config.seed).speed(1));
        } else {
            ui.label(format!("Random Seed: {}", self.water_config.seed));
        }

        ui.separator();
        ui.label("Lake Chance:");
        ui.add(egui::Slider::new(&mut self.water_config.lake_chance, 0.0..=1.0).text("Lake Chance"));

        ui.label("Lake Size:");
        ui.add(egui::Slider::new(&mut self.water_config.lake_size, 0.0..=1.0).text("Lake Size"));

        ui.label("River Count:");
        ui.add(egui::Slider::new(&mut self.water_config.river_count, 0..=100).text("River Count"));

        ui.label("River Width:");
        ui.add(egui::Slider::new(&mut self.water_config.river_width, 0.0..=1.0).text("River Width"));

        ui.label("River Momentum:");
        ui.add(egui::Slider::new(&mut self.water_config.river_momentum, 0.0..=1.0).text("River Momentum"));

        ui.label("River Direction Variation:");
        ui.add(egui::Slider::new(&mut self.water_config.river_direction_variation, 0.0..=1.0).text("River Direction Variation"));

        ui.label("Lake Drainage:");
        ui.add(egui::Slider::new(&mut self.water_config.lake_drainage, 0.0..=1.0).text("Lake Drainage"));


        if ui.button("Generate Water Map").clicked() {
            if let Some(heightmap) = &self.heightmap_data {

                let mut seed = self.water_config.seed;
                if self.water_config.use_random_seed {
                    seed = rand::random::<u32>();
                    self.water_config.seed = seed;
                }
                /*
                let (color_image, preview, biome)  = generate_biome_map(&self.config, &self.biome_config, heightmap, seed);
                self.biome_map = Some(biome);
                self.preview_texture = Some(ctx.load_texture("preview", color_image, egui::TextureOptions::default()));
                self.preview_image = Some(preview);
                */


            } else {
                ui.label("Please load a heightmap first.");
            }
        }

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
                        GenerationStep::Refinement => "2: Refinement",
                        GenerationStep::Water => "4: Water",
                        GenerationStep::Biomes => "3: Biomes",
                        GenerationStep::Objects => "5: Objects",
                        GenerationStep::Export => "6: Export",
                    }
                ));

                ui.separator();

                match self.current_step {
                    GenerationStep::Terrain => {
                        self.render_terrain_settings(ui, ctx);
                    }

                    GenerationStep::Refinement => {
                        self.render_refine_settings(ui, ctx);
                    }

                    GenerationStep::Water => {
                        self.render_water_settings(ui, ctx);
                    }

                    GenerationStep::Biomes => {
                        self.render_biome_settings(ui, ctx);
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
                            if ui.button("Back").clicked() {
                                self.current_step = match self.current_step {
                                    GenerationStep::Refinement => GenerationStep::Terrain,
                                    GenerationStep::Biomes => GenerationStep::Refinement,
                                    GenerationStep::Water => GenerationStep::Biomes,
                                    GenerationStep::Objects => GenerationStep::Water,
                                    GenerationStep::Export => GenerationStep::Objects,
                                    _ => self.current_step,
                                };
                            }
                        }

                        if !matches!(self.current_step, GenerationStep::Export) {
                            if ui.button("Next").clicked() {
                                self.current_step = match self.current_step {
                                    GenerationStep::Terrain => GenerationStep::Refinement,
                                    GenerationStep::Refinement => GenerationStep::Biomes,
                                    GenerationStep::Biomes => GenerationStep::Water,
                                    GenerationStep::Water => GenerationStep::Objects,
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
                    let scaled_size = image_size * scale;
            
                    // Center the image using manual layout
                    ui.vertical_centered(|ui| {
                        ui.add_space((available_size.y - scaled_size.y).max(0.0) / 2.0); // vertical centering
                        ui.horizontal_centered(|ui| {
                            ui.image(texture, scaled_size);
                        });
                    });
                } else {
                    ui.label("Press 'Generate Map' to create a new map preview.");
                }
            });
    }
}
