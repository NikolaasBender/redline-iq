# Building & Running Redline IQ

A guide to building, simulating, and deploying the Garmin app and cloud backend.

## Prerequisites

- **Docker** (used for both the Garmin SDK build toolchain and the backend)
- A `docker volume` named `garmin-sdk-devices` (created automatically by `make setup-device`)

---

## 1. Cloud Backend

### Run Locally

The backend is a Rust Axum API + static web portal, containerized via Docker Compose:

```bash
cd /path/to/race_segments
docker compose up backend
```

- API + portal available at `http://localhost:8080`
- SQLite database persisted in a named Docker volume (`backend-data`)
- Set `RUST_LOG=debug` is already configured in `docker-compose.yml`

### Environment Variables

Create a `.env` file in `backend/` for Google OAuth:

```
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret
REDIRECT_URL=http://localhost:8080/auth/google/callback
DATABASE_URL=sqlite:data/race_segments.db?mode=rwc
```

### Deploy via GHCR (GitHub Actions)

Pushing to `main` triggers `.github/workflows/docker-publish.yml`, which builds and publishes the image to GHCR:

```bash
docker pull ghcr.io/NikolaasBender/redline-iq-backend:latest
docker run -d -p 8080:8080 \
  -e GOOGLE_CLIENT_ID=... \
  -e GOOGLE_CLIENT_SECRET=... \
  -e REDIRECT_URL=https://your-domain/auth/google/callback \
  -e DATABASE_URL=sqlite:data/race_segments.db?mode=rwc \
  ghcr.io/NikolaasBender/redline-iq-backend:latest
```

---

## 2. Garmin Data Field

All Garmin build commands run via the `Makefile` inside `garmin_app/`. The SDK itself runs inside the `garmin-sdk-local` Docker image — no local SDK install required.

### First-Time Setup

```bash
cd garmin_app

# Build the local SDK Docker image
make setup

# Download device/simulator profiles (opens the Garmin SDK Manager GUI)
make setup-device
```

### Generate Developer Keys

Only needed once. Creates `developer_key.pem` and `developer_key.der`:

```bash
make keys
```

### Build the App

Compiles Monkey C source into `bin/LiveSegments.prg`:

```bash
make build
```

Target device is **Garmin Edge 840** (`edge840`). To change the target, edit the `DEVICE` variable at the top of `Makefile`.

### Run in Simulator

Launches the Connect IQ simulator (requires X11 display):

```bash
# Start the simulator in the background
make simulate

# Build + run the data field inside the simulator
make run
```

Inside the simulator:
- **Simulation → Data Fields → Timer** to start a fake activity
- **Simulation → Fit Data → Simulate Data** to simulate GPS movement

---

## 3. Sideloading to a Real Device

1. Build the app: `make build`
2. Plug your Garmin Edge into your computer via USB
3. Copy `garmin_app/bin/LiveSegments.prg` to `Garmin/Apps/` on the device
4. Disconnect; navigate to **Activity Profile → Data Screens → Add Data Field → Connect IQ Fields** and select **LiveSegments**

---

## 4. Testing the Sync Flow

1. Start the backend: `docker compose up backend`
2. Open `http://localhost:8080` and sign in (or use the mock `/login` endpoint in development)
3. Upload a GPX file with `[Segment Start]` / `[Segment End]` waypoints in the `<desc>` field
4. Note the returned `route_id` and call `POST /api/routes/active` with it
5. Run the app in the simulator — it will call `GET /api/segments` using the auth token and load your segment
