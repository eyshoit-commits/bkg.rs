use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::core::schema::{DocumentRecord, QueryFilter, QueryResult, QueryStrategy};

#[derive(Debug, Error)]
pub enum BraindbError {
    #[error("connection error: {0}")]
    Connection(String),
    #[error("request failed: {0}")]
    Request(String),
    #[error("unexpected response: {0}")]
    Response(String),
}

pub type BraindbResult<T> = Result<T, BraindbError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub collection: String,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertDocumentsRequest {
    pub collection: String,
    pub documents: Vec<DocumentRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridQueryRequest {
    pub collection: String,
    pub query: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub top_k: usize,
    pub strategy: QueryStrategy,
    pub filters: Vec<QueryFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub collections: Vec<CollectionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub name: String,
    pub document_count: usize,
    pub embedding_dimensions: Option<usize>,
}

#[async_trait]
pub trait BraindbClient: Send + Sync {
    async fn create_collection(&self, request: CreateCollectionRequest) -> BraindbResult<()>;
    async fn upsert_documents(&self, request: UpsertDocumentsRequest) -> BraindbResult<()>;
    async fn hybrid_query(&self, request: HybridQueryRequest) -> BraindbResult<Vec<QueryResult>>;
    async fn stats(&self) -> BraindbResult<StatsResponse>;
}

#[derive(Clone, Default)]
pub struct NullBraindbClient {
    state: Arc<RwLock<indexmap::IndexMap<String, Vec<DocumentRecord>>>>,
}

#[async_trait]
impl BraindbClient for NullBraindbClient {
    #[instrument(skip_all, fields(collection = %request.collection))]
    async fn create_collection(&self, request: CreateCollectionRequest) -> BraindbResult<()> {
        let mut state = self.state.write().await;
        state.entry(request.collection).or_default();
        Ok(())
    }

    #[instrument(skip_all, fields(collection = %request.collection, count = request.documents.len()))]
    async fn upsert_documents(&self, request: UpsertDocumentsRequest) -> BraindbResult<()> {
        let mut state = self.state.write().await;
        let docs = state.entry(request.collection).or_default();
        for record in request.documents {
            if let Some(existing) = docs.iter_mut().find(|doc| doc.id == record.id) {
                *existing = record;
            } else {
                docs.push(record);
            }
        }
        Ok(())
    }

    #[instrument(skip_all, fields(collection = %request.collection, top_k = request.top_k))]
    async fn hybrid_query(&self, request: HybridQueryRequest) -> BraindbResult<Vec<QueryResult>> {
        let state = self.state.read().await;
        let Some(docs) = state.get(&request.collection) else {
            return Ok(vec![]);
        };
        let mut results = Vec::new();
        for doc in docs {
            let mut score = 0.0;
            if let Some(ref query) = request.query {
                if doc.text.to_lowercase().contains(&query.to_lowercase()) {
                    score += 0.5;
                }
            }
            if let Some(ref vector) = request.vector {
                if let Some(embedding) = &doc.embedding {
                    let dot: f32 = embedding
                        .iter()
                        .zip(vector.iter())
                        .map(|(a, b)| a * b)
                        .sum();
                    score += dot;
                }
            }
            if score > 0.0 {
                results.push(QueryResult {
                    id: doc.id.clone(),
                    score,
                    document: doc.clone(),
                    rank: 0,
                });
            }
        }
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(request.top_k);
        for (idx, item) in results.iter_mut().enumerate() {
            item.rank = idx + 1;
        }
        Ok(results)
    }

    #[instrument(skip_all)]
    async fn stats(&self) -> BraindbResult<StatsResponse> {
        let state = self.state.read().await;
        let mut collections = Vec::new();
        for (name, docs) in state.iter() {
            let embedding_dimensions = docs
                .iter()
                .filter_map(|doc| doc.embedding.as_ref())
                .map(|embedding| embedding.len())
                .next();
            collections.push(CollectionStats {
                name: name.clone(),
                document_count: docs.len(),
                embedding_dimensions,
            });
        }
        Ok(StatsResponse { collections })
    }
}

#[derive(Clone)]
pub struct PluginBusBraindbClient {
    pub(crate) sender: tokio::sync::mpsc::Sender<crate::core::bus::OutboundCommand>,
}

#[async_trait]
impl BraindbClient for PluginBusBraindbClient {
    #[instrument(skip_all, fields(collection = %request.collection))]
    async fn create_collection(&self, request: CreateCollectionRequest) -> BraindbResult<()> {
        let payload = serde_json::to_value(&request)
            .map_err(|err| BraindbError::Request(format!("serialization error: {err}")))?;
        self.invoke("db.createCollection", payload)
            .await
            .map(|_| ())
    }

    #[instrument(skip_all, fields(collection = %request.collection, count = request.documents.len()))]
    async fn upsert_documents(&self, request: UpsertDocumentsRequest) -> BraindbResult<()> {
        let payload = serde_json::to_value(&request)
            .map_err(|err| BraindbError::Request(format!("serialization error: {err}")))?;
        self.invoke("db.upsert", payload).await.map(|_| ())
    }

    #[instrument(skip_all)]
    async fn hybrid_query(&self, request: HybridQueryRequest) -> BraindbResult<Vec<QueryResult>> {
        let payload = serde_json::to_value(&request)
            .map_err(|err| BraindbError::Request(format!("serialization error: {err}")))?;
        let value = self.invoke("db.hybridQuery", payload).await?;
        serde_json::from_value(value).map_err(|err| BraindbError::Response(format!("{err}")))
    }

    #[instrument(skip_all)]
    async fn stats(&self) -> BraindbResult<StatsResponse> {
        let value = self.invoke("db.stats", serde_json::Value::Null).await?;
        serde_json::from_value(value).map_err(|err| BraindbError::Response(format!("{err}")))
    }
}

impl PluginBusBraindbClient {
    async fn invoke(
        &self,
        capability: &str,
        payload: serde_json::Value,
    ) -> BraindbResult<serde_json::Value> {
        let request_id = Uuid::new_v4();
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let cmd = crate::core::bus::OutboundCommand::Invoke {
            request_id,
            capability: capability.to_string(),
            payload,
            responder: resp_tx,
        };
        self.sender
            .send(cmd)
            .await
            .map_err(|err| BraindbError::Connection(format!("{err}")))?;
        let result = resp_rx
            .await
            .map_err(|err| BraindbError::Connection(format!("{err}")))?;
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                warn!(%capability, error = %err, "braindb invocation failed");
                Err(BraindbError::Request(err))
            }
        }
    }
}
