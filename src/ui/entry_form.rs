use egui::{Color32, RichText};

use crate::app::BloodAnalyzerApp;
use crate::model::{AnalysisResult, Panel, Parameter, Status, UnitSystem, format_display_value};
use crate::reference_data;

/// Floor used only when no panel is visible; real columns are sized from
/// actual content width (see `panel_min_width`) so nothing gets clipped.
const MIN_COLUMN_WIDTH_FLOOR: f32 = 280.0;
const MAX_COLUMNS: usize = 4;
const CARD_PADDING: f32 = 20.0; // 10px inner margin on each side
const ROW_ITEM_SPACING: f32 = 20.0; // two horizontal gaps between label/input/unit

const ROW_HEIGHT: f32 = 20.0;
const INPUT_WIDTH: f32 = 68.0;
const LABEL_MIN_WIDTH: f32 = 110.0;
const LABEL_MAX_WIDTH: f32 = 300.0;

pub fn show(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp) {
    let visible_panels: Vec<Panel> = Panel::ALL
        .into_iter()
        .filter(|p| app.panel_selected(*p))
        .collect();

    if visible_panels.is_empty() {
        ui.label("No panels selected \u{2014} enable at least one panel above.");
        return;
    }

    // Size columns from the actual widest content among the panels being
    // shown (longest label + widest unit string in the active unit system),
    // not a flat guess — a fixed constant was too narrow for panels like CMP
    // (long names such as "Alanine Aminotransferase (ALT)" plus wide units
    // like "mL/min/1.73m²"), which clipped the rightmost column.
    let min_col_width = visible_panels
        .iter()
        .map(|p| panel_min_width(ui, *p, app.unit_system))
        .fold(MIN_COLUMN_WIDTH_FLOOR, f32::max);

    // Reserve a little width for the vertical scrollbar, which isn't accounted
    // for in `available_width` until after it's decided whether it's needed.
    let usable_width = (ui.available_width() - 24.0).max(0.0);
    let num_columns = ((usable_width / min_col_width).floor() as usize).clamp(1, MAX_COLUMNS);

    // Masonry-style packing: each panel goes into whichever column currently
    // has the least content (by marker count, as a proxy for card height),
    // so columns stack panels back-to-back with no forced row breaks —
    // unlike a row-locked grid, a short card never leaves a gap under it
    // waiting for a taller neighbor in the same row to finish.
    let mut column_load = vec![0usize; num_columns];
    let mut columns: Vec<Vec<Panel>> = vec![Vec::new(); num_columns];
    for panel in visible_panels {
        let weight = reference_data::PARAMETERS
            .iter()
            .filter(|p| p.panel == panel)
            .count();
        let (idx, _) = column_load
            .iter()
            .enumerate()
            .min_by_key(|&(_, &load)| load)
            .unwrap();
        columns[idx].push(panel);
        column_load[idx] += weight;
    }

    ui.columns(num_columns, |cols| {
        for (col_ui, panels) in cols.iter_mut().zip(columns.iter()) {
            for panel in panels {
                render_panel_card(col_ui, app, *panel);
                col_ui.add_space(10.0);
            }
        }
    });
}

/// Width of the widest parameter name in this panel, so every input box in
/// the column lines up regardless of label length.
fn label_column_width(ui: &egui::Ui, panel: Panel) -> f32 {
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let widest = reference_data::PARAMETERS
        .iter()
        .filter(|p| p.panel == panel)
        .map(|p| {
            ui.painter()
                .layout_no_wrap(p.name.to_string(), font_id.clone(), Color32::WHITE)
                .rect
                .width()
        })
        .fold(0.0_f32, f32::max);
    (widest + 6.0).clamp(LABEL_MIN_WIDTH, LABEL_MAX_WIDTH)
}

/// Total width this panel's rows actually need: label + input box + widest
/// unit string (in the active unit system) + card padding + inter-item gaps.
fn panel_min_width(ui: &egui::Ui, panel: Panel, unit_system: UnitSystem) -> f32 {
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let label_width = label_column_width(ui, panel);
    let widest_unit = reference_data::PARAMETERS
        .iter()
        .filter(|p| p.panel == panel)
        .map(|p| {
            ui.painter()
                .layout_no_wrap(
                    p.unit_for(unit_system).to_string(),
                    font_id.clone(),
                    Color32::WHITE,
                )
                .rect
                .width()
        })
        .fold(0.0_f32, f32::max);
    label_width + INPUT_WIDTH + widest_unit + CARD_PADDING + ROW_ITEM_SPACING
}

fn render_panel_card(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp, panel: Panel) {
    let visuals = ui.visuals().clone();
    egui::Frame::new()
        .fill(visuals.window_fill)
        .stroke(visuals.window_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(panel.label()).strong().size(15.0));
            ui.add_space(4.0);

            let label_width = label_column_width(ui, panel);
            let unit_system = app.unit_system;
            let striped_bg = visuals.faint_bg_color;

            for (i, param) in reference_data::PARAMETERS
                .iter()
                .filter(|p| p.panel == panel)
                .enumerate()
            {
                let bg = if i % 2 == 0 {
                    striped_bg
                } else {
                    Color32::TRANSPARENT
                };
                egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_sized([label_width, ROW_HEIGHT], egui::Label::new(param.name));
                            let text = app.inputs.entry(param.id).or_default();
                            ui.add_sized(
                                [INPUT_WIDTH, ROW_HEIGHT],
                                egui::TextEdit::singleline(text),
                            );
                            ui.label(param.unit_for(unit_system));
                        });

                        if let Some(result) = app.results.get(param.id) {
                            render_result(ui, param, result, unit_system);
                        }
                    });
            }
        });
}

fn render_result(
    ui: &mut egui::Ui,
    param: &Parameter,
    result: &AnalysisResult,
    unit_system: UnitSystem,
) {
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
    .strong()
    .small();

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
