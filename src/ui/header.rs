use egui::{Color32, RichText};

use crate::app::BloodAnalyzerApp;
use crate::model::{Panel, Sex, Status, UnitSystem};

pub fn show(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp) {
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.heading("Blood Analyzer");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // `fit_to_exact_size` forces the actual on-screen size; the
            // default fit mode instead sizes the image as a fraction of
            // whatever space happens to be available in the row at that
            // point in the layout, which is why `max_height` alone had no
            // visible effect no matter how high it was set.
            //
            // Sized to sit inline with the "Blood Analyzer" heading (18pt,
            // ~24-27px tall rendered) rather than ballooning the row height
            // and pushing everything below it further down the window.
            //
            // Uses a pre-downsampled 160x160 copy of the source 1024x1024
            // logo rather than scaling the full-size original at runtime —
            // egui's texture minification isn't mipmapped, so shrinking a
            // 1024px source down to ~32px on the fly aliased badly and
            // looked pixelated; downsampling once in advance avoids that.
            let logo =
                egui::Image::new(egui::include_image!("../../images/bloodtestlogo_small.png"))
                    .fit_to_exact_size(egui::Vec2::splat(32.0))
                    .sense(egui::Sense::click());
            if ui.add(logo).on_hover_text("About Blood Analyzer").clicked() {
                app.show_about_window = true;
            }
        });
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
    });

    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        ui.label("Panels:");
        for panel in Panel::ALL {
            let mut enabled = app.panel_selected(panel);
            if ui.checkbox(&mut enabled, panel.label()).changed() {
                app.set_panel_selected(panel, enabled);
            }
        }
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
            let abnormal = app
                .results
                .values()
                .filter(|r| r.status.is_abnormal())
                .count();
            ui.separator();
            if abnormal > 0 {
                ui.label(
                    RichText::new(format!(
                        "{abnormal} result(s) outside the standard reference range."
                    ))
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
