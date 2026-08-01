use std::collections::{HashMap, HashSet};

use chrono::Local;

use crate::analysis;
use crate::history::{History, SavedSession};
use crate::model::{AnalysisResult, Panel, Sex, UnitSystem};
use crate::pdf_export;
use crate::reference_data;
use crate::settings::{self, RangeOverrides};
use crate::ui;

pub struct BloodAnalyzerApp {
    pub sex: Sex,
    pub enabled_panels: HashSet<Panel>,
    pub unit_system: UnitSystem,
    pub inputs: HashMap<&'static str, String>,
    pub results: HashMap<&'static str, AnalysisResult>,
    pub analyzed: bool,
    pub range_overrides: RangeOverrides,
    pub history: History,
    pub show_reference_window: bool,
    pub show_history_window: bool,
    pub show_compare_window: bool,
    pub compare_a: Option<usize>,
    pub compare_b: Option<usize>,
    pub status_message: Option<String>,
    /// Transient text buffers for the Reference Data window's editable range
    /// fields, keyed e.g. `"wbc_low"` or `"hemoglobin_male_high"`.
    pub reference_edit_buffers: HashMap<String, String>,
}

impl BloodAnalyzerApp {
    pub fn new() -> Self {
        Self {
            sex: Sex::Male,
            enabled_panels: HashSet::from([Panel::Cbc, Panel::Cmp, Panel::Lipid]),
            unit_system: UnitSystem::Si,
            inputs: HashMap::new(),
            results: HashMap::new(),
            analyzed: false,
            range_overrides: RangeOverrides::load(),
            history: History::load(),
            show_reference_window: false,
            show_history_window: false,
            show_compare_window: false,
            compare_a: None,
            compare_b: None,
            status_message: None,
            reference_edit_buffers: HashMap::new(),
        }
    }

    pub fn panel_selected(&self, panel: Panel) -> bool {
        self.enabled_panels.contains(&panel)
    }

    pub fn set_panel_selected(&mut self, panel: Panel, enabled: bool) {
        if enabled {
            self.enabled_panels.insert(panel);
        } else {
            self.enabled_panels.remove(&panel);
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
            let Ok(display_value) = trimmed.parse::<f64>() else {
                continue;
            };
            let si_value = param.display_to_si(display_value, self.unit_system);
            let range = settings::effective_range(param, self.sex, &self.range_overrides);
            let result = analysis::analyze(si_value, range, param.critical);
            self.results.insert(param.id, result);
        }
        self.analyzed = true;
    }

    /// Re-interpret every non-empty entered value in the new unit system so a
    /// value typed before a toggle isn't silently misread after it.
    pub fn set_unit_system(&mut self, new_system: UnitSystem) {
        if new_system == self.unit_system {
            return;
        }
        for param in reference_data::PARAMETERS {
            let Some(text) = self.inputs.get(param.id) else { continue };
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(old_value) = trimmed.parse::<f64>() else { continue };
            let si_value = param.display_to_si(old_value, self.unit_system);
            let new_value = param.si_to_display(si_value, new_system);
            self.inputs.insert(param.id, format_value(new_value));
        }
        self.unit_system = new_system;
        if self.analyzed {
            self.run_analysis();
        }
    }

    pub fn save_current_session(&mut self) {
        let mut values = HashMap::new();
        for param in reference_data::PARAMETERS {
            let Some(text) = self.inputs.get(param.id) else { continue };
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(display_value) = trimmed.parse::<f64>() {
                let si_value = param.display_to_si(display_value, self.unit_system);
                values.insert(param.id.to_string(), si_value);
            }
        }
        if values.is_empty() {
            self.status_message = Some("Nothing to save — enter at least one value first.".to_string());
            return;
        }
        let session = SavedSession { timestamp: Local::now(), sex: self.sex, values };
        self.history.add(session);
        self.status_message = Some("Saved to history.".to_string());
    }

    pub fn load_session(&mut self, index: usize) {
        let Some(session) = self.history.0.get(index) else { return };
        self.sex = session.sex;
        self.inputs.clear();
        for (id, si_value) in &session.values {
            if let Some(param) = reference_data::find(id) {
                let display_value = param.si_to_display(*si_value, self.unit_system);
                self.inputs.insert(param.id, format_value(display_value));
            }
        }
        self.show_history_window = false;
        self.run_analysis();
    }

    pub fn delete_session(&mut self, index: usize) {
        self.history.remove(index);
    }

    pub fn export_pdf(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("blood_analyzer_report.pdf")
            .add_filter("PDF", &["pdf"])
            .save_file()
        else {
            return;
        };
        match pdf_export::write_report(self, &path) {
            Ok(()) => self.status_message = Some(format!("Report saved to {}", path.display())),
            Err(err) => self.status_message = Some(format!("Failed to save report: {err}")),
        }
    }
}

fn format_value(v: f64) -> String {
    let s = format!("{v:.4}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}

impl eframe::App for BloodAnalyzerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui::header::show(ui, self);
        });

        if self.show_reference_window {
            ui::reference_window::show(ui.ctx(), self);
        }
        if self.show_history_window {
            ui::history_window::show(ui.ctx(), self);
        }
        if self.show_compare_window {
            ui::compare_window::show(ui.ctx(), self);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui::entry_form::show(ui, self);
            });
        });
    }
}
