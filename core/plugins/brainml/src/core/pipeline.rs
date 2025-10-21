use crate::core::schema::{TrainRequest, TrainResponse};
use anyhow::Result;
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone, Default)]
pub struct PipelineManager {
    state: Arc<RwLock<HashMap<String, TrainResponse>>>,
}

impl PipelineManager {
    #[instrument(skip_all)]
    pub async fn train(&self, request: TrainRequest) -> Result<TrainResponse> {
        let mut state = self.state.write();
        let response = TrainResponse {
            pipeline: request.pipeline.clone(),
            status: "trained".to_string(),
            updated_at: Utc::now(),
        };
        state.insert(request.pipeline, response.clone());
        Ok(response)
    }

    pub fn snapshot(&self) -> Vec<TrainResponse> {
        self.state.read().values().cloned().collect()
    }
}
