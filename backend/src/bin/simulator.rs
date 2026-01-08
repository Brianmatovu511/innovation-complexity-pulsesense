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

    // Optional: allow overriding IDs from env
    let patient_id = std::env::var("PATIENT_ID").unwrap_or_else(|_| "demo-patient-1".into());
    let device_id = std::env::var("DEVICE_ID").unwrap_or_else(|_| "simulator-1".into());

    // Helper: send one reading with tiny retry/backoff (keeps simulator alive)
    async fn send_reading(
        client: &Client,
        base: &str,
        token: Option<&str>,
        reading: &SensorReading,
    ) -> Result<()> {
        let mut attempt = 0u32;

        loop {
            let mut req = client.post(format!("{}/ingest", base)).json(reading);

            if let Some(t) = token {
                if !t.trim().is_empty() {
                    req = req.header("authorization", format!("Bearer {}", t));
                }
            }

            match req.send().await {
                Ok(res) => {
                    let status = res.status();
                    if !status.is_success() {
                        let txt = res.text().await.unwrap_or_default();
                        eprintln!("ingest failed: {} {}", status, txt);
                    }
                    return Ok(());
                }
                Err(e) => {
                    attempt += 1;
                    let wait_ms = (200u64).saturating_mul(attempt as u64).min(1500);
                    eprintln!("ingest error (attempt {}): {} — retrying in {}ms", attempt, e, wait_ms);
                    sleep(Duration::from_millis(wait_ms)).await;
                    // keep retrying forever (simulator should not die)
                }
            }
        }
    }

    // Stateful signals (random-walk) so charts look smooth and realistic
    let mut rng = rand::thread_rng();
    let mut hr_value: f64 = rng.gen_range(68.0..82.0);
    let mut temp_value: f64 = rng.gen_range(36.4..36.9);

    // Steps state: 0 = idle, 1 = walk, 2 = run
    let mut mode: u8 = 0;
    let mut mode_ticks_left: i32 = 10;

    loop {
        // Tick every ~800ms
        // We emit 3 readings per tick so the dashboard updates continuously.

        // --- 1) Heart Rate (bpm): random walk + occasional spikes
        {
            // small drift
            hr_value += rng.gen_range(-1.2..1.2);

            // occasional short spike (like movement)
            if rng.gen_bool(0.05) {
                hr_value += rng.gen_range(6.0..18.0);
            }

            // clamp
            hr_value = hr_value.clamp(50.0, 140.0);

            let hr = SensorReading {
                patient_id: patient_id.clone(),
                device_id: device_id.clone(),
                code: SignalCode::HeartRate,
                value: hr_value,
                unit: "bpm".into(),
                ts: chrono::Utc::now(),
            };

            send_reading(&client, &base, token.as_deref(), &hr).await?;
        }

        // --- 2) Body Temperature (°C): very slow drift, rare fever-ish bump
        {
            temp_value += rng.gen_range(-0.02..0.02);

            // rare bump
            if rng.gen_bool(0.01) {
                temp_value += rng.gen_range(0.1..0.4);
            }

            temp_value = temp_value.clamp(35.8, 38.8);

            let temp = SensorReading {
                patient_id: patient_id.clone(),
                device_id: device_id.clone(),
                code: SignalCode::BodyTemperature,
                value: temp_value,
                unit: "°C".into(),
                ts: chrono::Utc::now(),
            };

            send_reading(&client, &base, token.as_deref(), &temp).await?;
        }

        // --- 3) Steps per minute: idle/walk/run modes with bursts
        {
            mode_ticks_left -= 1;
            if mode_ticks_left <= 0 {
                // change mode occasionally
                mode = match mode {
                    0 => {
                        if rng.gen_bool(0.55) { 1 } else { 0 } // idle -> often walk
                    }
                    1 => {
                        if rng.gen_bool(0.18) { 2 } else if rng.gen_bool(0.25) { 0 } else { 1 }
                    }
                    _ => {
                        if rng.gen_bool(0.45) { 1 } else { 0 } // run -> settle back
                    }
                };
                mode_ticks_left = match mode {
                    0 => rng.gen_range(6..18),  // idle lasts
                    1 => rng.gen_range(8..24),  // walk lasts
                    _ => rng.gen_range(3..10),  // run bursts are short
                };
            }

            let steps_value: f64 = match mode {
                0 => rng.gen_range(0.0..8.0),
                1 => rng.gen_range(20.0..60.0),
                _ => rng.gen_range(90.0..160.0),
            };

            let steps = SensorReading {
                patient_id: patient_id.clone(),
                device_id: device_id.clone(),
                code: SignalCode::StepsPerMinute,
                value: steps_value,
                unit: "steps/min".into(),
                ts: chrono::Utc::now(),
            };

            send_reading(&client, &base, token.as_deref(), &steps).await?;
        }

        sleep(Duration::from_millis(800)).await;
    }
}
