mod domain;
mod errors;
mod fhir;
mod routes;
mod telemetry;
mod ws;

use actix_web::{middleware, web, App, HttpServer};
use std::sync::{Arc, Mutex};

use crate::domain::store::AppState;
use crate::telemetry::init_tracing;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let state = web::Data::new(Arc::new(Mutex::new(AppState::new_demo())));

    tracing::info!(%host, %port, "starting backend");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            // IMPORTANT: NormalizePath can interfere with WebSocket upgrades.
            // Keep the logger, remove NormalizePath for reliability.
            .wrap(middleware::Logger::default())
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
