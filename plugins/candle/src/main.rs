use anyhow::{anyhow, Context, Result};
use axum::{extract::State, routing::get, routing::post, Json, Router};
use candle_core::{Device, Tensor};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    net::TcpListener,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use thiserror::Error;
use tokio::{fs, net::TcpStream, signal, task::JoinHandle, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use url::Url;

#[derive(Debug, Deserialize, Clone)]
struct CandleConfig {
    models_path: PathBuf,
    default_model: Option<String>,
    #[serde(default = "default_interval")]
    telemetry_interval: u64,
}

fn default_interval() -> u64 {
    10
}

impl Default for CandleConfig {
    fn default() -> Self {
        Self {
            models_path: PathBuf::from("./models"),
            default_model: None,
            telemetry_interval: default_interval(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct LinearModelFile {
    input_dim: usize,
    output_dim: usize,
    weights: Vec<f32>,
    bias: Vec<f32>,
    #[serde(default)]
    metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct ModelMetadata {
    id: String,
    input_dim: usize,
    output_dim: usize,
    checksum: String,
    path: PathBuf,
    loaded_at: String,
    metadata: Option<Value>,
}

#[derive(Clone)]
struct LoadedModel {
    weight: Tensor,
    bias: Tensor,
    input_dim: usize,
    output_dim: usize,
    checksum: String,
    path: PathBuf,
    metadata: Option<Value>,
    loaded_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
struct AppState {
    plugin_name: String,
    port: u16,
    config: CandleConfig,
    device: Device,
    models: Arc<RwLock<HashMap<String, LoadedModel>>>,
    base_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LoadModelRequest {
    id: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct RunModelRequest {
    id: String,
    inputs: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize)]
struct RunModelResponse {
    id: String,
    outputs: Vec<Vec<f32>>,
    execution_ms: u128,
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    models: Vec<ModelMetadata>,
}

#[derive(Debug, Error)]
enum CandleError {
    #[error("model {0} not loaded")]
    MissingModel(String),
    #[error("dimension mismatch: expected {expected} columns, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum BusMessage {
    #[serde(rename = "register")]
    Register {
        plugin: String,
        port: serde_json::Value,
        capabilities: Vec<String>,
        meta: Value,
    },
    #[serde(rename = "log")]
    Log {
        plugin: String,
        level: String,
        message: String,
        timestamp: String,
    },
    #[serde(rename = "health")]
    Health {
        plugin: String,
        status: String,
        detail: Option<String>,
    },
    #[serde(rename = "request")]
    Request {
        requestId: String,
        capability: String,
        payload: Value,
        token: Option<String>,
    },
    #[serde(rename = "response")]
    Response {
        requestId: String,
        success: bool,
        data: Option<Value>,
        error: Option<String>,
    },
    #[serde(rename = "telemetry")]
    Telemetry {
        plugin: String,
        cpu: f32,
        mem_bytes: u64,
        models_loaded: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let plugin_name = std::env::var("BKG_PLUGIN_NAME").unwrap_or_else(|_| "candle".into());
    let base_dir = std::env::current_dir()?;
    let config = load_config(base_dir.join("config.json"))
        .await
        .unwrap_or_default();

    let listener = TcpListener::bind("0.0.0.0:0").context("allocate candle port")?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let state = AppState {
        plugin_name: plugin_name.clone(),
        port,
        config,
        device: Device::Cpu,
        models: Arc::new(RwLock::new(HashMap::new())),
        base_dir,
    };

    fs::create_dir_all(state.models_path()).await?;

    if let Some(default_model) = state.config.default_model.clone() {
        let _ = state
            .load_model(LoadModelRequest {
                id: default_model.clone(),
                path: default_model,
            })
            .await
            .map_err(|err| warn!("failed to load default model: {err}"));
    }

    let mut bus_handle = tokio::spawn(spawn_bus(state.clone()));
    let mut http_handle = spawn_http_server(state.clone()).await?;

    tokio::select! {
        res = &mut bus_handle => {
            match res {
                Ok(Ok(())) => info!("plugin" = %state.plugin_name, "bus task completed"),
                Ok(Err(err)) => error!("error" = %err, "bus task failed"),
                Err(err) => error!("error" = %err, "bus join error"),
            }
        }
        res = &mut http_handle => {
            if let Err(err) = res {
                error!("error" = %err, "http server terminated unexpectedly");
            }
        }
        _ = signal::ctrl_c() => {
            info!("plugin" = %state.plugin_name, "received shutdown signal");
        }
    }

    bus_handle.abort();
    http_handle.abort();
    let _ = bus_handle.await;
    let _ = http_handle.await;

    Ok(())
}

async fn load_config(path: PathBuf) -> Result<CandleConfig> {
    if !path.exists() {
        return Ok(CandleConfig::default());
    }
    let bytes = fs::read(&path).await?;
    let config = serde_json::from_slice::<CandleConfig>(&bytes)?;
    Ok(config)
}

impl AppState {
    fn models_path(&self) -> PathBuf {
        if self.config.models_path.is_absolute() {
            self.config.models_path.clone()
        } else {
            self.base_dir.join(&self.config.models_path)
        }
    }

    fn resolve_model_path(&self, requested: &str) -> Result<PathBuf> {
        let mut path = PathBuf::from(requested);
        if !path.is_absolute() {
            path = self.models_path().join(requested);
        }
        let canon = path.canonicalize().context("canonicalize model path")?;
        let base = self
            .models_path()
            .canonicalize()
            .unwrap_or_else(|_| self.models_path());
        if !canon.starts_with(&base) {
            return Err(anyhow!("model path {requested} escapes models directory"));
        }
        Ok(canon)
    }

    async fn load_model(&self, request: LoadModelRequest) -> Result<ModelMetadata> {
        let path = self.resolve_model_path(&request.path)?;
        let payload = fs::read(&path).await?;
        let file = serde_json::from_slice::<LinearModelFile>(&payload)?;
        if file.weights.len() != file.input_dim * file.output_dim {
            return Err(anyhow!(
                "weights length {} does not match shape {}x{}",
                file.weights.len(),
                file.output_dim,
                file.input_dim
            ));
        }
        if file.bias.len() != file.output_dim {
            return Err(anyhow!(
                "bias length {} does not match output dim {}",
                file.bias.len(),
                file.output_dim
            ));
        }
        let checksum = hex::encode(Sha256::digest(&payload));
        let weight = Tensor::from_vec(
            file.weights.clone(),
            (file.output_dim, file.input_dim),
            &self.device,
        )?;
        let bias = Tensor::from_vec(file.bias.clone(), file.output_dim, &self.device)?;
        let loaded = LoadedModel {
            weight,
            bias,
            input_dim: file.input_dim,
            output_dim: file.output_dim,
            checksum: checksum.clone(),
            path: path.clone(),
            metadata: file.metadata.clone(),
            loaded_at: Utc::now(),
        };
        self.models.write().insert(request.id.clone(), loaded);
        let metadata = ModelMetadata {
            id: request.id,
            input_dim: file.input_dim,
            output_dim: file.output_dim,
            checksum,
            path,
            loaded_at: Utc::now().to_rfc3339(),
            metadata: file.metadata,
        };
        Ok(metadata)
    }

    fn run_model(&self, request: RunModelRequest) -> Result<RunModelResponse> {
        let models = self.models.read();
        let model = models
            .get(&request.id)
            .ok_or_else(|| CandleError::MissingModel(request.id.clone()))?;
        if request.inputs.is_empty() {
            return Err(anyhow!("inputs may not be empty"));
        }
        let cols = request.inputs[0].len();
        if cols != model.input_dim {
            return Err(CandleError::DimensionMismatch {
                expected: model.input_dim,
                actual: cols,
            }
            .into());
        }
        let start = Instant::now();
        let flat: Vec<f32> = request
            .inputs
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect();
        let input = Tensor::from_vec(flat, (request.inputs.len(), model.input_dim), &self.device)?;
        let logits = input.matmul(&model.weight.t())?.add(&model.bias)?;
        let outputs = logits.to_vec2::<f32>()?;
        let elapsed = start.elapsed().as_millis();
        Ok(RunModelResponse {
            id: request.id,
            outputs,
            execution_ms: elapsed,
        })
    }

    fn stats(&self) -> StatsResponse {
        let models = self.models.read();
        let entries = models
            .iter()
            .map(|(id, model)| ModelMetadata {
                id: id.clone(),
                input_dim: model.input_dim,
                output_dim: model.output_dim,
                checksum: model.checksum.clone(),
                path: model.path.clone(),
                loaded_at: model.loaded_at.to_rfc3339(),
                metadata: model.metadata.clone(),
            })
            .collect();
        StatsResponse { models: entries }
    }
}

async fn spawn_http_server(state: AppState) -> Result<JoinHandle<()>> {
    let address = format!("0.0.0.0:{}", state.port);
    let router = Router::new()
        .route("/health", get(health_handler))
        .route("/models", get(list_models))
        .route("/models/:id/run", post(run_model))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    Ok(handle)
}

async fn health_handler(State(state): State<AppState>) -> Json<Value> {
    let models = state.models.read();
    Json(serde_json::json!({
        "status": "ok",
        "loaded_models": models.len(),
    }))
}

async fn list_models(State(state): State<AppState>) -> Json<StatsResponse> {
    Json(state.stats())
}

async fn run_model(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<RunModelResponse>, axum::http::StatusCode> {
    let inputs = body
        .get("inputs")
        .ok_or(axum::http::StatusCode::BAD_REQUEST)?
        .clone();
    let inputs: Vec<Vec<f32>> =
        serde_json::from_value(inputs).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let result = state
        .run_model(RunModelRequest { id, inputs })
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(result))
}

async fn spawn_bus(state: AppState) -> Result<()> {
    let port = std::env::var("BKG_PLUGIN_BUS_PORT")
        .context("missing BKG_PLUGIN_BUS_PORT")?
        .parse::<u16>()?;
    let url = Url::parse(&format!("ws://127.0.0.1:{port}"))?;
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    let register = BusMessage::Register {
        plugin: state.plugin_name.clone(),
        port: serde_json::json!(state.port),
        capabilities: vec![
            "candle.model.load".into(),
            "candle.model.run".into(),
            "candle.stats".into(),
        ],
        meta: serde_json::json!({ "models_path": state.models_path() }),
    };
    write
        .send(Message::text(serde_json::to_string(&register)?))
        .await?;

    let mut telemetry_state = state.clone();
    let mut telemetry_write = write.clone();
    tokio::spawn(async move {
        loop {
            let snapshot = capture_telemetry(&telemetry_state);
            let message = BusMessage::Telemetry {
                plugin: telemetry_state.plugin_name.clone(),
                cpu: snapshot.cpu,
                mem_bytes: snapshot.memory,
                models_loaded: snapshot.models,
            };
            if telemetry_write
                .send(Message::text(serde_json::to_string(&message).unwrap()))
                .await
                .is_err()
            {
                break;
            }
            sleep(Duration::from_secs(
                telemetry_state.config.telemetry_interval,
            ))
            .await;
        }
    });

    let mut heartbeat_write = write.clone();
    let plugin = state.plugin_name.clone();
    tokio::spawn(async move {
        loop {
            let message = BusMessage::Health {
                plugin: plugin.clone(),
                status: "up".into(),
                detail: None,
            };
            if heartbeat_write
                .send(Message::text(serde_json::to_string(&message).unwrap()))
                .await
                .is_err()
            {
                break;
            }
            sleep(Duration::from_secs(10)).await;
        }
    });

    send_log(&state, "info", "candle plugin registered", &mut write).await?;

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if let Ok(bus_message) = serde_json::from_str::<BusMessage>(&text) {
                    handle_bus_message(&state, bus_message, &mut write).await?;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(err) => {
                error!("error" = %err, "bus error");
                break;
            }
        }
    }

    Ok(())
}

struct TelemetrySnapshot {
    cpu: f32,
    memory: u64,
    models: usize,
}

fn capture_telemetry(state: &AppState) -> TelemetrySnapshot {
    let mut system = System::new_all();
    system.refresh_processes();
    let pid = std::process::id();
    let cpu = system
        .process(sysinfo::Pid::from_u32(pid))
        .map(|process| process.cpu_usage())
        .unwrap_or_default();
    let memory = system
        .process(sysinfo::Pid::from_u32(pid))
        .map(|process| process.memory() * 1024)
        .unwrap_or_default();
    let models = state.models.read().len();
    TelemetrySnapshot {
        cpu,
        memory,
        models,
    }
}

async fn handle_bus_message(
    state: &AppState,
    message: BusMessage,
    write: &mut tokio_tungstenite::split::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<()> {
    if let BusMessage::Request {
        requestId,
        capability,
        payload,
        ..
    } = message
    {
        let response = match capability.as_str() {
            "candle.model.load" => match serde_json::from_value::<LoadModelRequest>(payload) {
                Ok(req) => match state.load_model(req).await {
                    Ok(meta) => {
                        let message = format!("loaded model {}", meta.id);
                        send_log(state, "info", &message, write).await.ok();
                        BusMessage::Response {
                            requestId,
                            success: true,
                            data: Some(serde_json::to_value(meta)?),
                            error: None,
                        }
                    }
                    Err(err) => {
                        warn!("error" = %err, "failed to load model");
                        BusMessage::Response {
                            requestId,
                            success: false,
                            data: None,
                            error: Some(err.to_string()),
                        }
                    }
                },
                Err(err) => BusMessage::Response {
                    requestId,
                    success: false,
                    data: None,
                    error: Some(format!("invalid payload: {err}")),
                },
            },
            "candle.model.run" => match serde_json::from_value::<RunModelRequest>(payload) {
                Ok(req) => match state.run_model(req) {
                    Ok(result) => {
                        let message =
                            format!("executed model {} in {}ms", result.id, result.execution_ms);
                        send_log(state, "debug", &message, write).await.ok();
                        BusMessage::Response {
                            requestId,
                            success: true,
                            data: Some(serde_json::to_value(result)?),
                            error: None,
                        }
                    }
                    Err(err) => BusMessage::Response {
                        requestId,
                        success: false,
                        data: None,
                        error: Some(err.to_string()),
                    },
                },
                Err(err) => BusMessage::Response {
                    requestId,
                    success: false,
                    data: None,
                    error: Some(format!("invalid payload: {err}")),
                },
            },
            "candle.stats" => {
                let stats = state.stats();
                BusMessage::Response {
                    requestId,
                    success: true,
                    data: Some(serde_json::to_value(stats)?),
                    error: None,
                }
            }
            other => BusMessage::Response {
                requestId,
                success: false,
                data: None,
                error: Some(format!("unsupported capability {other}")),
            },
        };
        write
            .send(Message::text(serde_json::to_string(&response)?))
            .await?;
    }
    Ok(())
}

async fn send_log(
    state: &AppState,
    level: &str,
    message: &str,
    write: &mut tokio_tungstenite::split::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
) -> Result<()> {
    let log = BusMessage::Log {
        plugin: state.plugin_name.clone(),
        level: level.into(),
        message: message.into(),
        timestamp: Utc::now().to_rfc3339(),
    };
    write
        .send(Message::text(serde_json::to_string(&log)?))
        .await?;
    Ok(())
}
