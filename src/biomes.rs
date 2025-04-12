use crate::config::{MapConfig, BiomeConfig};
use noise::{NoiseFn, Perlin, Seedable};

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
    if elev < sea_level * 0.8 {
        Biome::Ocean
    } else if elev < sea_level {
        if humidity > 0.5 {
            Biome::Beach
        } else {
            Biome::Ocean
        }
    } else if elev < sea_level * 1.2 {
        if humidity > 0.5 {
            if temp > 0.5 {
                Biome::Plains
            } else {
                Biome::Forest
            }
        } else {
            if temp > 0.5 {
                Biome::Desert
            } else {
                Biome::Swamp
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
            if temp > 0.5 {
                Biome::Desert
            } else {
                Biome::Swamp
            }
        }
    } else {
        if humidity > 0.5 {
            Biome::Snow
        } else {
            Biome::Desert
        }
    }
}

pub fn generate_biome_map(map_config: &MapConfig, biome_config: &BiomeConfig, heightmap: &[f32]) -> Vec<Biome> {

    let width = map_config.width as usize;
    let height = map_config.height as usize;
    let size = width * height;
    let mut biomes = Vec::with_capacity(size);
    let sea_level = map_config.sea_level as f32;
    let sea_level = sea_level.clamp(0.0, 1.0);
    let seed = map_config.seed;

    // add to seed to avoid colliSsions with heightmap noise
    let perlin_temp: Perlin = Perlin::new().set_seed(seed + 1000);
    let perlin_hum: Perlin = Perlin::new().set_seed(seed + 2000);

    let min_temp = (biome_config.base_temperature - biome_config.temperature_variation / 2.0 )as f64;
    let max_temp = (biome_config.base_temperature + biome_config.temperature_variation / 2.0) as f64;
    let min_hum = (biome_config.base_humidity - biome_config.humidity_variation / 2.0) as f64;
    let max_hum = (biome_config.base_humidity + biome_config.humidity_variation / 2.0) as f64;

    for i in 0..(width * height) as usize {
        let h = heightmap[i];

        let x = (i % width as usize) as f32 / width as f32;
        let y = (i / width as usize) as f32 / height as f32;



        // calculate slope based on neighboring pixels
        // flat areas have a slope of 0, steep areas have a slope of 1
        // assuming 1 pixel = 1m and height of the map is 1km
        // 0.5 means 45 degrees, 1 means 90 degrees (impossible in this case, but close values are possible)
        let mut slope = 0.0;
        if i % width as usize != 0 && i / width as usize != 0 && i % width as usize != (width - 1) as usize && i / width as usize != (height - 1) as usize {
            let left = heightmap[i - 1];
            let right = heightmap[i + 1];
            let up = heightmap[i - width as usize];
            let down = heightmap[i + width as usize];

            slope = ((left - h).abs() + (right - h).abs() + (up - h).abs() + (down - h).abs()) / 4.0;

            slope = slope * 1000.0; // scale to 1km
            // slope 0 -> 0
            // slope 1 -> 0.5
            // slope 2 -> 0.75
            // ...
            let angle_rad = slope.atan2(1.0);
            let norm_slope = angle_rad / std::f32::consts::PI * 2.0; // normalize to 0..1
            slope = norm_slope.clamp(0.0, 1.0);
        }
        // 
        // use noise to generate temperature and humidity
        let mut temp = (perlin_temp.get([x as f64 / biome_config.temperature_variation as f64, y as f64 / biome_config.temperature_variation as f64]) + 1.0) / 2.0;
        let mut humidity = (perlin_hum.get([x as f64 / biome_config.humidity_variation as f64, y as f64 / biome_config.humidity_variation as f64]) + 1.0) / 2.0;

        temp = temp * (max_temp - min_temp) + min_temp;
        humidity = humidity * (max_hum - min_hum) + min_hum;


        // apply biome blend factor to temperature and humidity
        // TODO

        let biome = choose_biome(temp, humidity, h, sea_level, slope);


        biomes.push(biome);
    }

    biomes
}