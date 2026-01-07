use actix::prelude::*; // IMPORTANT: brings StreamHandler, ActorContext, AsyncContext, etc.
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::domain::models::{SensorReading, SignalCode};
use crate::domain::store::AppState;
use crate::errors::AppError;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz))
        .route("/ingest", web::post().to(ingest))
        .route("/fhir/Observation", web::get().to(get_observations))
        .route("/ws/live", web::get().to(ws_live));
}

async fn healthz() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok"}))
}

fn check_ingest_token(req: &HttpRequest) -> Result<(), AppError> {
    let token = std::env::var("INGEST_TOKEN").unwrap_or_default();
    if token.trim().is_empty() {
        return Ok(());
    }

    let header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // expecting: Bearer <token>
    if header == format!("Bearer {}", token) {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

async fn ingest(
    state: web::Data<Arc<Mutex<AppState>>>,
    req: HttpRequest,
    payload: web::Json<SensorReading>,
) -> Result<HttpResponse, AppError> {
    check_ingest_token(&req)?;

    let reading = payload.into_inner();
    let mut s = state.lock().unwrap();
    s.validate(&reading)?;
    let stored = s.add_reading(reading);
    Ok(HttpResponse::Ok().json(stored))
}

#[derive(Debug, Deserialize)]
struct ObsQuery {
    code: Option<String>,
    limit: Option<usize>,
    from: Option<String>,
    to: Option<String>,
}

fn parse_code(code: &str) -> Option<SignalCode> {
    match code {
        "heart-rate" => Some(SignalCode::HeartRate),
        "body-temperature" => Some(SignalCode::BodyTemperature),
        "steps-per-minute" => Some(SignalCode::StepsPerMinute),
        _ => None,
    }
}

fn parse_dt(s: &Option<String>) -> Result<Option<DateTime<Utc>>, AppError> {
    if let Some(v) = s {
        let dt = DateTime::parse_from_rfc3339(v)
            .map_err(|_| AppError::Validation("from/to must be RFC3339 timestamps".into()))?
            .with_timezone(&Utc);
        Ok(Some(dt))
    } else {
        Ok(None)
    }
}

async fn get_observations(
    state: web::Data<Arc<Mutex<AppState>>>,
    q: web::Query<ObsQuery>,
) -> Result<HttpResponse, AppError> {
    let code = q.code.as_deref().and_then(parse_code);
    let limit = q.limit.unwrap_or(200).min(2000);

    let from = parse_dt(&q.from)?;
    let to = parse_dt(&q.to)?;

    let s = state.lock().unwrap();
    let obs = s.query(code, limit, from, to);
    let bundle = crate::fhir::to_bundle(&obs)?;
    Ok(HttpResponse::Ok().json(bundle))
}

// -------------------------
// WebSocket: actor-based (reliable)
// -------------------------

struct LiveWs {
    state: Arc<Mutex<AppState>>,
    client_id: Option<u64>,
}

impl LiveWs {
    fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
            client_id: None,
        }
    }
}

// Message used to push hub text into the websocket
struct PushTxt(String);

impl Message for PushTxt {
    type Result = ();
}

impl Actor for LiveWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Channel for this websocket client to receive broadcasts
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        // Register in hub
        let hub = { self.state.lock().unwrap().ws_hub.clone() };
        let id = hub.add_client(tx);
        self.client_id = Some(id);

        // Hello
        ctx.text(r#"{"type":"hello","msg":"connected"}"#);

        // Forward hub -> websocket (tokio -> actix bridge)
        let addr = ctx.address();
        actix_rt::spawn(async move {
            while let Some(txt) = rx.recv().await {
                addr.do_send(PushTxt(txt));
            }
        });

        // Keepalive ping (helps prevent idle disconnects)
        ctx.run_interval(Duration::from_secs(20), |_actor, ctx| {
            ctx.ping(b"ping");
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(id) = self.client_id.take() {
            let hub = { self.state.lock().unwrap().ws_hub.clone() };
            hub.remove_client(id);
        }
    }
}

impl Handler<PushTxt> for LiveWs {
    type Result = ();

    fn handle(&mut self, msg: PushTxt, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// IMPORTANT: StreamHandler from actix::prelude (NOT ws::StreamHandler)
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LiveWs {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(_)) => {}
            Ok(ws::Message::Binary(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

async fn ws_live(
    state: web::Data<Arc<Mutex<AppState>>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(LiveWs::new(state.get_ref().clone()), &req, stream)
}
