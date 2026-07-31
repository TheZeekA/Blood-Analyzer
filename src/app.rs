use std::collections::HashMap;

use crate::analysis;
use crate::model::{AnalysisResult, Panel, Sex};
use crate::reference_data;
use crate::ui;

pub struct BloodAnalyzerApp {
    pub sex: Sex,
    pub show_cbc: bool,
    pub show_cmp: bool,
    pub show_lipid: bool,
    pub inputs: HashMap<&'static str, String>,
    pub results: Vec<AnalysisResult>,
    pub analyzed: bool,
}

impl Default for BloodAnalyzerApp {
    fn default() -> Self {
        Self {
            sex: Sex::Male,
            show_cbc: true,
            show_cmp: true,
            show_lipid: true,
            inputs: HashMap::new(),
            results: Vec::new(),
            analyzed: false,
        }
    }
}

impl BloodAnalyzerApp {
    pub fn panel_selected(&self, panel: Panel) -> bool {
        match panel {
            Panel::Cbc => self.show_cbc,
            Panel::Cmp => self.show_cmp,
            Panel::Lipid => self.show_lipid,
        }
    }

    pub fn run_analysis(&mut self) {
        self.results.clear();
        for param in reference_data::PARAMETERS {
            if !self.panel_selected(param.panel) {
                continue;
            }
            let Some(text) = self.inputs.get(param.id) else {
                continue;
            };
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(value) = trimmed.parse::<f64>() {
                self.results.push(analysis::analyze(param, value, self.sex));
            }
        }
        // Abnormal results surfaced first.
        self.results.sort_by_key(|r| r.status == crate::model::Status::Normal);
        self.analyzed = true;
    }
}

impl eframe::App for BloodAnalyzerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui::header::show(ui, self);
        });
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui::entry_form::show(ui, self);

                ui.add_space(8.0);
                if ui.button("Analyze").clicked() {
                    self.run_analysis();
                }
                ui.add_space(8.0);

                if self.analyzed {
                    ui.separator();
                    ui::results_view::show(ui, self);
                }
            });
        });
    }
}
