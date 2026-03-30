# Running & Testing the Custom Live Segments Project

Here is how to test and deploy the system we have built, consisting of the Rust cloud backend and the Garmin Connect IQ app.

## 1. Running the Cloud Backend (Railway & Local)

### Local Testing
The backend is a Rust Axum API with a static frontend portal. You can run it locally using Docker Compose:
```bash
cd /home/nick/Documents/coding_projects/race_segments
docker-compose up backend
```
You can access the frontend portal at `http://localhost:8080` to upload a GPX file.

### GHCR Auto-Deployment (GitHub Actions)
We have configured a `.github/workflows/docker-publish.yml` pipeline.
1. When you push to the `main` branch, GitHub Actions builds the Docker image and publishes it to the GitHub Container Registry (`ghcr.io/NikolaasBender/redline-iq-backend:latest`).
2. To run the deployed image on any server:
   ```bash
   docker run -d -p 8080:8080 ghcr.io/NikolaasBender/redline-iq-backend:latest
   ```

## 2. Compiling the Garmin App

The Connect IQ app (`garmin_app/`) uses a community Docker image to compile the Monkey C code without needing to install the SDK locally.
To build the `.prg` application file manually:
```bash
cd /home/nick/Documents/coding_projects/race_segments/garmin_app
make keys   # Generates your developer key
make build  # Uses Docker to compile the code
```
This will result in a `LiveSegments.prg` inside `garmin_app/bin/`.

## 3. Testing with the Garmin Simulator

Since the Connect IQ Simulator requires a graphical interface, you should run it locally on your computer:
1. Open Visual Studio Code and install the **Monkey C** extension by Garmin.
2. Open the SDK Manager (`Ctrl+Shift+P` -> `Monkey C: Verify Installation`) to download the latest SDK.
3. Open the `garmin_app/` folder in VS Code.
4. Press `F5` or click "Run -> Start Debugging" to launch the visual simulator for the Edge 1040.
5. Inside the simulator, you can click **Simulation -> Data Fields -> Timer** to start an activity, and **Simulation -> Fit Data -> Simulate Data** to simulate moving along a route!

## 4. Sideloading to your Garmin Edge

When you are ready to test it on a real ride:
1. Plug your Garmin Edge into your computer via USB.
2. Wait for it to show up as an external drive (like a USB stick).
3. Copy the `LiveSegments.prg` file from the `garmin_app/bin/` folder.
4. Paste it into the `Garmin/Apps/` folder on your Edge device.
5. Disconnect the device. When you go into an Activity Profile -> Data Screens -> Add Data Field, you will see "LiveSegments" under the Connect IQ Fields category!
