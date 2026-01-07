use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub device_id: String,
    pub patient_id: String,
    pub code: SignalCode,
    pub value: f64,
    pub unit: String,
    pub ts: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalCode {
    HeartRate,
    BodyTemperature,
    StepsPerMinute,
}

impl SignalCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalCode::HeartRate => "heart-rate",
            SignalCode::BodyTemperature => "body-temperature",
            SignalCode::StepsPerMinute => "steps-per-minute",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredObservation {
    pub id: Uuid,
    pub reading: SensorReading,
}
