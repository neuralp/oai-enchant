#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod editors;
mod lint;
mod logo;
mod model;
mod sidebar;

fn main() -> eframe::Result<()> {
    let icon = std::sync::Arc::new(logo::make_icon());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OAI Enchant – OpenAPI Editor")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "OAI Enchant",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
