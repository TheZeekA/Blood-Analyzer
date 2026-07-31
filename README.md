# Blood Analyzer

A Windows desktop tool for entering blood test results and checking them against
standard adult reference ranges. Flags values outside the normal range and shows
what each marker measures and what could cause a high or low result.

> **Educational reference tool only — not a substitute for professional medical
> diagnosis or advice.** Reference ranges are general adult values (SI units) and
> may differ from your lab's own reference ranges. Always confirm against the
> range printed on the lab report, and use clinical judgment.

The app is stateless: nothing you enter is saved to disk. Values live only in
memory for the current session.

## Screenshots

**Initial view** — select sex and panels, enter values:

![Initial view](screenshots/01_initial_view.png)

**Results view** — abnormal results surfaced first, color-coded by severity:

![Results view](screenshots/02_results_view.png)

**Expanded results** — click a result to see what it measures and possible causes:

![Expanded results view](screenshots/03_expanded_view.png)

## Panels covered (v1)

- **CBC** — Complete Blood Count (WBC, RBC, hemoglobin, hematocrit, indices,
  platelets, differential)
- **CMP** — Comprehensive Metabolic Panel (glucose, electrolytes, kidney panel,
  liver panel)
- **Lipid Panel** — total cholesterol, LDL, HDL, triglycerides

Reference ranges are given in SI units and are sex-specific where clinically
relevant (e.g. hemoglobin, hematocrit, RBC, creatinine, HDL).

## Building and running

Requires the Rust toolchain (install from [rustup.rs](https://rustup.rs) if not
already installed).

```bash
cargo build --release
```

The native executable is produced at `target\release\blood_analyzer.exe`.

To run directly during development:

```bash
cargo run
```

To run the unit test suite (range-boundary and classification logic):

```bash
cargo test
```

## Usage

1. Select the patient's sex and which panels to fill in.
2. Enter any known values; leave the rest blank.
3. Click **Analyze**.
4. Abnormal results are listed first, color-coded (yellow/orange = outside
   range, red = critical). Click a result to expand it and see what the
   marker measures and possible causes of a high or low value.
