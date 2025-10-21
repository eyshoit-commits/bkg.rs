use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BrainmlConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    #[serde(default = "default_bus")]
    pub bus: String,
    #[serde(default)]
    pub embedding_model: Option<String>,
    #[serde(default)]
    pub collection_defaults: CollectionDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct CollectionDefaults {
    #[serde(default = "default_top_k")]
    #[validate(range(min = 1, max = 200))]
    pub top_k: usize,
    #[serde(default = "default_rrf_k")]
    #[validate(range(min = 1, max = 100))]
    pub rrf_k: usize,
}

fn default_bus() -> String {
    "ws://127.0.0.1:43121".to_string()
}

fn default_top_k() -> usize {
    10
}

fn default_rrf_k() -> usize {
    60
}

#[derive(Debug)]
pub struct BrainmlConfigLoader;

impl BrainmlConfigLoader {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<BrainmlConfig> {
        let contents = fs::read_to_string(path.as_ref()).context("reading brainml config")?;
        let config: BrainmlConfig =
            serde_json::from_str(&contents).context("parsing brainml config")?;
        config.validate().context("validating brainml config")?;
        Ok(config)
    }
}
