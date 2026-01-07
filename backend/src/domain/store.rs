use crate::domain::models::{SensorReading, SignalCode, StoredObservation};
use crate::errors::AppError;
use crate::fhir;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::VecDeque;
use uuid::Uuid;

const MAX_BUFFER: usize = 2_000;

#[derive(Debug)]
pub struct AppState {
    pub observations: VecDeque<StoredObservation>,
    pub ws_hub: crate::ws::Hub,
    pub demo_patient_id: String,
    pub demo_device_id: String,
}

impl AppState {
    pub fn new_demo() -> Self {
        Self {
            observations: VecDeque::with_capacity(MAX_BUFFER),
            ws_hub: crate::ws::Hub::new(),
            demo_patient_id: "patient-001".to_string(),
            demo_device_id: "device-001".to_string(),
        }
    }

    pub fn validate(&self, r: &SensorReading) -> Result<(), AppError> {
        if r.device_id.trim().is_empty() || r.patient_id.trim().is_empty() {
            return Err(AppError::Validation("device_id and patient_id are required".into()));
        }

        // Simple range validation (extend for higher grade)
        match r.code {
            SignalCode::HeartRate => {
                if !(20.0..=240.0).contains(&r.value) {
                    return Err(AppError::Validation("heart-rate out of range (20..240)".into()));
                }
                if r.unit != "beats/min" && r.unit != "bpm" {
                    return Err(AppError::Validation("heart-rate unit should be 'beats/min' or 'bpm'".into()));
                }
            }
            SignalCode::BodyTemperature => {
                if !(30.0..=45.0).contains(&r.value) {
                    return Err(AppError::Validation("body-temperature out of range (30..45)".into()));
                }
                if r.unit != "C" && r.unit != "°C" {
                    return Err(AppError::Validation("body-temperature unit should be 'C' or '°C'".into()));
                }
            }
            SignalCode::StepsPerMinute => {
                if !(0.0..=400.0).contains(&r.value) {
                    return Err(AppError::Validation("steps-per-minute out of range (0..400)".into()));
                }
                if r.unit != "steps/min" {
                    return Err(AppError::Validation("steps-per-minute unit should be 'steps/min'".into()));
                }
            }
        }
        Ok(())
    }

    pub fn add_reading(&mut self, reading: SensorReading) -> StoredObservation {
        let obs = StoredObservation {
            id: Uuid::new_v4(),
            reading,
        };

        if self.observations.len() >= MAX_BUFFER {
            self.observations.pop_front();
        }
        self.observations.push_back(obs.clone());

        // Broadcast FHIR Observation to websocket subscribers
        if let Ok(fhir_obs) = fhir::to_fhir_observation(&obs) {
            self.ws_hub.broadcast_json(&fhir_obs);
        }

        obs
    }

    pub fn query(
        &self,
        code: Option<SignalCode>,
        limit: usize,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Vec<StoredObservation> {
        let mut out: Vec<StoredObservation> = self
            .observations
            .iter()
            .filter(|o| code.map(|c| o.reading.code as u8 == c as u8).unwrap_or(true))
            .filter(|o| from.map(|f| o.reading.ts >= f).unwrap_or(true))
            .filter(|o| to.map(|t| o.reading.ts <= t).unwrap_or(true))
            .cloned()
            .collect();

        out.sort_by_key(|o| o.reading.ts);
        out.into_iter().rev().take(limit).collect::<Vec<_>>().into_iter().rev().collect()
    }
}

// Small summary type for UI/debug
#[derive(Debug, Serialize)]
pub struct Summary {
    pub count: usize,
    pub latest_ts: Option<DateTime<Utc>>,
}
