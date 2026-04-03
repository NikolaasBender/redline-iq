# Architecture Overview

This document illustrates how custom segments flow from your route planner to your Garmin Edge device.

## System Flow

```mermaid
sequenceDiagram
    participant U as User
    participant W as Web Portal (localhost:8080)
    participant B as Rust Backend (Axum + SQLite)
    participant G as Garmin Edge (Connect IQ)

    U->>W: 1. Sign in with Google
    W->>B: GET /auth/google → OAuth redirect
    B-->>U: Return sync token

    U->>W: 2. Upload annotated GPX file
    W->>B: POST /api/upload (Bearer token)
    B->>B: Parse [Segment Start/End] waypoints
    B-->>B: Store route + segments in SQLite

    U->>W: 3. Select active route
    W->>B: POST /api/routes/active { route_id }
    B-->>B: Link route to user profile

    U->>W: 4. Download Course
    W->>B: GET /api/routes/:id/course.gpx
    B-->>U: Return course with [SEG] course points
    U->>G: 5. Load GPX via USB/Garmin Connect

    U->>G: 6. Start cycling activity (with Course)
    note over G,B: Garmin fetches via Bluetooth/Phone
    G->>B: GET /api/segments (Bearer token)
    B-->>G: Return compact segment JSON

    loop Every Second
        G->>G: 7. Check distanceToNextPoint vs [SEG] points
        G->>G: 8. State Machine: IDLE ⭢ APPROACHING ⭢ RACING ⭢ RESULTS
        G->>G: 9. Render ClimbPro-style vs Strava-style UI
    end
```

## Component Breakdown

### Garmin App (`garmin_app/`)

| File | Responsibility |
|------|---------------|
| `LiveSegmentApp.mc` | App entry point, manages segment list and sync status |
| `LiveSegmentView.mc` | State-based renderer: ClimbPro-style (approaching) and Strava-style (racing) |
| `LiveSegmentDelegate.mc` | Handles touch/button input |
| `SegmentTracker.mc` | 4-state machine, course-point detection, ahead/behind interpolation |
| `CloudSyncer.mc` | Fetches and parses multiple segments from JSON |

Build target: **Garmin Edge 840** (`edge840`). Built using a local Docker image (`garmin-sdk-local`) wrapping the Connect IQ SDK.

### Backend (`backend/`)

| File | Responsibility |
|------|---------------|
| `main.rs` | Axum router, API handlers (routes, segments, auto-sync) |
| `course_export.rs` | Generates GPX courses with embedded [SEG] course points |
| `auth.rs` | Google OAuth flow |
| `gpx.rs` | Parses uploaded GPX files for segment discovery |
| `db.rs` | SQLite schema management |

The backend serves its own static web portal from the `public/` directory alongside the JSON API.

## Data Model (SQLite)

```
users  → token, active_route_id
routes → id, user_id, name, segments_json
```

`segments_json` is the parsed output of the GPX file, stored as a JSON string and returned verbatim to the Garmin app.
