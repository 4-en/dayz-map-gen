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
