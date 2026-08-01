use egui::RichText;

use crate::app::BloodAnalyzerApp;

pub fn show(ctx: &egui::Context, app: &mut BloodAnalyzerApp) {
    let mut open = app.show_about_window;

    egui::Window::new("About Blood Analyzer")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Developed & Composed by Teddy Jones").strong());
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Email:");
                ui.hyperlink_to(
                    "teddyjones@outlook.co.nz",
                    "mailto:teddyjones@outlook.co.nz",
                );
            });
            ui.add_space(6.0);
            ui.hyperlink_to(
                "GitHub Repository",
                "https://github.com/TheZeekA/Blood-Analyzer",
            );
            ui.add_space(4.0);
        });

    app.show_about_window = open;
}
