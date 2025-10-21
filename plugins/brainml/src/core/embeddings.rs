use crate::adapters::llm::{EmbeddingRequest, LlmClient};
use crate::core::schema::DocumentInput;
use anyhow::Result;
use tracing::instrument;

#[instrument(skip_all, fields(batch = inputs.len()))]
pub async fn embed_documents<C: LlmClient + ?Sized>(
    client: &C,
    inputs: &[DocumentInput],
    model: Option<&str>,
) -> Result<Vec<Vec<f32>>> {
    let request = EmbeddingRequest {
        input: inputs.iter().map(|doc| doc.text.clone()).collect(),
        model: model.map(|s| s.to_string()),
    };
    let vectors = client.embed(request).await?;
    Ok(vectors.into_iter().map(|vec| vec.embedding).collect())
}
