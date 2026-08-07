# Android port design

Date: 2026-08-07

## Goal

Ship a polished Android release (Play Store or wide sharing) of Blood
Analyzer with a native Kotlin/Jetpack Compose UI, backed by the existing
Rust analysis/reference-data/history logic reused as a shared core library.
The existing Windows desktop app (`eframe`/`egui`) continues to work
unchanged.

## Non-goals

- No web/Tauri UI. No iOS. No attempt to make `eframe`/`egui` run on
  Android — the desktop UI code is not reused for the Android app; only
  the non-GUI logic is shared.
- No requirement to keep desktop and Android UI feature-identical from day
  one; feature parity is the eventual target but screens land incrementally
  (see Phased delivery).

## Why this shape (context for future readers)

Two things ruled out simpler options:

- `eframe`/`egui` Android support exists (via an `android-native-activity`
  feature and `android-activity`) but the packaging toolchain around it is
  young and rough — `cargo-apk`, the previous go-to packager, is deprecated
  in favor of `xbuild`, and community reports describe the setup as
  difficult outside a dedicated Android dev environment.
- `rfd` (used for the desktop Save dialog) has **no Android support at
  all** — only Windows/macOS/Linux. Any Android build needs a different
  file-save/share mechanism regardless of GUI framework choice.

Given the user wants a polished, Play-Store-quality release and is willing
to invest in a native Kotlin/Compose UI, the chosen shape is: extract the
platform-independent logic into a shared Rust core, bridge it to Kotlin
with UniFFI (auto-generated bindings, far less error-prone than hand
writing JNI), and build a new native Compose UI on top.

## Architecture: Cargo workspace, 3 members

Convert the repo root into a Cargo workspace with:

- **`core/`** (new lib crate, `blood_analyzer_core`) — pure Rust, no GUI
  dependency. Houses `model.rs`, `analysis.rs`, `reference_data.rs`
  unchanged (already framework-free), plus refactored `history.rs`,
  `settings.rs`, and PDF report generation (see below).
- **`desktop/`** — today's existing app: `app.rs`, `ui/*`, `main.rs`,
  `build.rs` (winresource), `rfd` usage. Moves under this member, now
  depends on `core` for logic it previously owned directly. Behavior is
  unchanged; `cargo build --release` from this member still produces the
  Windows `.exe` with its embedded icon.
- **`android/`** — new. A cdylib crate wrapping `core` with
  `#[uniffi::export]`, built per-ABI via `cargo ndk`, plus a Gradle
  Android Studio project (`android/app`) containing the Kotlin/Compose UI
  that consumes the generated Kotlin bindings and loads the `.so` via
  `jniLibs`.

## Changes required in `core` to become platform-agnostic

- **Storage path injection.** `History::file_path()` and
  `RangeOverrides::file_path()` currently read `%APPDATA%` directly via
  `std::env::var_os`. Change both to accept an injected base directory
  (e.g. `History::load(base_dir: &Path)` /
  `RangeOverrides::load(base_dir: &Path)`). Desktop passes
  `%APPDATA%\BloodAnalyzer` as today. Android passes its
  `context.filesDir` path, obtained on the Kotlin side and passed into
  Rust through the UniFFI init call.
- **PDF generation decoupled from the desktop app type.**
  `pdf_export::write_report` currently borrows `&BloodAnalyzerApp` (an
  `eframe`/`egui` struct that can't live in `core`) and hardcodes
  `C:\Windows\Fonts\arial.ttf`. Refactor to take a plain `ReportData`
  struct (results map, sex, unit system — no egui type) and accept font
  data as a parameter instead of a hardcoded path. Desktop continues
  loading system Arial from `C:\Windows\Fonts`; Android bundles a
  permissively-licensed font (e.g. Noto Sans) in the `android` crate and
  passes its bytes in. The function returns PDF bytes rather than writing
  a file directly, so both desktop (write via `rfd`-selected path) and
  Android (hand bytes to Kotlin for SAF/share) can do their own
  platform-appropriate output step.

Existing unit tests (range-boundary classification, override resolution,
history/settings round-trip) move with `core` and keep passing unchanged
except for call-site updates for the injected base-dir parameter.

## UniFFI bridge surface

Exposed from `core` via `#[uniffi::export]`:

- `init(storage_dir: String)` / equivalent config call
- `analyze(value, range, critical) -> AnalysisResult`
- list parameters / panels (mirrors `reference_data::PARAMETERS`,
  `Panel::ALL`)
- get / set / reset range overrides
- save / load / list / delete history entries
- compare two saved sessions
- `generate_pdf_bytes(report_data) -> Vec<u8>`

Errors surface as a UniFFI error enum rather than `Result<_, String>`
where that improves the generated Kotlin API. `uniffi-bindgen` generates
the Kotlin bindings from these annotations — no hand-written JNI.

## Kotlin/Compose app

Screens mirror the existing desktop windows:

- **Entry** — panels as cards, touch-sized numeric fields (48dp+ targets
  per Material guidance), on-screen keyboard for numeric entry
- **Results** — inline under each field, same color-coded severity as
  desktop; tap-to-expand (the desktop app already uses click-to-expand,
  not hover, so this interaction carries over directly)
- **Reference Data** — view/edit ranges, reset to default
- **History** — list, reload, delete
- **Compare** — two sessions side by side
- **About** — credit + repo link
- **PDF export** — generates bytes via `core`, then Kotlin writes them out
  through Android's `ACTION_CREATE_DOCUMENT` intent or a share-sheet
  intent, replacing the desktop's `rfd` Save dialog (unsupported on
  Android)

Navigation via Compose Navigation. No hover-dependent affordances anywhere
in the new UI.

## Storage & PDF export on Android

- Local data (`history.json`, `range_overrides.json`) lives under the
  app's internal storage (`context.filesDir`), obtained and passed to Rust
  at startup — no external storage permission needed.
- PDF export: Rust returns bytes; Kotlin either writes to a
  user-chosen location via SAF (`ACTION_CREATE_DOCUMENT`) or opens the
  system share sheet, whichever the Phase 3 screen work settles on.

## Build tooling

- Install Android Studio, Android SDK, NDK.
- Add Rust Android targets: `aarch64-linux-android`,
  `armv7-linux-androideabi`, `x86_64-linux-android` (emulator).
- Build the cdylib per-ABI with `cargo ndk`.
- Generate Kotlin bindings with `uniffi-bindgen`.
- Gradle module loads the resulting `.so` files via `jniLibs`.
- CI: add a job building/testing the `core` crate on the existing Linux
  runner (no GUI dependency, so this is cheap). A full Android APK build
  in CI is a later nice-to-have, not required for v1.

## Phased delivery plan

Sequenced to front-load the highest-uncertainty step (the Android
toolchain, new to this project) before investing in UI work:

0. **Toolchain spike.** Trivial one-function UniFFI round-trip
   (Kotlin calls into Rust, gets a value back) running on an
   emulator/device. Confirms the whole pipeline — NDK, `cargo ndk`,
   `uniffi-bindgen`, Gradle `jniLibs` wiring — actually works before
   anything else depends on it.
1. **Extract `core`.** Move `model.rs`/`analysis.rs`/`reference_data.rs`
   unchanged; refactor `history.rs`/`settings.rs` for injected storage
   paths and `pdf_export.rs` for the `ReportData`/font-bytes API. Confirm
   `desktop/` still builds and `cargo test` still passes against the new
   `core`.
2. **Wire one real screen.** Storage init + `analyze` call from a minimal
   Compose screen (e.g. just CBC panel), proving the real data path works
   end to end, not just the toolchain spike's dummy function.
3. **Build out remaining screens** to feature parity: full panel set,
   Reference Data, History, Compare, PDF export/share, About.
4. **Polish for release.** Touch ergonomics pass across all screens, app
   icon (adaptive icon per Android conventions), release signing config,
   Play Store listing assets (description, screenshots — existing desktop
   screenshots can inform layout but need Android-native equivalents).

## Testing

- `core`'s existing unit test suite (range boundaries, override
  resolution, history/settings persistence round-trip) moves over and
  keeps running via `cargo test -p blood_analyzer_core`.
- Compose UI tests are optional and not required for v1; can be added
  later if desired.

## Open risks

- Android Rust toolchain maturity is the main unknown — Phase 0 exists
  specifically to surface any blockers early.
- UniFFI's supported type surface may require minor reshaping of some
  `core` types (e.g. enums with data) when the actual bindgen step runs;
  expect small signature adjustments during Phase 1/2, not a redesign.
