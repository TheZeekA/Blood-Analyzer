# Android Core Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the repo into a Cargo workspace with a platform-independent `blood_analyzer_core` crate (model, analysis, reference data, history, range overrides, PDF generation) and a `desktop` crate (today's `eframe`/`egui` app), with zero behavior change to the Windows desktop app.

**Architecture:** Extract the six logic modules (`model`, `analysis`, `reference_data`, `history`, `settings`, `pdf_export`) into a new `core/` lib crate with no GUI dependency. The desktop crate keeps its module names as thin `use blood_analyzer_core::X;` re-exports at its crate root, so every existing `crate::model::...`-style reference throughout `app.rs`/`ui/*.rs` keeps compiling unchanged — only the handful of call sites that actually need new parameters (injected storage directory, injected font data) are touched.

**Tech Stack:** Rust (edition 2024), Cargo workspaces, existing dependencies (`chrono`, `serde`, `serde_json`, `genpdf`, `eframe`/`egui`, `rfd`, `winresource`) — no new dependencies.

## Global Constraints

- Every existing unit test (in `analysis.rs`, `settings.rs`, `history.rs`) must keep passing, unmodified in behavior, throughout.
- The Windows desktop app's behavior, file layout under `%APPDATA%\BloodAnalyzer\`, and `cargo build --release` output path (`target\release\blood_analyzer.exe`, icon embedded) must not change.
- No Android/UniFFI/Kotlin work happens in this plan — this plan is scoped to Phase 1 ("Extract `core`") of `docs/superpowers/specs/2026-08-07-android-port-design.md`. Android toolchain work is a separate follow-up plan once this lands.
- No new dependencies are introduced; only which crate depends on which existing dependency changes.

---

### Task 1: Workspace skeleton — move `model`, `analysis`, `reference_data` into `core`

**Files:**
- Create: `Cargo.toml` (rewritten as workspace manifest)
- Create: `core/Cargo.toml`
- Create: `core/src/lib.rs`
- Move: `src/model.rs` → `core/src/model.rs` (no content changes)
- Move: `src/analysis.rs` → `core/src/analysis.rs` (no content changes)
- Move: `src/reference_data.rs` → `core/src/reference_data.rs` (no content changes)
- Move: `src/` → `desktop/src/`, `images/` → `desktop/images/`, `build.rs` → `desktop/build.rs`
- Create: `desktop/Cargo.toml`
- Modify: `desktop/src/main.rs`

**Interfaces:**
- Produces: `blood_analyzer_core::model::*`, `blood_analyzer_core::analysis::analyze(...)`, `blood_analyzer_core::reference_data::{PARAMETERS, find}` — identical signatures to today's `crate::model`/`crate::analysis`/`crate::reference_data`, just re-exported at the desktop crate root as `crate::model`/`crate::analysis`/`crate::reference_data` so no other file needs edits.

- [ ] **Step 1: Move the whole existing crate into `desktop/`**

```bash
mkdir desktop
git mv src desktop/src
git mv images desktop/images
git mv build.rs desktop/build.rs
```

- [ ] **Step 2: Move the three framework-free modules into a new `core/` crate**

```bash
mkdir -p core/src
git mv desktop/src/model.rs core/src/model.rs
git mv desktop/src/analysis.rs core/src/analysis.rs
git mv desktop/src/reference_data.rs core/src/reference_data.rs
```

These three files reference each other only via `crate::model::...`, which continues to resolve correctly once they live together in the new `core` crate — no content edits needed.

- [ ] **Step 3: Write `core/src/lib.rs`**

```rust
pub mod analysis;
pub mod model;
pub mod reference_data;
```

- [ ] **Step 4: Write `core/Cargo.toml`**

```toml
[package]
name = "blood_analyzer_core"
version = "0.3.0"
edition = "2024"
license = "MIT"

[dependencies]
serde = { version = "1.0.229", features = ["derive"] }
```

- [ ] **Step 5: Write `desktop/Cargo.toml`**

```toml
[package]
name = "blood_analyzer"
version = "0.3.0"
edition = "2024"
build = "build.rs"
license = "MIT"

[dependencies]
blood_analyzer_core = { path = "../core" }
chrono = { version = "0.4.45", features = ["serde"] }
eframe = "0.35.0"
egui = "0.35.0"
egui_extras = { version = "0.35.0", default-features = false, features = ["image"] }
genpdf = "0.2.0"
rfd = "0.17.2"
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"

[build-dependencies]
winresource = "0.1.31"
```

(`serde`/`serde_json`/`genpdf` stay here for now because `history.rs`, `settings.rs`, and `pdf_export.rs` still live in `desktop/` at this point — they move out in Tasks 2–3, and unused deps are trimmed in Task 4.)

- [ ] **Step 6: Rewrite the root `Cargo.toml` as a workspace manifest**

```toml
[workspace]
resolver = "2"
members = ["core", "desktop"]
```

- [ ] **Step 7: Edit `desktop/src/main.rs` module declarations**

Replace:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod app;
mod history;
mod model;
mod pdf_export;
mod reference_data;
mod settings;
mod ui;

use app::BloodAnalyzerApp;
```

with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod history;
mod pdf_export;
mod settings;
mod ui;

use blood_analyzer_core::analysis;
use blood_analyzer_core::model;
use blood_analyzer_core::reference_data;

use app::BloodAnalyzerApp;
```

The rest of `main.rs` (icon loading, `eframe::run_native`) is unchanged — its `include_bytes!("../images/bloodtestlogo.png")` still resolves correctly since `images/` moved alongside it into `desktop/`.

- [ ] **Step 8: Build and test the whole workspace**

Run: `cargo build --workspace`
Expected: builds `blood_analyzer_core` and `blood_analyzer` successfully (this also regenerates `Cargo.lock` for the new workspace layout).

Run: `cargo test --workspace`
Expected: PASS — `analysis.rs`'s 4 tests now run as part of `blood_analyzer_core`; `settings.rs`'s 4 tests and `history.rs`'s 2 tests still run as part of `blood_analyzer` (unmoved in this task).

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
Extract model/analysis/reference_data into a blood_analyzer_core crate

First step of the Android port: convert the repo into a Cargo workspace
so platform-independent logic can eventually be shared with an Android
build. The desktop crate re-exports these modules at its crate root so
no other call site needs to change.
EOF
)"
```

---

### Task 2: Move `history` and `settings` into `core` with injected storage directory

**Files:**
- Move: `desktop/src/history.rs` → `core/src/history.rs` (rewritten)
- Move: `desktop/src/settings.rs` → `core/src/settings.rs` (rewritten)
- Modify: `core/Cargo.toml`, `core/src/lib.rs`
- Modify: `desktop/src/main.rs`, `desktop/src/app.rs`, `desktop/src/ui/reference_window.rs`

**Interfaces:**
- Consumes: `blood_analyzer_core::model::Sex` (Task 1)
- Produces: `History::load(storage_dir: &Path) -> History`, `History::add(&mut self, session: SavedSession, storage_dir: &Path)`, `History::remove(&mut self, index: usize, storage_dir: &Path)`, `RangeOverrides::load(storage_dir: &Path) -> RangeOverrides`, `RangeOverrides::save(&self, storage_dir: &Path)` — all previously took no arguments and read `%APPDATA%` directly; callers now own computing and passing the storage directory.

- [ ] **Step 1: Move the files**

```bash
git mv desktop/src/history.rs core/src/history.rs
git mv desktop/src/settings.rs core/src/settings.rs
```

- [ ] **Step 2: Replace `core/src/history.rs` content**

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

use crate::model::Sex;

/// One saved, dated set of results. `values` are canonical SI floats keyed by
/// `Parameter::id`, independent of whatever unit system was active when saved.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedSession {
    pub timestamp: DateTime<Local>,
    pub sex: Sex,
    pub values: HashMap<String, f64>,
}

impl SavedSession {
    pub fn label(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M").to_string()
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct History(pub Vec<SavedSession>);

impl History {
    fn file_path(storage_dir: &Path) -> PathBuf {
        storage_dir.join("history.json")
    }

    pub fn load(storage_dir: &Path) -> Self {
        std::fs::read_to_string(Self::file_path(storage_dir))
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, storage_dir: &Path) {
        let path = Self::file_path(storage_dir);
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Newest-first for display.
    pub fn sorted_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.0.len()).collect();
        indices.sort_by(|&a, &b| self.0[b].timestamp.cmp(&self.0[a].timestamp));
        indices
    }

    pub fn add(&mut self, session: SavedSession, storage_dir: &Path) {
        self.0.push(session);
        self.save(storage_dir);
    }

    pub fn remove(&mut self, index: usize, storage_dir: &Path) {
        if index < self.0.len() {
            self.0.remove(index);
            self.save(storage_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let mut values = HashMap::new();
        values.insert("wbc".to_string(), 7.5);
        values.insert("glucose".to_string(), 5.1);
        let session = SavedSession {
            timestamp: Local::now(),
            sex: Sex::Female,
            values,
        };
        let history = History(vec![session]);

        let json = serde_json::to_string(&history).unwrap();
        let restored: History = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.0.len(), 1);
        assert_eq!(restored.0[0].values.get("wbc"), Some(&7.5));
        assert_eq!(restored.0[0].sex, Sex::Female);
    }

    #[test]
    fn sorted_indices_newest_first() {
        let mk = |secs_ago: i64| SavedSession {
            timestamp: Local::now() - chrono::Duration::seconds(secs_ago),
            sex: Sex::Male,
            values: HashMap::new(),
        };
        let history = History(vec![mk(100), mk(0), mk(50)]);
        assert_eq!(history.sorted_indices(), vec![1, 2, 0]);
    }
}
```

- [ ] **Step 3: Replace `core/src/settings.rs` content**

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::model::{Parameter, RangeSpec, Sex};

/// User-edited reference ranges, keyed by `Parameter::id`. Layered on top of
/// the built-in defaults in `reference_data::PARAMETERS` rather than mutating
/// them, so "reset to default" is just "remove the override".
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RangeOverrides(pub HashMap<String, RangeSpec>);

impl RangeOverrides {
    fn file_path(storage_dir: &Path) -> PathBuf {
        storage_dir.join("range_overrides.json")
    }

    pub fn load(storage_dir: &Path) -> Self {
        std::fs::read_to_string(Self::file_path(storage_dir))
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, storage_dir: &Path) {
        let path = Self::file_path(storage_dir);
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn set(&mut self, param_id: &str, range: RangeSpec) {
        self.0.insert(param_id.to_string(), range);
    }

    pub fn reset(&mut self, param_id: &str) {
        self.0.remove(param_id);
    }

    pub fn reset_all(&mut self) {
        self.0.clear();
    }

    pub fn get(&self, param_id: &str) -> Option<RangeSpec> {
        self.0.get(param_id).copied()
    }
}

/// Resolve the range actually in effect for a parameter: an override if one
/// exists, otherwise the built-in default (sex-resolved).
pub fn effective_range(param: &Parameter, sex: Sex, overrides: &RangeOverrides) -> (f64, f64) {
    match overrides.get(param.id) {
        Some(range) => range.resolve(sex),
        None => param.range.resolve(sex),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Panel;

    fn test_param() -> Parameter {
        Parameter {
            id: "test",
            name: "Test",
            unit: "unit",
            panel: Panel::Cmp,
            range: RangeSpec::Fixed {
                low: 1.0,
                high: 2.0,
            },
            critical: None,
            description: "",
            high_meaning: "",
            low_meaning: "",
            us_unit: "unit",
            si_to_us_factor: 1.0,
            source_url: "",
            suggestion_high: None,
            suggestion_low: None,
        }
    }

    #[test]
    fn falls_back_to_default_when_no_override() {
        let param = test_param();
        let overrides = RangeOverrides::default();
        assert_eq!(effective_range(&param, Sex::Male, &overrides), (1.0, 2.0));
    }

    #[test]
    fn override_takes_precedence() {
        let param = test_param();
        let mut overrides = RangeOverrides::default();
        overrides.set(
            "test",
            RangeSpec::Fixed {
                low: 5.0,
                high: 9.0,
            },
        );
        assert_eq!(effective_range(&param, Sex::Male, &overrides), (5.0, 9.0));
    }

    #[test]
    fn reset_removes_override() {
        let param = test_param();
        let mut overrides = RangeOverrides::default();
        overrides.set(
            "test",
            RangeSpec::Fixed {
                low: 5.0,
                high: 9.0,
            },
        );
        overrides.reset("test");
        assert_eq!(effective_range(&param, Sex::Male, &overrides), (1.0, 2.0));
    }

    #[test]
    fn serde_roundtrip() {
        let mut overrides = RangeOverrides::default();
        overrides.set(
            "wbc",
            RangeSpec::Fixed {
                low: 3.5,
                high: 10.5,
            },
        );
        overrides.set(
            "hemoglobin",
            RangeSpec::BySex {
                male: (130.0, 170.0),
                female: (115.0, 150.0),
            },
        );
        let json = serde_json::to_string(&overrides).unwrap();
        let restored: RangeOverrides = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get("wbc"), overrides.get("wbc"));
    }
}
```

- [ ] **Step 4: Add `chrono`/`serde_json` to `core/Cargo.toml` and declare the new modules**

In `core/Cargo.toml`, replace the `[dependencies]` section:

```toml
[dependencies]
chrono = { version = "0.4.45", features = ["serde"] }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
```

In `core/src/lib.rs`, replace the content:

```rust
pub mod analysis;
pub mod history;
pub mod model;
pub mod reference_data;
pub mod settings;
```

- [ ] **Step 5: Edit `desktop/src/main.rs`**

Replace:

```rust
mod app;
mod history;
mod pdf_export;
mod settings;
mod ui;

use blood_analyzer_core::analysis;
use blood_analyzer_core::model;
use blood_analyzer_core::reference_data;
```

with:

```rust
mod app;
mod pdf_export;
mod ui;

use blood_analyzer_core::analysis;
use blood_analyzer_core::history;
use blood_analyzer_core::model;
use blood_analyzer_core::reference_data;
use blood_analyzer_core::settings;
```

- [ ] **Step 6: Edit `desktop/src/app.rs` to compute and thread through a storage directory**

Add `use std::path::PathBuf;` to the top-level `use std::collections::{HashMap, HashSet};` line:

```rust
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
```

Add a `storage_dir` field to the struct — replace:

```rust
    /// Transient text buffers for the Reference Data window's editable range
    /// fields, keyed e.g. `"wbc_low"` or `"hemoglobin_male_high"`.
    pub reference_edit_buffers: HashMap<String, String>,
}
```

with:

```rust
    /// Transient text buffers for the Reference Data window's editable range
    /// fields, keyed e.g. `"wbc_low"` or `"hemoglobin_male_high"`.
    pub reference_edit_buffers: HashMap<String, String>,
    /// Directory `history.json`/`range_overrides.json` are read from and
    /// written to. Computed once at startup.
    pub storage_dir: PathBuf,
}
```

Replace `new()`:

```rust
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
            show_about_window: false,
            compare_a: None,
            compare_b: None,
            status_message: None,
            reference_edit_buffers: HashMap::new(),
        }
    }
```

with:

```rust
impl BloodAnalyzerApp {
    pub fn new() -> Self {
        let storage_dir = default_storage_dir();
        Self {
            sex: Sex::Male,
            enabled_panels: HashSet::from([Panel::Cbc, Panel::Cmp, Panel::Lipid]),
            unit_system: UnitSystem::Si,
            inputs: HashMap::new(),
            results: HashMap::new(),
            analyzed: false,
            range_overrides: RangeOverrides::load(&storage_dir),
            history: History::load(&storage_dir),
            show_reference_window: false,
            show_history_window: false,
            show_compare_window: false,
            show_about_window: false,
            compare_a: None,
            compare_b: None,
            status_message: None,
            reference_edit_buffers: HashMap::new(),
            storage_dir,
        }
    }
```

Update `save_current_session` and `delete_session` — replace:

```rust
        self.history.add(session);
        self.status_message = Some("Saved to history.".to_string());
```

with:

```rust
        self.history.add(session, &self.storage_dir);
        self.status_message = Some("Saved to history.".to_string());
```

and replace:

```rust
    pub fn delete_session(&mut self, index: usize) {
        self.history.remove(index);
    }
```

with:

```rust
    pub fn delete_session(&mut self, index: usize) {
        self.history.remove(index, &self.storage_dir);
    }
```

Add the storage-dir helper function near the bottom of the file, right before `fn format_value`:

```rust
/// `%APPDATA%\BloodAnalyzer` on Windows. Falls back to a `BloodAnalyzer`
/// directory relative to the current working directory in the (practically
/// unreachable, on Windows) case `%APPDATA%` isn't set, rather than losing
/// the ability to persist data.
fn default_storage_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_default();
    base.join("BloodAnalyzer")
}

fn format_value(v: f64) -> String {
```

(This removes the old `fn format_value(v: f64) -> String {` line that's already there — you're inserting the new function directly above the existing one, not duplicating it.)

- [ ] **Step 7: Edit `desktop/src/ui/reference_window.rs` — thread the storage dir through the 4 save-on-edit call sites**

Replace (line ~22):

```rust
                app.range_overrides.reset_all();
                app.range_overrides.save();
```

with:

```rust
                app.range_overrides.reset_all();
                app.range_overrides.save(&app.storage_dir);
```

Replace (line ~56):

```rust
                                app.range_overrides.reset(param.id);
                                app.range_overrides.save();
```

with:

```rust
                                app.range_overrides.reset(param.id);
                                app.range_overrides.save(&app.storage_dir);
```

Replace (line ~115):

```rust
        app.range_overrides.set(
            param.id,
            RangeSpec::Fixed {
                low: low_si,
                high: high_si,
            },
        );
        app.range_overrides.save();
```

with:

```rust
        app.range_overrides.set(
            param.id,
            RangeSpec::Fixed {
                low: low_si,
                high: high_si,
            },
        );
        app.range_overrides.save(&app.storage_dir);
```

Replace (line ~165):

```rust
                app.range_overrides.set(param.id, new_range);
                app.range_overrides.save();
```

with:

```rust
                app.range_overrides.set(param.id, new_range);
                app.range_overrides.save(&app.storage_dir);
```

- [ ] **Step 8: Build and test**

Run: `cargo build --workspace`
Expected: builds successfully.

Run: `cargo test --workspace`
Expected: PASS — `history.rs`'s 2 tests and `settings.rs`'s 4 tests now run as part of `blood_analyzer_core`.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
Move history and settings into blood_analyzer_core

Storage location becomes an explicit parameter instead of history.rs and
settings.rs reading %APPDATA% directly, so a future Android build can
supply its own app-internal storage path through the same API. The
desktop app computes %APPDATA%\BloodAnalyzer once at startup and threads
it through unchanged otherwise.
EOF
)"
```

---

### Task 3: Move `pdf_export` into `core`, decoupled from the desktop app type and Windows fonts

**Files:**
- Move: `desktop/src/pdf_export.rs` → `core/src/pdf_export.rs` (rewritten)
- Modify: `core/Cargo.toml`, `core/src/lib.rs`
- Modify: `desktop/src/main.rs`, `desktop/src/app.rs`

**Interfaces:**
- Consumes: `blood_analyzer_core::model::{AnalysisResult, Panel, Sex, Status, UnitSystem, format_display_value}` (Task 1), `blood_analyzer_core::reference_data::PARAMETERS` (Task 1)
- Produces: `pdf_export::ReportData { sex, unit_system, results }` (plain data, no GUI dependency) and `pdf_export::generate_pdf(report: &ReportData, family: genpdf::fonts::FontFamily<genpdf::fonts::FontData>) -> Result<Vec<u8>, String>` — replaces the old `write_report(app: &BloodAnalyzerApp, path: &Path) -> Result<(), String>`, which took the egui app type directly and wrote straight to a hardcoded-font-derived file. Callers now load their own platform fonts and write the returned bytes themselves.

- [ ] **Step 1: Move the file**

```bash
git mv desktop/src/pdf_export.rs core/src/pdf_export.rs
```

- [ ] **Step 2: Replace `core/src/pdf_export.rs` content**

```rust
use std::collections::HashMap;

use genpdf::elements::{Break, Paragraph};
use genpdf::style::{Color, Style, StyledString};
use genpdf::{Document, fonts};

use crate::model::{AnalysisResult, Panel, Sex, Status, UnitSystem, format_display_value};
use crate::reference_data;

/// Everything `generate_pdf` needs to know about the current session. Plain
/// data with no dependency on any particular UI framework, so both the
/// desktop app and (eventually) an Android build can populate it themselves.
pub struct ReportData {
    pub sex: Sex,
    pub unit_system: UnitSystem,
    pub results: HashMap<&'static str, AnalysisResult>,
}

/// Generates the PDF report and returns its bytes. Font data is supplied by
/// the caller since font availability/loading is platform-specific (the
/// desktop build loads system fonts from `C:\Windows\Fonts`; a mobile build
/// would bundle its own font file instead).
pub fn generate_pdf(
    report: &ReportData,
    family: fonts::FontFamily<fonts::FontData>,
) -> Result<Vec<u8>, String> {
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
        report.sex,
        report.unit_system.label(),
    )));
    doc.push(Break::new(1));
    doc.push(Paragraph::new(StyledString::new(
        "Educational reference tool only \u{2014} not a substitute for professional medical \
         diagnosis or advice. Reference ranges are general adult values and may differ from \
         your lab's own reference ranges.",
        Style::new().italic(),
    )));
    doc.push(Break::new(1));

    if report.results.is_empty() {
        doc.push(Paragraph::new("No analyzed results to report."));
    } else {
        for panel in Panel::ALL {
            let mut params: Vec<_> = reference_data::PARAMETERS
                .iter()
                .filter(|p| p.panel == panel && report.results.contains_key(p.id))
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
                let result = &report.results[param.id];
                let value = param.si_to_display(result.value, report.unit_system);
                let low = param.si_to_display(result.range_low, report.unit_system);
                let high = param.si_to_display(result.range_high, report.unit_system);
                let unit = param.unit_for(report.unit_system);

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

    let mut bytes = Vec::new();
    doc.render(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn status_color(status: Status) -> Color {
    match status {
        Status::CriticalLow | Status::CriticalHigh => Color::Rgb(190, 30, 30),
        Status::Low | Status::High => Color::Rgb(180, 110, 10),
        Status::Normal => Color::Rgb(30, 120, 40),
    }
}
```

- [ ] **Step 3: Add `genpdf` to `core/Cargo.toml` and declare the new module**

In `core/Cargo.toml`, replace the `[dependencies]` section:

```toml
[dependencies]
chrono = { version = "0.4.45", features = ["serde"] }
genpdf = "0.2.0"
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
```

In `core/src/lib.rs`, replace the content:

```rust
pub mod analysis;
pub mod history;
pub mod model;
pub mod pdf_export;
pub mod reference_data;
pub mod settings;
```

- [ ] **Step 4: Edit `desktop/src/main.rs`**

Replace:

```rust
mod app;
mod pdf_export;
mod ui;

use blood_analyzer_core::analysis;
use blood_analyzer_core::history;
use blood_analyzer_core::model;
use blood_analyzer_core::reference_data;
use blood_analyzer_core::settings;
```

with:

```rust
mod app;
mod ui;

use blood_analyzer_core::analysis;
use blood_analyzer_core::history;
use blood_analyzer_core::model;
use blood_analyzer_core::pdf_export;
use blood_analyzer_core::reference_data;
use blood_analyzer_core::settings;
```

- [ ] **Step 5: Rewrite `export_pdf` in `desktop/src/app.rs`**

Replace:

```rust
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
```

with:

```rust
    pub fn export_pdf(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("blood_analyzer_report.pdf")
            .add_filter("PDF", &["pdf"])
            .save_file()
        else {
            return;
        };
        match self.render_pdf_report() {
            Ok(bytes) => match std::fs::write(&path, bytes) {
                Ok(()) => {
                    self.status_message = Some(format!("Report saved to {}", path.display()))
                }
                Err(err) => self.status_message = Some(format!("Failed to save report: {err}")),
            },
            Err(err) => self.status_message = Some(format!("Failed to generate report: {err}")),
        }
    }

    /// Loads the system Arial font family and renders the current results
    /// into PDF bytes. Font loading is Windows-specific (this app is
    /// Windows-only), which is why it lives here rather than in
    /// `blood_analyzer_core`.
    fn render_pdf_report(&self) -> Result<Vec<u8>, String> {
        let fonts_dir = std::path::Path::new(r"C:\Windows\Fonts");
        let load = |file: &str| -> Result<genpdf::fonts::FontData, String> {
            genpdf::fonts::FontData::load(fonts_dir.join(file), None)
                .map_err(|e| format!("could not load {file}: {e}"))
        };
        let family = genpdf::fonts::FontFamily {
            regular: load("arial.ttf")?,
            bold: load("arialbd.ttf")?,
            italic: load("ariali.ttf")?,
            bold_italic: load("arialbi.ttf")?,
        };

        let report = pdf_export::ReportData {
            sex: self.sex,
            unit_system: self.unit_system,
            results: self.results.clone(),
        };
        pdf_export::generate_pdf(&report, family)
    }
```

- [ ] **Step 6: Build and test**

Run: `cargo build --workspace`
Expected: builds successfully.

Run: `cargo test --workspace`
Expected: PASS — same test set as Task 2 (pdf_export.rs has no tests).

- [ ] **Step 7: Manually verify PDF export still works**

Run: `cargo run -p blood_analyzer`, enter at least one value (e.g. WBC), click Analyze, click "Export PDF", save to a temp path, and confirm the PDF opens and contains the same content as before (title, disclaimer, per-panel results with color-coded status).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
Move PDF generation into blood_analyzer_core, decoupled from the egui app

pdf_export::generate_pdf now takes a plain ReportData struct and a
caller-supplied font family, returning PDF bytes instead of writing a
file directly. The desktop app owns loading Windows system fonts and
writing the bytes to the rfd-selected path — both steps a future Android
build would need to do differently anyway.
EOF
)"
```

---

### Task 4: Dependency cleanup and full verification

**Files:**
- Modify: `desktop/Cargo.toml`

**Interfaces:**
- Consumes: the fully-extracted `core` crate from Tasks 1–3.
- Produces: nothing new — this task only removes now-unused direct dependencies from `desktop` and re-verifies the whole workspace end to end.

- [ ] **Step 1: Remove now-unused direct dependencies from `desktop/Cargo.toml`**

`serde` and `serde_json` are no longer used directly by any file under `desktop/src/` (only by code that now lives in `core`). Replace the `[dependencies]` section:

```toml
[dependencies]
blood_analyzer_core = { path = "../core" }
chrono = { version = "0.4.45", features = ["serde"] }
eframe = "0.35.0"
egui = "0.35.0"
egui_extras = { version = "0.35.0", default-features = false, features = ["image"] }
genpdf = "0.2.0"
rfd = "0.17.2"
```

(`chrono` stays — `app.rs` still calls `Local::now()` directly when building a `SavedSession`. `genpdf` stays — `app.rs` still loads `genpdf::fonts::FontData`/`FontFamily` for Windows font loading.)

- [ ] **Step 2: Run the exact checks CI runs**

Run: `cargo fmt --all -- --check`
Expected: no output (already formatted). If it reports diffs, run `cargo fmt --all` and re-check.

Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings/errors across both `core` and `desktop`.

Run: `cargo build --workspace --verbose`
Expected: builds successfully.

Run: `cargo test --workspace --verbose`
Expected: PASS — full suite (analysis + settings + history tests, now all in `blood_analyzer_core`).

- [ ] **Step 3: Verify the release build still produces the expected artifact**

Run: `cargo build --release`
Expected: succeeds and produces `target\release\blood_analyzer.exe` (same path as before the refactor, since the workspace shares one `target/` directory at the repo root regardless of which member you build).

Run: `ls target/release/blood_analyzer.exe` (or `Get-Item target\release\blood_analyzer.exe` in PowerShell)
Expected: file exists.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "$(cat <<'EOF'
Drop now-unused serde/serde_json deps from the desktop crate

Cleanup after the core-extraction refactor: these were only needed by
code that now lives in blood_analyzer_core.
EOF
)"
```

---

## After this plan

This completes Phase 1 of the Android port design. The repo is now a workspace with a reusable `blood_analyzer_core` crate and an unchanged-behavior `desktop` app. The next plan (written separately, once Android Studio/NDK/Rust Android targets are set up on this machine) covers the design's Phase 0 (toolchain spike) and Phase 2 (wiring UniFFI + a first real Compose screen against this `core` crate).
