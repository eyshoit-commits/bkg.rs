use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentInput {
    pub id: Option<String>,
    pub text: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentRecord {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DocumentRecord {
    pub fn new(input: DocumentInput, embedding: Option<Vec<f32>>) -> Self {
        let now = Utc::now();
        Self {
            id: input.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            text: input.text,
            metadata: input.metadata,
            embedding,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IndexRequest {
    pub collection: String,
    #[serde(default)]
    pub documents: Vec<DocumentInput>,
    #[serde(default)]
    pub embed: bool,
    #[serde(default)]
    pub fts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    pub collection: String,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub hybrid: bool,
    #[serde(default)]
    pub filters: Vec<QueryFilter>,
}

fn default_top_k() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilter {
    pub field: String,
    pub value: serde_json::Value,
    #[serde(default)]
    pub operator: FilterOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
}

impl Default for FilterOperator {
    fn default() -> Self {
        FilterOperator::Eq
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub id: String,
    pub score: f32,
    pub rank: usize,
    pub document: DocumentRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrainRequest {
    pub pipeline: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrainResponse {
    pub pipeline: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatsResponse {
    pub collections: Vec<CollectionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CollectionStats {
    pub name: String,
    pub document_count: usize,
    pub embedding_dimensions: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedDocument {
    pub id: String,
    pub score: f32,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryStrategy {
    FullText,
    Vector,
    Hybrid,
}
