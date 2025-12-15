#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai_suggest;
mod api;
mod app;
mod audit;
mod export;
mod i18n;
mod local_db;
mod schema_diff;
mod schema_explorer;
mod secure_storage;
mod settings_io;
mod sql_highlight;
mod sql_safety;
mod theme;
mod version;

use app::D1ManagerApp;

fn load_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([800.0, 600.0]);

    let viewport = if let Some(icon) = load_icon() {
        viewport.with_icon(std::sync::Arc::new(icon))
    } else {
        viewport
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "D1 Manager",
        options,
        Box::new(|cc| Ok(Box::new(D1ManagerApp::new(cc)))),
    )
}
