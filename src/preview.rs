pub fn get_color_for_height(h: f64, sea_level: f64) -> (u8, u8, u8) {
    if h < sea_level * 0.6 {
        (0, 0, 100)
    } else if h < sea_level {
        (64, 164, 223)
    } else if h < 0.5 {
        (34, 139, 34)
    } else if h < 0.65 {
        (160, 82, 45)
    } else if h < 0.85 {
        (139, 137, 137)
    } else {
        (255, 250, 250)
    }
}
