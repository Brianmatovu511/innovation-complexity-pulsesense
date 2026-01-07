use anyhow::Result;
use rand::Rng;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

use pulsesense_backend::domain::models::{SensorReading, SignalCode};

#[tokio::main]
async fn main() -> Result<()> {
    let base = std::env::var("BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let client = Client::new();

    // Optional token support (if you set INGEST_TOKEN in .env)
    let token = std::env::var("INGEST_TOKEN").ok();

    loop {
        let reading = SensorReading {
            patient_id: "demo-patient-1".into(),
            device_id: "simulator-1".into(),
            code: SignalCode::HeartRate,
            value: rand::thread_rng().gen_range(60.0..95.0),
            unit: "bpm".into(),
            ts: chrono::Utc::now(),
        };

        let mut req = client.post(format!("{}/ingest", base)).json(&reading);
        if let Some(t) = token.as_deref() {
            if !t.trim().is_empty() {
                req = req.header("authorization", format!("Bearer {}", t));
            }
        }

        let res = req.send().await?;
        let status = res.status();

        if !status.is_success() {
            let txt = res.text().await.unwrap_or_default();
            eprintln!("ingest failed: {} {}", status, txt);
        }

        sleep(Duration::from_millis(800)).await;
    }
}
