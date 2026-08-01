# Contributing to Blood Analyzer

Thanks for your interest in contributing. This is a small Windows desktop
tool built in Rust with [egui](https://github.com/emilk/egui); contributions
of any size are welcome — new markers, bug fixes, UI polish, or doc fixes.

## Before you start

For anything beyond a small fix (a new panel, a new feature, a behavior
change), please open an issue first to discuss the approach. This avoids
wasted work on a PR that doesn't fit the project's direction.

## Getting set up

1. Install the Rust toolchain from [rustup.rs](https://rustup.rs).
2. Fork the repo and clone your fork.
3. Build and run:

   ```bash
   cargo build
   cargo run
   ```

4. Run the test suite before making changes, to confirm your environment is
   working:

   ```bash
   cargo test
   ```

## Making changes

- Keep pull requests focused — one logical change per PR is much easier to
  review than a bundle of unrelated fixes.
- Run `cargo fmt` and `cargo clippy` before committing; CI will check both.
- Add or update tests under `src/` for any behavior change (range boundaries,
  override resolution, history/settings persistence, etc.).
- If you add a new marker or reference range, cite the source (e.g. a
  MedlinePlus or professional guideline URL) the same way existing entries do
  in `reference_data.rs`.
- This is an educational reference tool, not a medical device — avoid
  language that reads as diagnostic or prescriptive; keep lifestyle notes
  general and non-prescriptive, matching the existing tone.

## Submitting a pull request

1. Push your branch to your fork and open a PR against `main`.
2. Fill in the PR template — what changed and why, and how you tested it.
3. Make sure CI passes (build, test, clippy, fmt).
4. A maintainer will review and may ask for changes before merging.

## Reporting bugs / requesting features

Use the issue templates under **Issues → New Issue**. Include your OS/build,
steps to reproduce, and expected vs. actual behavior for bugs.

## Security issues

Please don't open a public issue for a security concern — see
[SECURITY.md](SECURITY.md) for how to report privately.
