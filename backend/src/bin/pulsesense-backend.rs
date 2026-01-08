use actix_web::{middleware, web, App, HttpServer};
use std::sync::{Arc, Mutex};

use pulsesense_backend::domain::store::AppState;
use pulsesense_backend::routes;
use pulsesense_backend::telemetry::init_tracing;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    // Docker-friendly default; you can override locally with HOST=127.0.0.1
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);

    let bind_addr = format!("{}:{}", host, port);

    let state = web::Data::new(Arc::new(Mutex::new(AppState::new_demo())));

    tracing::info!(%bind_addr, "starting backend");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .configure(routes::configure)
    })
    .bind(bind_addr)?
    .run()
    .await
}
