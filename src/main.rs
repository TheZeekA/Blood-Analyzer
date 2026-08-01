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

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 950.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Blood Analyzer",
        options,
        Box::new(|_cc| Ok(Box::new(BloodAnalyzerApp::new()))),
    )
}
