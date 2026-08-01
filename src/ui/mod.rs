pub mod about_window;
pub mod compare_window;
pub mod entry_form;
pub mod header;
pub mod history_window;
pub mod reference_window;

use crate::model::Status;

pub fn status_color(status: Status) -> egui::Color32 {
    match status {
        Status::CriticalLow | Status::CriticalHigh => egui::Color32::from_rgb(220, 50, 47),
        Status::Low | Status::High => egui::Color32::from_rgb(210, 140, 20),
        Status::Normal => egui::Color32::from_rgb(60, 160, 70),
    }
}
