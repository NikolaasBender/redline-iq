# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI pipeline for linting and formatting the Rust codebase.
- Architecture diagram in `docs/architecture.md`.

## [0.1.0] - 2026-03-30
### Added
- Initial creation of the Garmin Connect IQ Data Field (`garmin_app/`) for Custom Live Segments.
- Rust (Axum) Backend for parsing annotated GPX files and serving segment payload.
- Automated testing structure for the GPX extraction logic.
- GitHub Actions workflow for automated deployment to Railway.app.
- Web UI portal to handle GPX uploads and manage active sync token.
- Build instructions, architecture docs, and README pitch.
