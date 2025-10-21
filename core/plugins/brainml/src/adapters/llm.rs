use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::core::bus::OutboundCommand;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("connection error: {0}")]
    Connection(String),
    #[error("request error: {0}")]
    Request(String),
    #[error("response error: {0}")]
    Response(String),
}

pub type LlmResult<T> = Result<T, LlmError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub input: Vec<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub embedding: Vec<f32>,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn embed(&self, request: EmbeddingRequest) -> LlmResult<Vec<EmbeddingVector>>;
}

#[derive(Clone)]
pub struct PluginBusLlmClient {
    pub(crate) sender: tokio::sync::mpsc::Sender<OutboundCommand>,
}

#[async_trait]
impl LlmClient for PluginBusLlmClient {
    #[instrument(skip_all, fields(batch = request.input.len()))]
    async fn embed(&self, request: EmbeddingRequest) -> LlmResult<Vec<EmbeddingVector>> {
        let payload = serde_json::to_value(&request)
            .map_err(|err| LlmError::Request(format!("serialization error: {err}")))?;
        let response = self
            .invoke("llm.embed", payload)
            .await?
            .get("data")
            .cloned()
            .ok_or_else(|| LlmError::Response("missing data".into()))?;
        let vectors: Vec<EmbeddingVector> =
            serde_json::from_value(response).map_err(|err| LlmError::Response(format!("{err}")))?;
        Ok(vectors)
    }
}

impl PluginBusLlmClient {
    pub fn new(sender: tokio::sync::mpsc::Sender<OutboundCommand>) -> Self {
        Self { sender }
    }

    async fn invoke(
        &self,
        capability: &str,
        payload: serde_json::Value,
    ) -> LlmResult<serde_json::Value> {
        let request_id = Uuid::new_v4();
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let cmd = OutboundCommand::Invoke {
            request_id,
            capability: capability.to_string(),
            payload,
            responder: resp_tx,
        };
        self.sender
            .send(cmd)
            .await
            .map_err(|err| LlmError::Connection(format!("{err}")))?;
        match resp_rx
            .await
            .map_err(|err| LlmError::Connection(format!("{err}")))?
        {
            Ok(value) => Ok(value),
            Err(err) => {
                warn!(%capability, error = %err, "llm invocation failed");
                Err(LlmError::Request(err))
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct NullLlmClient;

#[async_trait]
impl LlmClient for NullLlmClient {
    #[instrument(skip_all)]
    async fn embed(&self, request: EmbeddingRequest) -> LlmResult<Vec<EmbeddingVector>> {
        Ok(request
            .input
            .into_iter()
            .map(|text| {
                let hash = crate::util::id::hash_to_floats(&text, 1536);
                EmbeddingVector { embedding: hash }
            })
            .collect())
    }
}
