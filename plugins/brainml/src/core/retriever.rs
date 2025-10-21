use crate::adapters::braindb::{
    BraindbClient, CreateCollectionRequest, HybridQueryRequest, UpsertDocumentsRequest,
};
use crate::core::schema::{
    DocumentInput, DocumentRecord, QueryRequest, QueryResult, QueryStrategy,
};
use anyhow::Result;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn ensure_collection<C: BraindbClient + ?Sized>(client: &C, name: &str) -> Result<()> {
    let schema = serde_json::json!({
        "name": name,
        "fields": ["id", "text", "metadata", "embedding"],
    });
    client
        .create_collection(CreateCollectionRequest {
            collection: name.to_string(),
            schema,
        })
        .await?;
    Ok(())
}

#[instrument(skip_all, fields(collection = request.collection, documents = request.documents.len()))]
pub async fn upsert_documents<C: BraindbClient + ?Sized>(
    client: &C,
    request: UpsertDocumentsRequest,
) -> Result<()> {
    client.upsert_documents(request).await?;
    Ok(())
}

#[instrument(skip_all, fields(collection = query.collection, top_k = query.top_k))]
pub async fn hybrid_query<C: BraindbClient + ?Sized>(
    client: &C,
    query: QueryRequest,
    vector: Option<Vec<f32>>,
) -> Result<Vec<QueryResult>> {
    let strategy = if query.hybrid {
        QueryStrategy::Hybrid
    } else if vector.is_some() {
        QueryStrategy::Vector
    } else {
        QueryStrategy::FullText
    };
    let request = HybridQueryRequest {
        collection: query.collection,
        query: query.query,
        vector,
        top_k: query.top_k,
        strategy,
        filters: query.filters,
    };
    let results = client.hybrid_query(request).await?;
    Ok(results)
}

pub fn build_records(documents: &[DocumentInput], embeddings: &[Vec<f32>]) -> Vec<DocumentRecord> {
    documents
        .iter()
        .enumerate()
        .map(|(idx, doc)| {
            let embedding = embeddings.get(idx).cloned();
            DocumentRecord::new(doc.clone(), embedding)
        })
        .collect()
}
