#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod app;
mod model;
mod reference_data;
mod ui;

use app::BloodAnalyzerApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 820.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Blood Analyzer",
        options,
        Box::new(|_cc| Ok(Box::new(BloodAnalyzerApp::default()))),
    )
}
