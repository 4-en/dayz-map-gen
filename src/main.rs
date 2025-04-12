mod app;
mod config;
mod terrain;
mod preview;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "DayZ Map Generator",
        options,
        Box::new(|_cc| Box::new(app::DayZMapApp::default())),
    )
}
