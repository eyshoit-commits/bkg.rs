use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::models::GooseSettings;

#[derive(Debug, Clone)]
pub struct GoosePluginConfig {
    pub name: String,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub settings: GooseSettings,
    pub config_schema: Value,
}

#[derive(Debug, Clone, Deserialize)]
struct RawPluginConfig {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub settings: GooseSettings,
}

static DEFAULT_CONFIG: Lazy<Value> = Lazy::new(|| {
    serde_json::from_str(include_str!("../config.json")).expect("invalid embedded config")
});

static CONFIG_SCHEMA: Lazy<Value> = Lazy::new(|| {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "GoosePluginSettings",
        "type": "object",
        "properties": {
            "defaultTarget": {"type": "string", "format": "uri", "description": "Base target URL for all requests"},
            "users": {"type": "integer", "minimum": 1, "maximum": 100000, "description": "Number of concurrent virtual users"},
            "hatchRate": {"type": "integer", "minimum": 1, "maximum": 100000, "description": "Users spawned per second"},
            "runTimeSeconds": {"type": "integer", "minimum": 1, "maximum": 86400, "description": "Total run time in seconds"},
            "timeoutSeconds": {"type": "integer", "minimum": 1, "maximum": 600, "description": "HTTP request timeout in seconds"},
            "globalHeaders": {"type": "object", "additionalProperties": {"type": "string"}},
            "verifyTls": {"type": "boolean", "default": true},
            "maxHistory": {"type": "integer", "minimum": 1, "maximum": 200, "default": 20},
            "schedule": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "required": ["name", "method", "path"],
                    "properties": {
                        "name": {"type": "string"},
                        "method": {"type": "string", "enum": ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD"]},
                        "path": {"type": "string"},
                        "weight": {"type": "integer", "minimum": 1, "default": 1},
                        "body": {"type": "string"},
                        "headers": {"type": "object", "additionalProperties": {"type": "string"}},
                        "query": {"type": "object", "additionalProperties": {"type": "string"}},
                        "thinkTimeMs": {"type": "integer", "minimum": 0, "maximum": 600000}
                    }
                }
            }
        },
        "required": ["defaultTarget", "schedule"],
        "additionalProperties": false
    })
});

pub fn load_plugin_config(path: Option<PathBuf>) -> Result<GoosePluginConfig> {
    let content = if let Some(path) = path {
        fs::read_to_string(&path)
            .with_context(|| format!("failed to read config at {}", path.display()))?
    } else {
        DEFAULT_CONFIG.to_string()
    };
    let value: Value = serde_json::from_str(&content).context("invalid goose config json")?;
    let merged = merge_with_defaults(value);
    let raw: RawPluginConfig =
        serde_json::from_value(merged.clone()).context("invalid goose config structure")?;
    Ok(GoosePluginConfig {
        name: raw.name,
        description: raw.description,
        capabilities: raw.capabilities,
        settings: raw.settings,
        config_schema: CONFIG_SCHEMA.clone(),
    })
}

fn merge_with_defaults(value: Value) -> Value {
    match (value, DEFAULT_CONFIG.clone()) {
        (Value::Object(mut current), Value::Object(defaults)) => {
            for (key, default_value) in defaults {
                let merged_value = if let Some(existing) = current.remove(&key) {
                    merge_with_defaults_inner(existing, default_value)
                } else {
                    default_value
                };
                current.insert(key, merged_value);
            }
            Value::Object(current)
        }
        (value, _) => value,
    }
}

fn merge_with_defaults_inner(value: Value, default_value: Value) -> Value {
    match (value, default_value) {
        (Value::Object(mut current), Value::Object(defaults)) => {
            for (key, default_entry) in defaults {
                let merged = if let Some(existing) = current.remove(&key) {
                    merge_with_defaults_inner(existing, default_entry)
                } else {
                    default_entry
                };
                current.insert(key, merged);
            }
            Value::Object(current)
        }
        (Value::Array(current), Value::Array(_defaults)) if !current.is_empty() => {
            Value::Array(current)
        }
        (Value::Null, default_value) => default_value,
        (Value::String(s), Value::String(_)) if s.is_empty() => Value::String(s),
        (current, _) => current,
    }
}
