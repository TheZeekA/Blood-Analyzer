use egui::{Color32, RichText};

use crate::app::BloodAnalyzerApp;
use crate::model::{Panel, Sex};

pub fn show(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp) {
    ui.add_space(6.0);
    ui.heading("Blood Analyzer");

    egui::Frame::new()
        .fill(Color32::from_rgb(90, 60, 10))
        .inner_margin(egui::Margin::same(8))
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new(
                    "Educational reference tool only \u{2014} not a substitute for professional \
                     medical diagnosis or advice. Reference ranges are general adult values and \
                     may differ from your lab's own reference ranges; always confirm against the \
                     range printed on the lab report.",
                )
                .color(Color32::WHITE)
                .small(),
            );
        });

    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label("Sex:");
        ui.selectable_value(&mut app.sex, Sex::Male, "Male");
        ui.selectable_value(&mut app.sex, Sex::Female, "Female");

        ui.separator();

        ui.label("Panels:");
        ui.checkbox(&mut app.show_cbc, Panel::Cbc.label());
        ui.checkbox(&mut app.show_cmp, Panel::Cmp.label());
        ui.checkbox(&mut app.show_lipid, Panel::Lipid.label());
    });
    ui.add_space(6.0);
}
