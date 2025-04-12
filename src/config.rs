#[derive(Debug, Clone)]
pub struct MapConfig {
    pub width: u32,
    pub height: u32,
    pub scale_base: f64,
    pub amp_base: f64,
    pub scale_mid: f64,
    pub amp_mid: f64,
    pub scale_detail: f64,
    pub amp_detail: f64,
    pub seed: u32,
    pub use_random_seed: bool,
    pub island_mode: bool,
    pub island_border: f64,
    pub island_curve: f64,
    pub sea_level: f64,
    pub mountainous: f64,
}

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
            mountainous: 1.0,
        }
    }
}

pub struct RefinerConfig {
    pub height_offset: f32,
    pub height_coeff: f32,
    pub height_exponent: f32,
    pub smoothness: f32,
    pub curve_points: Option<Vec<(f32, f32)>>,
    pub paint_map_overlay: Option<Vec<f32>>,
}

impl Default for RefinerConfig {
    fn default() -> Self {
        Self {
            height_offset: 0.0,
            height_coeff: 1.0,
            height_exponent: 1.0,
            smoothness: 0.0,
            curve_points: None,
            paint_map_overlay: None,
        }
    }
}

pub struct BiomeConfig {
    pub base_temperature: f32,
    pub base_humidity: f32,
    pub temperature_variation: f32,
    pub humidity_variation: f32,
    pub biome_blend_factor: f32,
    pub scale: f64,
    pub seed: u32,
    pub use_random_seed: bool,
}

impl Default for BiomeConfig {
    fn default() -> Self {
        Self {
            base_temperature: 15.0,
            base_humidity: 50.0,
            temperature_variation: 20.0,
            humidity_variation: 20.0,
            biome_blend_factor: 0.5,
            scale: 10000.0,
            seed: 12345,
            use_random_seed: true,
        }
    }
}

pub struct WaterConfig {
    pub lake_chance: f32,
    pub lake_size: f32,
    pub river_count: u32,
    pub river_width: f32,
    pub river_momentum: f32,
    pub river_direction_variation: f32,
    pub lake_drainage: f32,
    pub seed: u32,
    pub use_random_seed: bool,
}

impl Default for WaterConfig {
    fn default() -> Self {
        Self {
            lake_chance: 0.1,
            lake_size: 0.1,
            river_count: 10,
            river_width: 0.05,
            river_momentum: 0.5,
            river_direction_variation: 0.1,
            lake_drainage: 0.1,
            seed: 32345,
            use_random_seed: true,
        }
    }
}
