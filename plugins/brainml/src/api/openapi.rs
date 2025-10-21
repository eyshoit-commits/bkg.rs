use crate::core::schema::{
    AdminStatus, HealthResponse, IndexRequest, QueryRequest, QueryResponse, TrainRequest,
    TrainResponse,
};
use axum::routing::get;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(
        schemas(IndexRequest, QueryRequest, QueryResponse, TrainRequest, TrainResponse, AdminStatus, HealthResponse)
    ),
    tags((name = "brainml", description = "BrainML plugin API"))
)]
pub struct BrainmlApiDoc;

pub fn routes() -> axum::Router<crate::api::AppState> {
    axum::Router::new()
        .merge(SwaggerUi::new("/api/docs").url("/api-docs/openapi.json", BrainmlApiDoc::openapi()))
}
