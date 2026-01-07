# PulseSense — Real-Time Health Sensor Dashboard (FHIR + D3)

This starter repo matches the INCO (Innovation & Complexity Management) required structure:
- Rust **Actix Web** backend
- **WebSocket** real-time stream
- Backend emits **FHIR-like Observation JSON**
- Frontend uses **D3.js** and updates live
- Docker + docker-compose
- GitHub Actions CI (build + test)

## Quick start (dev)

### 1) Backend
```bash
cd backend
cp .env.example .env
cargo run --bin pulsesense-backend
```

Backend runs on: http://127.0.0.1:8080

### 2) Frontend
Open `frontend/index.html` in a browser (or serve it):
```bash
cd frontend
python -m http.server 5173
```
Then open: http://127.0.0.1:5173

### 3) Simulate sensor data
In a second terminal:
```bash
cd backend
cargo run --bin simulator
```

## API
- `POST /ingest` ingest a reading
- `GET /fhir/Observation?code=heart-rate&limit=100` query recent observations (FHIR Bundle)
- `GET /healthz` health check
- `GET /ws/live` websocket stream of new observations

## Notes

This repository is intentionally small but structured so it can grow without becoming messy.  
It already includes:

- a simple domain model for sensor readings
- input validation at ingestion time
- mapping readings to a FHIR-like Observation format
- centralized error handling
- basic telemetry/logging setup
- a live WebSocket stream for the dashboard

## Next steps

1. **Confirm the final signals**
   - Keep 3–5 signals (e.g., Heart Rate, Temperature, Steps/min, SpO₂, Battery).

2. **Connect a real data source**
   - Replace the simulator with a real sensor feed (or a CSV/serial/BLE source).
   - Ensure the payload format stays consistent with the backend ingest endpoint.

3. **Add persistence**
   - Store readings in SQLite/Postgres (and keep the in-memory mode for quick demos).
   - Add query filters (by patient/device, time range, and signal type).

4. **Improve security**
   - Add authentication (simple token/JWT).
   - Restrict access per user/device if needed.

5. **Testing + reliability**
   - Add integration tests for ingest, FHIR output, and WebSocket streaming.
   - Add basic rate limiting and better validation messages.
