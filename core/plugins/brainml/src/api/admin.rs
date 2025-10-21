use super::errors::ApiError;
use super::AppState;
use crate::core::schema::{AdminStatus, TrainRequest, TrainResponse};
use axum::extract::State;
use axum::routing::{get, post};
use axum::Json;
use tracing::instrument;
use utoipa::path;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/api/v1/brainml/admin/status", get(status_handler))
        .route("/api/v1/brainml/train", post(train_handler))
}

#[utoipa::path(
    get,
    path = "/api/v1/brainml/admin/status",
    responses((status = 200, description = "Admin status", body = AdminStatus)),
    tag = "brainml"
)]
#[instrument(skip_all)]
pub async fn status_handler(State(state): State<AppState>) -> Result<Json<AdminStatus>, ApiError> {
    let uptime = state.start_time.elapsed().as_secs();
    Ok(Json(AdminStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        capabilities: vec![
            "brainml.index".into(),
            "brainml.query".into(),
            "brainml.train".into(),
            "brainml.stats".into(),
            "brainml.admin".into(),
        ],
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/brainml/train",
    request_body = TrainRequest,
    responses((status = 200, description = "Pipeline training started", body = TrainResponse)),
    tag = "brainml"
)]
#[instrument(skip_all, fields(pipeline = %payload.pipeline))]
pub async fn train_handler(
    State(state): State<AppState>,
    Json(payload): Json<TrainRequest>,
) -> Result<Json<TrainResponse>, ApiError> {
    if payload.pipeline.trim().is_empty() {
        return Err(ApiError::Invalid("pipeline name required".into()));
    }
    let response = state.process_train(payload).await.map_err(ApiError::from)?;
    Ok(Json(response))
}
