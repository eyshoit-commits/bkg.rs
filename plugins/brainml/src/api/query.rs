use super::errors::ApiError;
use super::AppState;
use crate::core::schema::{QueryRequest, QueryResponse};
use axum::extract::State;
use axum::routing::post;
use axum::Json;
use tracing::instrument;
use utoipa::path;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route("/api/v1/brainml/query", post(query_handler))
}

#[utoipa::path(
    post,
    path = "/api/v1/brainml/query",
    request_body = QueryRequest,
    responses((status = 200, description = "Query results", body = QueryResponse)),
    tag = "brainml"
)]
#[instrument(skip_all, fields(collection = %payload.collection, top_k = payload.top_k))]
pub async fn query_handler(
    State(state): State<AppState>,
    Json(mut payload): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, ApiError> {
    if payload.collection.trim().is_empty() {
        return Err(ApiError::Invalid("collection is required".into()));
    }
    let response = state.process_query(payload).await.map_err(ApiError::from)?;
    Ok(Json(response))
}
