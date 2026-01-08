# PulseSense — Real-Time Health Sensor Dashboard (FHIR + D3)

PulseSense is a containerized, real-time health monitoring demo that streams simulated physiological sensor data to a live web dashboard.  
It demonstrates modern backend architecture, real-time communication, and **FHIR-aligned healthcare data modeling**.

This project was built to satisfy the **Innovation & Complexity Management** course requirements and to serve as a solid technical portfolio example.

---

## ✨ Key Features

- 🦀 **Rust + Actix Web** backend  
- 🔁 **WebSocket** real-time data streaming  
- 🏥 **FHIR-compliant Observation JSON (FHIR R4–style)**  
- 📊 **D3.js** live data visualization  
- 🤖 Built-in **sensor simulator**  
- 🐳 Fully **Dockerized** (backend, frontend, simulator)  
- ⚕️ Health-checked service orchestration with Docker Compose  

---

## 🏥 FHIR Compliance (Overview)

PulseSense emits data modeled after **HL7 FHIR Observation resources**.

Each measurement follows the FHIR Observation structure, including:

- `resourceType: Observation`
- `status`
- `code` (heart rate, body temperature, steps/min)
- `subject` (patient reference)
- `device`
- `effectiveDateTime`
- `valueQuantity`

FHIR-like Observations are available via:

- REST API (`/fhir/Observation`)
- Live WebSocket stream (`/ws/live`)

This makes PulseSense suitable for **health informatics demonstrations** and future interoperability extensions.

---

## 🚀 One-Command Quick Start (Recommended)

### Requirements
- Docker
- Docker Compose

### Run everything (backend + frontend + simulator)

```bash
docker compose up --build
```

Then open:

- **Frontend Dashboard:** http://127.0.0.1:5173  
- **Backend Health Check:** http://127.0.0.1:8080/healthz  

✔ Backend, frontend, and simulator start automatically  
✔ Live charts update in real time  

---

## 🧠 Architecture Overview

```text
┌──────────────┐
│  Simulator   │
│ (Rust binary)│
└──────┬───────┘
       │ HTTP /ingest
       ▼
┌─────────────────────────┐
│  Backend (Actix Web)    │
│                         │
│ - REST API              │
│ - FHIR Observation map  │
│ - WebSocket /ws/live    │
│ - In-memory store       │
└──────┬─────────┬────────┘
       │ WS       │ REST
       ▼          ▼
┌─────────────────────────┐
│ Frontend (Nginx + D3)   │
│                         │
│ - Live charts           │
│ - Status indicators     │
│ - FHIR stream view      │
└─────────────────────────┘
```

---

## 🔌 API Endpoints

- `POST /ingest` — ingest a sensor reading  
- `GET /fhir/Observation?code=heart-rate&limit=100` — query recent observations  
- `GET /healthz` — backend health check  
- `GET /ws/live` — WebSocket stream of new observations  

---

## 👤 Author

**Brian Doctor**  
Health Informatics (B.Sc.)  
INCO — Innovation & Complexity Management