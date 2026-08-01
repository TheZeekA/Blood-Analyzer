# Security Policy

Blood Analyzer is a local-only desktop tool: it does not send data anywhere,
does not run a server, and does not accept network input. Even so, if you find
a security issue (e.g. something that could corrupt or expose the local
`history.json` / `range_overrides.json` files, an unsafe dependency, or a
supply-chain concern in the build process), please report it privately rather
than opening a public issue.

## Reporting a Vulnerability

Use GitHub's private vulnerability reporting for this repository:

1. Go to the [Security tab](https://github.com/TheZeekA/Blood-Analyzer/security) of this repo.
2. Click **"Report a vulnerability"** under Advisories.
3. Describe the issue, how to reproduce it, and its potential impact.

You'll get a response acknowledging the report, and credit in the advisory
(unless you'd prefer to stay anonymous) once it's resolved.

## Supported Versions

Only the latest release/`main` branch is supported. There is no long-term
support for older versions — please update before reporting an issue to
confirm it still reproduces.

## Scope

In scope:
- The Rust application code in this repository
- The build process (`Cargo.toml`, `build.rs`, GitHub Actions workflows)
- Dependency vulnerabilities flagged for this project

Out of scope:
- The accuracy of medical reference ranges or clinical content (this is an
  educational tool, not a medical device — see the disclaimer in the README)
- Issues requiring physical access to a machine where the app is already
  installed
