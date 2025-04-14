use std::fs::File;
use std::io::{BufWriter, Write};

pub fn export_heightmap_to_asc(
    heightmap: &[f32],
    width: u32,
    height: u32,
    filename: &str,
    min_elevation: f32,
    max_elevation: f32,
) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "ncols         {}", width)?;
    writeln!(writer, "nrows         {}", height)?;
    writeln!(writer, "xllcorner     0.0")?;
    writeln!(writer, "yllcorner     0.0")?;
    writeln!(writer, "cellsize      1.0")?;
    writeln!(writer, "NODATA_value  -9999")?;

    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            let val = heightmap[i];
            let elevation = min_elevation + val * (max_elevation - min_elevation);
            write!(writer, "{:.2} ", elevation)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}