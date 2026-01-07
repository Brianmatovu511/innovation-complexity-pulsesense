use actix_web::{test, web, App};
use std::sync::{Arc, Mutex};

use pulsesense_backend::{domain::store::AppState, routes};

#[actix_rt::test]
async fn healthz_works() {
    let state = web::Data::new(Arc::new(Mutex::new(AppState::new_demo())));
    let app = test::init_service(App::new().app_data(state).configure(routes::configure)).await;

    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
