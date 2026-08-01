use egui::{Color32, RichText};

use crate::app::BloodAnalyzerApp;
use crate::model::{format_display_value, AnalysisResult, Panel, Parameter, Status, UnitSystem};
use crate::reference_data;

/// Columns are laid out in rows of at most this many, wrapping to additional
/// rows rather than squashing everything into one ever-narrower row.
const MAX_COLUMNS_PER_ROW: usize = 4;

pub fn show(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp) {
    let visible_panels: Vec<Panel> = Panel::ALL.into_iter().filter(|p| app.panel_selected(*p)).collect();

    if visible_panels.is_empty() {
        ui.label("No panels selected \u{2014} enable at least one panel above.");
        return;
    }

    for row in visible_panels.chunks(MAX_COLUMNS_PER_ROW) {
        ui.columns(row.len(), |columns| {
            for (col, panel) in columns.iter_mut().zip(row.iter()) {
                render_panel_column(col, app, *panel);
            }
        });
        ui.add_space(12.0);
    }
}

fn render_panel_column(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp, panel: Panel) {
    ui.heading(panel.label());
    ui.add_space(4.0);

    let unit_system = app.unit_system;

    for param in reference_data::PARAMETERS.iter().filter(|p| p.panel == panel) {
        ui.horizontal(|ui| {
            ui.add_sized([160.0, 18.0], egui::Label::new(param.name));
            let text = app.inputs.entry(param.id).or_default();
            ui.add(egui::TextEdit::singleline(text).desired_width(65.0));
            ui.label(param.unit_for(unit_system));
        });

        if let Some(result) = app.results.get(param.id) {
            render_result(ui, param, result, unit_system);
        }

        ui.add_space(2.0);
    }
}

fn render_result(ui: &mut egui::Ui, param: &Parameter, result: &AnalysisResult, unit_system: UnitSystem) {
    let color = super::status_color(result.status);
    let unit = param.unit_for(unit_system);
    let value = param.si_to_display(result.value, unit_system);
    let low = param.si_to_display(result.range_low, unit_system);
    let high = param.si_to_display(result.range_high, unit_system);

    let header = RichText::new(format!(
        "{}  [{}]  (range {}\u{2013}{} {})",
        format_display_value(value),
        result.status.label(),
        format_display_value(low),
        format_display_value(high),
        unit,
    ))
    .color(color)
    .strong();

    egui::CollapsingHeader::new(header)
        .id_salt(param.id)
        .show(ui, |ui| {
            ui.label(param.description);

            let meaning = match result.status {
                Status::High | Status::CriticalHigh => {
                    Some(("Possible causes of high result:", param.high_meaning))
                }
                Status::Low | Status::CriticalLow => {
                    Some(("Possible causes of low result:", param.low_meaning))
                }
                Status::Normal => None,
            };
            if let Some((label, text)) = meaning {
                ui.add_space(4.0);
                ui.label(RichText::new(label).strong());
                ui.label(text);
            }

            let suggestion = match result.status {
                Status::High | Status::CriticalHigh => param.suggestion_high,
                Status::Low | Status::CriticalLow => param.suggestion_low,
                Status::Normal => None,
            };
            if let Some(text) = suggestion {
                ui.add_space(4.0);
                ui.label(RichText::new("General lifestyle note (not medical advice \u{2014} talk to your doctor):").strong());
                ui.label(text);
            }

            if matches!(result.status, Status::CriticalLow | Status::CriticalHigh) {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "This value is outside the critical threshold and may warrant prompt clinical attention.",
                    )
                    .color(Color32::from_rgb(220, 50, 47))
                    .italics(),
                );
            }

            ui.add_space(4.0);
            ui.hyperlink_to("MedlinePlus: learn more", param.source_url);
        });
}
