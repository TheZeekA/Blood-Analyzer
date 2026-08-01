use crate::analysis;
use crate::app::BloodAnalyzerApp;
use crate::history::SavedSession;
use crate::model::format_display_value;
use crate::reference_data;
use crate::settings;

pub fn show(ctx: &egui::Context, app: &mut BloodAnalyzerApp) {
    let mut open = app.show_compare_window;

    egui::Window::new("Compare Results")
        .open(&mut open)
        .default_size([640.0, 520.0])
        .vscroll(true)
        .show(ctx, |ui| {
            if app.history.0.len() < 2 {
                ui.label("Save at least two result sets to compare them.");
                return;
            }

            let sorted = app.history.sorted_indices();
            let session_labels: Vec<(usize, String)> =
                sorted.iter().map(|&idx| (idx, app.history.0[idx].label())).collect();

            ui.horizontal(|ui| {
                ui.label("Session A:");
                let selected = app
                    .compare_a
                    .and_then(|i| session_labels.iter().find(|(idx, _)| *idx == i))
                    .map(|(_, l)| l.clone())
                    .unwrap_or_else(|| "Select...".to_string());
                egui::ComboBox::from_id_salt("compare_a").selected_text(selected).show_ui(ui, |ui| {
                    for (idx, label) in &session_labels {
                        ui.selectable_value(&mut app.compare_a, Some(*idx), label.clone());
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label("Session B:");
                let selected = app
                    .compare_b
                    .and_then(|i| session_labels.iter().find(|(idx, _)| *idx == i))
                    .map(|(_, l)| l.clone())
                    .unwrap_or_else(|| "Select...".to_string());
                egui::ComboBox::from_id_salt("compare_b").selected_text(selected).show_ui(ui, |ui| {
                    for (idx, label) in &session_labels {
                        ui.selectable_value(&mut app.compare_b, Some(*idx), label.clone());
                    }
                });
            });

            ui.separator();

            if let (Some(a_idx), Some(b_idx)) = (app.compare_a, app.compare_b) {
                render_comparison(ui, app, a_idx, b_idx);
            } else {
                ui.label("Select two saved result sets above to compare.");
            }
        });

    app.show_compare_window = open;
}

fn render_comparison(ui: &mut egui::Ui, app: &BloodAnalyzerApp, a_idx: usize, b_idx: usize) {
    let session_a = &app.history.0[a_idx];
    let session_b = &app.history.0[b_idx];
    let unit_system = app.unit_system;

    egui::Grid::new("compare_grid").num_columns(4).striped(true).show(ui, |ui| {
        ui.strong("Marker");
        ui.strong(session_a.label());
        ui.strong(session_b.label());
        ui.strong("Change");
        ui.end_row();

        for param in reference_data::PARAMETERS {
            let a_val = session_a.values.get(param.id);
            let b_val = session_b.values.get(param.id);
            if a_val.is_none() && b_val.is_none() {
                continue;
            }

            ui.label(param.name);
            render_cell(ui, app, param, a_val.copied(), session_a);
            render_cell(ui, app, param, b_val.copied(), session_b);

            match (a_val, b_val) {
                (Some(&a), Some(&b)) => {
                    let diff_si = b - a;
                    let arrow = if diff_si.abs() < 1e-9 {
                        "\u{2192}"
                    } else if diff_si > 0.0 {
                        "\u{2191}"
                    } else {
                        "\u{2193}"
                    };
                    let diff_display = param.si_to_display(diff_si, unit_system).abs();
                    ui.label(format!("{arrow} {}", format_display_value(diff_display)));
                }
                _ => {
                    ui.label("\u{2014}");
                }
            }
            ui.end_row();
        }
    });
}

fn render_cell(
    ui: &mut egui::Ui,
    app: &BloodAnalyzerApp,
    param: &crate::model::Parameter,
    value: Option<f64>,
    session: &SavedSession,
) {
    match value {
        Some(si_value) => {
            let range = settings::effective_range(param, session.sex, &app.range_overrides);
            let result = analysis::analyze(si_value, range, param.critical);
            let display = param.si_to_display(si_value, app.unit_system);
            ui.colored_label(
                super::status_color(result.status),
                format!("{} {}", format_display_value(display), param.unit_for(app.unit_system)),
            );
        }
        None => {
            ui.label("\u{2014}");
        }
    }
}
