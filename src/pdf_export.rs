use std::path::Path;

use genpdf::elements::{Break, Paragraph};
use genpdf::style::{Color, Style, StyledString};
use genpdf::{Document, fonts};

use crate::app::BloodAnalyzerApp;
use crate::model::{Panel, Status, format_display_value};
use crate::reference_data;

/// Generates the PDF report and writes it to `path`. Uses the system Arial
/// font (this app is Windows-only, so it's always present) rather than
/// bundling/downloading a font file.
pub fn write_report(app: &BloodAnalyzerApp, path: &Path) -> Result<(), String> {
    let fonts_dir = Path::new(r"C:\Windows\Fonts");
    let load = |file: &str| -> Result<fonts::FontData, String> {
        fonts::FontData::load(fonts_dir.join(file), None)
            .map_err(|e| format!("could not load {file}: {e}"))
    };
    let family = fonts::FontFamily {
        regular: load("arial.ttf")?,
        bold: load("arialbd.ttf")?,
        italic: load("ariali.ttf")?,
        bold_italic: load("arialbi.ttf")?,
    };

    let mut doc = Document::new(family);
    doc.set_title("Blood Analyzer Report");
    doc.set_font_size(11);
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(15);
    doc.set_page_decorator(decorator);

    doc.push(Paragraph::new(StyledString::new(
        "Blood Analyzer Report",
        Style::new().bold().with_font_size(18),
    )));
    doc.push(Paragraph::new(format!(
        "Generated: {}    Sex: {:?}    Units: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        app.sex,
        app.unit_system.label(),
    )));
    doc.push(Break::new(1));
    doc.push(Paragraph::new(StyledString::new(
        "Educational reference tool only \u{2014} not a substitute for professional medical \
         diagnosis or advice. Reference ranges are general adult values and may differ from \
         your lab's own reference ranges.",
        Style::new().italic(),
    )));
    doc.push(Break::new(1));

    if app.results.is_empty() {
        doc.push(Paragraph::new("No analyzed results to report."));
    } else {
        for panel in Panel::ALL {
            let mut params: Vec<_> = reference_data::PARAMETERS
                .iter()
                .filter(|p| p.panel == panel && app.results.contains_key(p.id))
                .collect();
            if params.is_empty() {
                continue;
            }
            params.sort_by_key(|p| p.name);

            doc.push(Paragraph::new(StyledString::new(
                panel.label(),
                Style::new().bold().with_font_size(14),
            )));
            doc.push(Break::new(0.5));

            for param in params {
                let result = &app.results[param.id];
                let value = param.si_to_display(result.value, app.unit_system);
                let low = param.si_to_display(result.range_low, app.unit_system);
                let high = param.si_to_display(result.range_high, app.unit_system);
                let unit = param.unit_for(app.unit_system);

                let color = status_color(result.status);
                doc.push(Paragraph::new(StyledString::new(
                    format!(
                        "{}: {} {}  [{}]  (range {}\u{2013}{} {})",
                        param.name,
                        format_display_value(value),
                        unit,
                        result.status.label(),
                        format_display_value(low),
                        format_display_value(high),
                        unit,
                    ),
                    Style::new().bold().with_color(color),
                )));

                if result.status.is_abnormal() {
                    doc.push(Paragraph::new(param.description));
                    let cause = match result.status {
                        Status::High | Status::CriticalHigh => Some(param.high_meaning),
                        Status::Low | Status::CriticalLow => Some(param.low_meaning),
                        Status::Normal => None,
                    };
                    if let Some(cause) = cause {
                        doc.push(Paragraph::new(cause));
                    }
                    let suggestion = match result.status {
                        Status::High | Status::CriticalHigh => param.suggestion_high,
                        Status::Low | Status::CriticalLow => param.suggestion_low,
                        Status::Normal => None,
                    };
                    if let Some(suggestion) = suggestion {
                        doc.push(Paragraph::new(StyledString::new(
                            format!("Lifestyle note (not medical advice): {suggestion}"),
                            Style::new().italic(),
                        )));
                    }
                }
                doc.push(Break::new(0.5));
            }
            doc.push(Break::new(1));
        }
    }

    doc.render_to_file(path).map_err(|e| e.to_string())
}

fn status_color(status: Status) -> Color {
    match status {
        Status::CriticalLow | Status::CriticalHigh => Color::Rgb(190, 30, 30),
        Status::Low | Status::High => Color::Rgb(180, 110, 10),
        Status::Normal => Color::Rgb(30, 120, 40),
    }
}
