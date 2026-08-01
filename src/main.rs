#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod app;
mod history;
mod model;
mod pdf_export;
mod reference_data;
mod settings;
mod ui;

use app::BloodAnalyzerApp;

const ICON_PNG: &[u8] = include_bytes!("../images/bloodtestlogo.png");

fn main() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(ICON_PNG).expect("failed to decode app icon");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 950.0]).with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Blood Analyzer",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(BloodAnalyzerApp::new()))
        }),
    )
}
