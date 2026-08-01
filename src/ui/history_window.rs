use crate::analysis;
use crate::app::BloodAnalyzerApp;
use crate::history::SavedSession;
use crate::reference_data;
use crate::settings;

pub fn show(ctx: &egui::Context, app: &mut BloodAnalyzerApp) {
    let mut open = app.show_history_window;
    let mut load_index: Option<usize> = None;
    let mut delete_index: Option<usize> = None;

    egui::Window::new("History")
        .open(&mut open)
        .default_size([460.0, 420.0])
        .vscroll(true)
        .show(ctx, |ui| {
            if app.history.0.is_empty() {
                ui.label("No saved results yet. Use \"Save Results\" on the main screen.");
                return;
            }

            for idx in app.history.sorted_indices() {
                let session = &app.history.0[idx];
                let abnormal = session_abnormal_count(app, session);
                ui.horizontal(|ui| {
                    ui.strong(session.label());
                    ui.label(format!(
                        "{} value(s), {} abnormal",
                        session.values.len(),
                        abnormal
                    ));
                    if ui.button("Load").clicked() {
                        load_index = Some(idx);
                    }
                    if ui.button("Delete").clicked() {
                        delete_index = Some(idx);
                    }
                });
                ui.separator();
            }
        });

    app.show_history_window = open;

    // Apply after syncing `open` so load_session's own window-close request
    // (it wants the window to close on a successful load) isn't clobbered by
    // the stale `open` captured before this frame's Load click.
    if let Some(idx) = load_index {
        app.load_session(idx);
    }
    if let Some(idx) = delete_index {
        app.delete_session(idx);
    }
}

fn session_abnormal_count(app: &BloodAnalyzerApp, session: &SavedSession) -> usize {
    session
        .values
        .iter()
        .filter_map(|(id, &value)| reference_data::find(id).map(|param| (param, value)))
        .filter(|(param, value)| {
            let range = settings::effective_range(param, session.sex, &app.range_overrides);
            analysis::analyze(*value, range, param.critical)
                .status
                .is_abnormal()
        })
        .count()
}
