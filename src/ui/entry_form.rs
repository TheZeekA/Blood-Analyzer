use crate::app::BloodAnalyzerApp;
use crate::model::Panel;
use crate::reference_data;

pub fn show(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp) {
    for panel in Panel::ALL {
        if !app.panel_selected(panel) {
            continue;
        }

        ui.add_space(4.0);
        ui.heading(panel.label());
        ui.add_space(4.0);

        egui::Grid::new(format!("entry_grid_{panel:?}"))
            .num_columns(3)
            .spacing([12.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                for param in reference_data::PARAMETERS.iter().filter(|p| p.panel == panel) {
                    ui.label(param.name);
                    let text = app.inputs.entry(param.id).or_default();
                    ui.add(egui::TextEdit::singleline(text).desired_width(100.0));
                    ui.label(param.unit);
                    ui.end_row();
                }
            });
    }
}
