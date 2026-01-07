use crate::domain::models::{SignalCode, StoredObservation};
use crate::errors::AppError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct FhirReference {
    pub reference: String,
}

#[derive(Debug, Serialize)]
pub struct FhirCode {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct FhirValueQuantity {
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Serialize)]
pub struct FhirObservation {
    pub resourceType: &'static str,
    pub id: String,
    pub status: &'static str,
    pub code: FhirCode,
    pub subject: FhirReference,
    pub device: FhirReference,
    pub effectiveDateTime: DateTime<Utc>,
    pub valueQuantity: FhirValueQuantity,
}

#[derive(Debug, Serialize)]
pub struct FhirBundleEntry<T> {
    pub resource: T,
}

#[derive(Debug, Serialize)]
pub struct FhirBundle<T> {
    pub resourceType: &'static str,
    #[serde(rename = "type")]
    pub bundle_type: &'static str,
    pub total: usize,
    pub entry: Vec<FhirBundleEntry<T>>,
}

fn signal_name(code: SignalCode) -> &'static str {
    match code {
        SignalCode::HeartRate => "Heart Rate",
        SignalCode::BodyTemperature => "Body Temperature",
        SignalCode::StepsPerMinute => "Steps per Minute",
    }
}

pub fn to_fhir_observation(obs: &StoredObservation) -> Result<FhirObservation, AppError> {
    Ok(FhirObservation {
        resourceType: "Observation",
        id: obs.id.to_string(),
        status: "final",
        code: FhirCode { text: signal_name(obs.reading.code).to_string() },
        subject: FhirReference { reference: format!("Patient/{}", obs.reading.patient_id) },
        device: FhirReference { reference: format!("Device/{}", obs.reading.device_id) },
        effectiveDateTime: obs.reading.ts,
        valueQuantity: FhirValueQuantity { value: obs.reading.value, unit: obs.reading.unit.clone() },
    })
}

pub fn to_bundle(observations: &[StoredObservation]) -> Result<FhirBundle<FhirObservation>, AppError> {
    let mut entry = Vec::with_capacity(observations.len());
    for o in observations {
        entry.push(FhirBundleEntry { resource: to_fhir_observation(o)? });
    }
    Ok(FhirBundle {
        resourceType: "Bundle",
        bundle_type: "searchset",
        total: observations.len(),
        entry,
    })
}
