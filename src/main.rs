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

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "D1 Manager",
        options,
        Box::new(|cc| Ok(Box::new(D1ManagerApp::new(cc)))),
    )
}
