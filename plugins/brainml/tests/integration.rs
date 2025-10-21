use anyhow::Result;
use brainml::adapters::braindb::NullBraindbClient;
use brainml::adapters::llm::NullLlmClient;
use brainml::api::AppState;
use brainml::core::config::BrainmlConfig;
use brainml::core::pipeline::PipelineManager;
use brainml::core::schema::{DocumentInput, IndexRequest, QueryRequest};
use std::sync::Arc;

fn test_config() -> BrainmlConfig {
    BrainmlConfig {
        port: 43133,
        bus: "ws://127.0.0.1:43121".into(),
        embedding_model: None,
        collection_defaults: Default::default(),
    }
}

fn test_state() -> AppState {
    AppState {
        braindb: Arc::new(NullBraindbClient::default()),
        llm: Arc::new(NullLlmClient::default()),
        pipeline: PipelineManager::default(),
        config: test_config(),
        start_time: std::time::Instant::now(),
    }
}

#[tokio::test]
async fn index_then_query_returns_results() -> Result<()> {
    let state = test_state();
    let request = IndexRequest {
        collection: "docs".into(),
        documents: vec![
            DocumentInput {
                id: Some("doc-alpha".into()),
                text: "Rust makes systems programming accessible".into(),
                metadata: serde_json::json!({"lang": "en"}),
            },
            DocumentInput {
                id: Some("doc-beta".into()),
                text: "Brainml focuses on retrieval augmented workflows".into(),
                metadata: serde_json::json!({"lang": "en"}),
            },
            DocumentInput {
                id: Some("doc-gamma".into()),
                text: "Hybrid ranking blends lexical and semantic recall".into(),
                metadata: serde_json::json!({"lang": "en"}),
            },
        ],
        embed: true,
        fts: true,
    };
    let response = state.process_index(request).await?;
    assert_eq!(response.results.len(), 0);

    let query = QueryRequest {
        collection: "docs".into(),
        query: Some("Rust systems".into()),
        vector: None,
        top_k: 3,
        hybrid: true,
        filters: Vec::new(),
    };
    let response = state.process_query(query).await?;
    assert!(!response.results.is_empty());
    assert_eq!(response.results[0].document.id, "doc-alpha");
    assert!(response
        .results
        .windows(2)
        .all(|window| window[0].rank < window[1].rank));
    Ok(())
}
