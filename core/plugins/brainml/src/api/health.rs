use super::AppState;
use crate::core::schema::HealthResponse;
use axum::extract::State;
use axum::routing::get;
use axum::Json;
use utoipa::path;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/health/live", get(live_handler))
        .route("/health/ready", get(ready_handler))
}

#[utoipa::path(
    get,
    path = "/health/live",
    responses((status = 200, description = "Liveness", body = HealthResponse)),
    tag = "brainml"
)]
async fn live_handler(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        details: None,
    })
}

#[utoipa::path(
    get,
    path = "/health/ready",
    responses((status = 200, description = "Readiness", body = HealthResponse)),
    tag = "brainml"
)]
async fn ready_handler(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ready".into(),
        details: None,
    })
}
