pub mod admin;
pub mod errors;
pub mod health;
pub mod index;
pub mod openapi;
pub mod query;

use crate::core::embeddings::embed_documents;
use crate::core::pipeline::PipelineManager;
use crate::core::ranker::reciprocal_rank_fusion;
use crate::core::retriever::{build_records, ensure_collection, hybrid_query, upsert_documents};
use crate::core::schema::{
    AdminStatus, IndexRequest, QueryRequest, QueryResponse, TrainRequest, TrainResponse,
};
use crate::core::scoring::normalize_scores;
use crate::{adapters::braindb::BraindbClient, adapters::llm::LlmClient};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, Router};
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct AppState {
    pub braindb: Arc<dyn BraindbClient>,
    pub llm: Arc<dyn LlmClient>,
    pub pipeline: PipelineManager,
    pub config: crate::core::config::BrainmlConfig,
    pub start_time: std::time::Instant,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(index::routes())
        .merge(query::routes())
        .merge(admin::routes())
        .merge(health::routes())
        .merge(openapi::routes())
        .with_state(state)
}

impl AppState {
    #[instrument(skip_all, fields(collection = %request.collection))]
    pub async fn process_index(
        &self,
        request: IndexRequest,
    ) -> Result<QueryResponse, anyhow::Error> {
        ensure_collection(self.braindb.as_ref(), &request.collection).await?;
        let embeddings = if request.embed {
            embed_documents(
                self.llm.as_ref(),
                &request.documents,
                self.config.embedding_model.as_deref(),
            )
            .await?
        } else {
            vec![Vec::new(); request.documents.len()]
        };
        let records = build_records(&request.documents, &embeddings);
        upsert_documents(
            self.braindb.as_ref(),
            crate::adapters::braindb::UpsertDocumentsRequest {
                collection: request.collection.clone(),
                documents: records,
            },
        )
        .await?;
        Ok(QueryResponse {
            results: Vec::new(),
            latency_ms: 0,
        })
    }

    #[instrument(skip_all, fields(collection = %request.collection))]
    pub async fn process_query(
        &self,
        request: QueryRequest,
    ) -> Result<QueryResponse, anyhow::Error> {
        let mut payload = request;
        let vector = if payload.vector.is_none() && payload.hybrid {
            if let Some(query) = &payload.query {
                embed_documents(
                    self.llm.as_ref(),
                    &[crate::core::schema::DocumentInput {
                        id: None,
                        text: query.clone(),
                        metadata: serde_json::Value::Null,
                    }],
                    self.config.embedding_model.as_deref(),
                )
                .await?
                .into_iter()
                .next()
            } else {
                None
            }
        } else {
            payload.vector.clone()
        };
        let mut results = hybrid_query(self.braindb.as_ref(), payload, vector).await?;
        results = reciprocal_rank_fusion(results, self.config.collection_defaults.rrf_k);
        normalize_scores(&mut results);
        Ok(QueryResponse {
            latency_ms: 0,
            results,
        })
    }

    pub async fn process_train(
        &self,
        request: TrainRequest,
    ) -> Result<TrainResponse, anyhow::Error> {
        self.pipeline.train(request).await
    }
}
