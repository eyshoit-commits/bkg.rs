use super::errors::ApiError;
use super::AppState;
use crate::core::schema::{IndexRequest, QueryResponse};
use axum::extract::State;
use axum::routing::post;
use axum::Json;
use tracing::instrument;
use utoipa::path;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route("/api/v1/brainml/index", post(index_handler))
}

#[utoipa::path(
    post,
    path = "/api/v1/brainml/index",
    request_body = IndexRequest,
    responses((status = 200, description = "Documents indexed", body = QueryResponse)),
    tag = "brainml"
)]
#[instrument(skip_all, fields(collection = %payload.collection, docs = payload.documents.len()))]
pub async fn index_handler(
    State(state): State<AppState>,
    Json(payload): Json<IndexRequest>,
) -> Result<Json<QueryResponse>, ApiError> {
    if payload.collection.trim().is_empty() {
        return Err(ApiError::Invalid("collection is required".into()));
    }
    let response = state.process_index(payload).await.map_err(ApiError::from)?;
    Ok(Json(response))
}
