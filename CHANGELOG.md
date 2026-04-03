# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Strava-style segment race screen with large ahead/behind timer (green = AHEAD, red = BEHIND)
- Dual-marker progress bar: orange dot (your position) vs. white circle with "T" (target pace)
- Elevation profile mini-chart with live position indicator
- Connection status icons: bottom-left cloud sync (idle/syncing/success/error), bottom-right Bluetooth
- Touch-based debug toggle for mock Bluetooth connection state (simulator testing aid)
- Request logging middleware in the Rust backend (`println!` on every incoming request)
- Google OAuth 2.0 sign-in flow (`/auth/google` + `/auth/google/callback`)
- Active route selection API (`POST /api/routes/active`)
- SQLite persistence via SQLx (`users` and `routes` tables)
- `CloudSyncer` Monkey C class for authenticated HTTP segment fetches with status callbacks
- `SegmentTracker` Monkey C class with haversine distance, 50 m start detection, and linear interpolation for ahead/behind
- Local Docker-based Garmin SDK build environment (`garmin-sdk-local` image, full Makefile workflow)
- `make simulate` and `make run` targets for the Connect IQ simulator

### Changed
- `building.md` rewritten to reflect Docker-only SDK workflow (no local SDK install required)
- `architecture.md` updated with full component breakdown and correct sequence diagram
- `README.md` updated with current feature list, API reference table, and accurate description

### Fixed
- Corrected `docs/architecture.md` sequence diagram (was missing auth and active route selection steps)

---

## [0.1.0] - 2026-03-30

### Added
- Initial Garmin Connect IQ Data Field (`garmin_app/`) for Custom Live Segments
- Rust (Axum) backend for parsing annotated GPX files and serving segment payload
- Automated testing structure for GPX extraction logic
- GitHub Actions workflow for automated Docker image publishing to GHCR
- Web portal for GPX uploads and sync token management
- Build instructions, architecture docs, and README pitch
