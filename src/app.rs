use crate::{config::MapConfig, preview::get_color_for_height, terrain::generate_map};
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

    fn render_refine_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Refine Heightmap");
        ui.separator();

        /*

        // Curve controls
        ui.collapsing("Height Curve", |ui| {
            // Control points UI here
            // Presets: Linear, Steep Peaks, Flatlands, etc.
        });

        // Other refinement sliders
        ui.checkbox(&mut self.config.apply_terracing, "Terracing");
        if self.config.apply_terracing {
            ui.add(
                egui::Slider::new(&mut self.config.terrace_levels, 2..=20).text("Terrace Levels"),
            );
        }

        if ui.button("Apply Refinement").clicked() {
            self.refine_heightmap();
        }
        */
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
