use egui::{Color32, RichText};

use crate::app::BloodAnalyzerApp;
use crate::model::Status;
use crate::reference_data;

fn status_color(status: Status) -> Color32 {
    match status {
        Status::CriticalLow | Status::CriticalHigh => Color32::from_rgb(220, 50, 47),
        Status::Low | Status::High => Color32::from_rgb(210, 140, 20),
        Status::Normal => Color32::from_rgb(60, 160, 70),
    }
}

pub fn show(ui: &mut egui::Ui, app: &mut BloodAnalyzerApp) {
    ui.heading("Results");

    if app.results.is_empty() {
        ui.label("No values were entered. Fill in at least one field above and click Analyze.");
        return;
    }

    let abnormal_count = app.results.iter().filter(|r| r.status.is_abnormal()).count();
    if abnormal_count > 0 {
        ui.label(
            RichText::new(format!(
                "{abnormal_count} result(s) outside the standard reference range."
            ))
            .color(Color32::from_rgb(210, 140, 20))
            .strong(),
        );
    } else {
        ui.label(RichText::new("All entered results are within the standard reference range.").color(Color32::from_rgb(60, 160, 70)));
    }
    ui.add_space(6.0);

    for result in &app.results {
        let Some(param) = reference_data::find(result.parameter_id) else {
            continue;
        };

        let color = status_color(result.status);
        let header = RichText::new(format!(
            "{}: {}  {}  [{}]  (range {}\u{2013}{} {})",
            param.name,
            result.value,
            param.unit,
            result.status.label(),
            result.range_low,
            result.range_high,
            param.unit,
        ))
        .color(color)
        .strong();

        egui::CollapsingHeader::new(header)
            .id_salt(param.id)
            .show(ui, |ui| {
                ui.label(param.description);
                match result.status {
                    Status::High | Status::CriticalHigh => {
                        ui.add_space(4.0);
                        ui.label(RichText::new("Possible causes of high result:").strong());
                        ui.label(param.high_meaning);
                    }
                    Status::Low | Status::CriticalLow => {
                        ui.add_space(4.0);
                        ui.label(RichText::new("Possible causes of low result:").strong());
                        ui.label(param.low_meaning);
                    }
                    Status::Normal => {}
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
            });
    }
}
