use egui::Color32;

use crate::app::BloodAnalyzerApp;
use crate::model::{Parameter, RangeSpec, Sex, UnitSystem, format_display_value};
use crate::reference_data;
use crate::settings;

pub fn show(ctx: &egui::Context, app: &mut BloodAnalyzerApp) {
    let mut open = app.show_reference_window;
    egui::Window::new("Reference Data")
        .open(&mut open)
        .default_size([620.0, 520.0])
        .vscroll(true)
        .show(ctx, |ui| {
            ui.label(
                "Editable adult reference ranges. Edits apply immediately and are saved on this \
                 machine. Source: MedlinePlus (NIH/National Library of Medicine).",
            );
            ui.add_space(6.0);
            if ui.button("Reset all to defaults").clicked() {
                app.range_overrides.reset_all();
                app.range_overrides.save();
                app.reference_edit_buffers.clear();
            }
            ui.separator();

            let unit_system = app.unit_system;
            for param in reference_data::PARAMETERS {
                ui.push_id(param.id, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(param.name);
                        ui.hyperlink_to("source", param.source_url);
                    });

                    let is_overridden = app.range_overrides.get(param.id).is_some();

                    match param.range {
                        RangeSpec::Fixed { .. } => {
                            ui.horizontal(|ui| {
                                render_fixed_row(ui, app, param, unit_system);
                            });
                        }
                        RangeSpec::BySex { .. } => {
                            render_by_sex_rows(ui, app, param, unit_system);
                        }
                    }

                    if is_overridden {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                Color32::from_rgb(210, 140, 20),
                                "Modified from default",
                            );
                            if ui.small_button("Reset").clicked() {
                                app.range_overrides.reset(param.id);
                                app.range_overrides.save();
                                remove_buffers(app, param.id);
                            }
                        });
                    }
                    ui.separator();
                });
            }
        });
    app.show_reference_window = open;
}

fn text_field(
    ui: &mut egui::Ui,
    app: &mut BloodAnalyzerApp,
    key: &str,
    default_text: impl FnOnce() -> String,
) -> (String, bool) {
    let buf = app
        .reference_edit_buffers
        .entry(key.to_string())
        .or_insert_with(default_text);
    let response = ui.add(egui::TextEdit::singleline(buf).desired_width(55.0));
    (buf.clone(), response.changed())
}

fn render_fixed_row(
    ui: &mut egui::Ui,
    app: &mut BloodAnalyzerApp,
    param: &Parameter,
    unit_system: UnitSystem,
) {
    let effective = settings::effective_range(param, Sex::Male, &app.range_overrides);
    let low_default = format_display_value(param.si_to_display(effective.0, unit_system));
    let high_default = format_display_value(param.si_to_display(effective.1, unit_system));

    let low_key = format!("{}_low", param.id);
    let high_key = format!("{}_high", param.id);

    let (low_text, changed_low) = text_field(ui, app, &low_key, || low_default);
    ui.label("\u{2013}");
    let (high_text, changed_high) = text_field(ui, app, &high_key, || high_default);
    ui.label(param.unit_for(unit_system));

    if (changed_low || changed_high)
        && let (Ok(low_disp), Ok(high_disp)) = (
            low_text.trim().parse::<f64>(),
            high_text.trim().parse::<f64>(),
        )
    {
        let low_si = param.display_to_si(low_disp, unit_system);
        let high_si = param.display_to_si(high_disp, unit_system);
        app.range_overrides.set(
            param.id,
            RangeSpec::Fixed {
                low: low_si,
                high: high_si,
            },
        );
        app.range_overrides.save();
    }
}

fn render_by_sex_rows(
    ui: &mut egui::Ui,
    app: &mut BloodAnalyzerApp,
    param: &Parameter,
    unit_system: UnitSystem,
) {
    for (sex, label) in [(Sex::Male, "Male"), (Sex::Female, "Female")] {
        ui.horizontal(|ui| {
            ui.label(label);
            let effective = settings::effective_range(param, sex, &app.range_overrides);
            let low_default = format_display_value(param.si_to_display(effective.0, unit_system));
            let high_default = format_display_value(param.si_to_display(effective.1, unit_system));

            let low_key = format!("{}_{:?}_low", param.id, sex);
            let high_key = format!("{}_{:?}_high", param.id, sex);

            let (low_text, changed_low) = text_field(ui, app, &low_key, || low_default);
            ui.label("\u{2013}");
            let (high_text, changed_high) = text_field(ui, app, &high_key, || high_default);
            ui.label(param.unit_for(unit_system));

            if (changed_low || changed_high)
                && let (Ok(low_disp), Ok(high_disp)) = (
                    low_text.trim().parse::<f64>(),
                    high_text.trim().parse::<f64>(),
                )
            {
                let low_si = param.display_to_si(low_disp, unit_system);
                let high_si = param.display_to_si(high_disp, unit_system);
                let other_sex = if sex == Sex::Male {
                    Sex::Female
                } else {
                    Sex::Male
                };
                let other_range = settings::effective_range(param, other_sex, &app.range_overrides);
                let new_range = match sex {
                    Sex::Male => RangeSpec::BySex {
                        male: (low_si, high_si),
                        female: other_range,
                    },
                    Sex::Female => RangeSpec::BySex {
                        male: other_range,
                        female: (low_si, high_si),
                    },
                };
                app.range_overrides.set(param.id, new_range);
                app.range_overrides.save();
            }
        });
    }
}

fn remove_buffers(app: &mut BloodAnalyzerApp, param_id: &str) {
    let prefix = format!("{param_id}_");
    app.reference_edit_buffers
        .retain(|k, _| !k.starts_with(&prefix));
}
