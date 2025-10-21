use anyhow::Result;
use brainml::adapters::braindb::NullBraindbClient;
use brainml::adapters::llm::NullLlmClient;
use brainml::api::AppState;
use brainml::core::config::BrainmlConfig;
use brainml::core::pipeline::PipelineManager;
use brainml::core::schema::{DocumentInput, IndexRequest, QueryRequest};
use std::sync::Arc;

fn state() -> AppState {
    AppState {
        braindb: Arc::new(NullBraindbClient::default()),
        llm: Arc::new(NullLlmClient::default()),
        pipeline: PipelineManager::default(),
        config: BrainmlConfig {
            port: 43133,
            bus: "ws://127.0.0.1:43121".into(),
            embedding_model: None,
            collection_defaults: Default::default(),
        },
        start_time: std::time::Instant::now(),
    }
}

#[tokio::test]
async fn hybrid_query_combines_signals() -> Result<()> {
    let state = state();
    state
        .process_index(IndexRequest {
            collection: "hybrid".into(),
            documents: vec![
                DocumentInput {
                    id: Some("doc-a".into()),
                    text: "vector heavy context embedding".into(),
                    metadata: serde_json::json!({}),
                },
                DocumentInput {
                    id: Some("doc-b".into()),
                    text: "lexical keyword match".into(),
                    metadata: serde_json::json!({}),
                },
            ],
            embed: true,
            fts: true,
        })
        .await?;

    let response = state
        .process_query(QueryRequest {
            collection: "hybrid".into(),
            query: Some("keyword".into()),
            vector: None,
            top_k: 2,
            hybrid: true,
            filters: Vec::new(),
        })
        .await?;
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].document.id, "doc-b");
    assert!(response.results[0].score >= response.results[1].score);
    Ok(())
}
