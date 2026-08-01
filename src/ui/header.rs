use egui::{Color32, RichText};

use crate::app::BloodAnalyzerApp;
use crate::model::{Panel, Sex, Status, UnitSystem};

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

        ui.label("Units:");
        let mut unit_system = app.unit_system;
        ui.selectable_value(&mut unit_system, UnitSystem::Si, "SI");
        ui.selectable_value(&mut unit_system, UnitSystem::UsConventional, "US");
        if unit_system != app.unit_system {
            app.set_unit_system(unit_system);
        }

        ui.separator();

        ui.label("Panels:");
        ui.checkbox(&mut app.show_cbc, Panel::Cbc.label());
        ui.checkbox(&mut app.show_cmp, Panel::Cmp.label());
        ui.checkbox(&mut app.show_lipid, Panel::Lipid.label());
    });

    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if ui.button(RichText::new("Analyze").strong()).clicked() {
            app.run_analysis();
        }
        if ui.button("Reference Data").clicked() {
            app.show_reference_window = !app.show_reference_window;
        }
        if ui.button("History").clicked() {
            app.show_history_window = !app.show_history_window;
        }
        if ui.button("Compare").clicked() {
            app.show_compare_window = !app.show_compare_window;
        }
        if ui.button("Save Results").clicked() {
            app.save_current_session();
        }
        if ui.button("Export PDF").clicked() {
            app.export_pdf();
        }

        if app.analyzed {
            let abnormal = app.results.values().filter(|r| r.status.is_abnormal()).count();
            ui.separator();
            if abnormal > 0 {
                ui.label(
                    RichText::new(format!("{abnormal} result(s) outside the standard reference range."))
                        .color(Color32::from_rgb(210, 140, 20))
                        .strong(),
                );
            } else if !app.results.is_empty() {
                ui.label(
                    RichText::new("All entered results are within the standard reference range.")
                        .color(Color32::from_rgb(60, 160, 70)),
                );
            }
            let critical = app
                .results
                .values()
                .filter(|r| matches!(r.status, Status::CriticalLow | Status::CriticalHigh))
                .count();
            if critical > 0 {
                ui.label(
                    RichText::new(format!("{critical} critical value(s)."))
                        .color(Color32::from_rgb(220, 50, 47))
                        .strong(),
                );
            }
        }
    });

    if let Some(message) = app.status_message.clone() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(message).italics());
            if ui.small_button("\u{2715}").clicked() {
                app.status_message = None;
            }
        });
    }

    ui.add_space(4.0);
}
