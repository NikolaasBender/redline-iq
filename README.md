# 🚴‍♂️ Redline IQ

**Break free from Strava paywalls and rigid segment creation.**

Have you ever plotted an epic route on **RideWithGPS** and wished you could race custom segments along the way—without having to ride the road first to create a "Garmin Segment", and without paying for a Strava subscription?

Welcome to **Redline IQ**, the ultimate tool for cyclists who want total control over their in-ride competition.

## 🌟 Why Use Custom Live Segments?

- **Zero Subscriptions Required:** Stop paying monthly fees just to see how fast you are climbing.
- **Define Segments Anywhere:** Add markers to your GPX files or RideWithGPS routes.
- **Race the Unknown:** You don't need a prior activity history on a road to race it. Create a segment from your armchair, sync it to your device, and go smash it.
- **Seamless Cloud Sync:** Import your route via RideWithGPS URL or upload a GPX. The moment you open the Data Field on your Edge, your active segments download automatically over Bluetooth.

## 🛠️ Features

### Garmin Data Field
- **ClimbPro-style State Machine** — transitions through IDLE, APPROACHING, RACING, and RESULTS states automatically
- **Strava-style Segment Screen** — large ahead/behind timer (green/red), segment name bar, progress bar, and elevation profile
- **Native Course Integration** — uses `distanceToNextPoint` for surgical segment triggering via embedded course points
- **Dual-Marker Progress Bar** — orange dot tracks *your* position, white circle tracks the *target* pace in real time
- **Connection Status Icons** — bottom-left sync indicator and bottom-right Bluetooth icon

### Cloud Backend (Rust / Axum)
- **One-Button Course Export** — Download a `.gpx` file for your Garmin with route navigation and segment course points pre-embedded.
- **RideWithGPS Import** — Paste a route URL (e.g., `https://ridewithgps.com/routes/53786083`) to ingest telemetry automatically.
- **Segment Detection Logic**:
    - **Text-based**: Looks for `[Segment Start]` and `[Segment End]` in waypoints.
    - **POI-based**: Automatically recognizes RideWithGPS "Points of Interest" with type `segment_start` or `segment_end`.
- **Google OAuth Sign-in** — Authenticate to manage your library and get a sync token.
- **Premium Web Portal** — Dark-mode map visualizer with segment telemetry and course management.

## 🚀 Get Started

Ready to level up your training?

- 🛠️ [Building & Running](./docs/building.md)
- 📐 [Architecture Overview](./docs/architecture.md)

## 📡 API Quick Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/segments` | Fetch active segments (requires `Authorization: Bearer <token>`) |
| `POST` | `/api/upload` | Upload a GPX file (requires auth) |
| `POST` | `/api/routes/import` | Import route via URL `{"url": "..."}` |
| `POST` | `/api/routes/active` | Set active route `{"route_id": "..."}` |
| `GET` | `/api/routes` | List all routes in user's library |
| `GET` | `/auth/google` | Initiate Google OAuth login |
| `GET` | `/auth/google/callback` | OAuth callback — returns sync token |

---
*Built for the open road. Not affiliated with Garmin or Strava.*
