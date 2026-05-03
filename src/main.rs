#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod editors;
mod model;
mod sidebar;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OAI Enchant – OpenAPI Editor")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "OAI Enchant",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
