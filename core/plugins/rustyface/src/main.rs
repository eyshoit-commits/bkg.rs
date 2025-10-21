use anyhow::{anyhow, Context, Result};
use axum::{extract::State, routing::get, routing::post, Json, Router};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use image::{imageops::FilterType, DynamicImage, ImageBuffer, Luma};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, net::TcpListener, path::PathBuf, sync::Arc, time::Duration};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use thiserror::Error;
use tokio::{fs, net::TcpStream, signal, task::JoinHandle, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
struct RustyFaceConfig {
    datasets_path: PathBuf,
    #[serde(default = "default_dataset_id")]
    default_dataset: String,
    #[serde(default = "default_interval")]
    telemetry_interval: u64,
}

fn default_dataset_id() -> String {
    "default".to_string()
}

fn default_interval() -> u64 {
    10
}

impl Default for RustyFaceConfig {
    fn default() -> Self {
        Self {
            datasets_path: PathBuf::from("./datasets"),
            default_dataset: default_dataset_id(),
            telemetry_interval: default_interval(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct EncodeResponse {
    id: String,
    dataset: Option<String>,
    embedding: Vec<f32>,
    dimensions: usize,
    checksum: String,
}

#[derive(Debug, Clone, Serialize)]
struct SearchResult {
    id: String,
    score: f32,
    metadata: Option<Value>,
    last_seen: String,
}

#[derive(Debug, Clone, Serialize)]
struct SearchResponse {
    dataset: String,
    query_embedding: Vec<f32>,
    results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize)]
struct DatasetSummary {
    id: String,
    entries: usize,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct EncodeRequest {
    image_base64: String,
    dataset: Option<String>,
    entry_id: Option<String>,
    metadata: Option<Value>,
    #[serde(default = "default_true")]
    persist: bool,
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    dataset: Option<String>,
    top_k: Option<usize>,
    image_base64: Option<String>,
    embedding: Option<Vec<f32>>,
    threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct DatasetManageRequest {
    action: String,
    dataset: Option<String>,
    entry_id: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone)]
struct FaceEntry {
    id: String,
    embedding: Vec<f32>,
    metadata: Option<Value>,
    last_seen: DateTime<Utc>,
}

#[derive(Clone)]
struct FaceDataset {
    id: String,
    entries: HashMap<String, FaceEntry>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    path: PathBuf,
}

#[derive(Clone)]
struct AppState {
    plugin_name: String,
    port: u16,
    config: RustyFaceConfig,
    datasets: Arc<RwLock<HashMap<String, FaceDataset>>>,
    base_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedDataset {
    id: String,
    created_at: String,
    updated_at: String,
    entries: Vec<PersistedEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedEntry {
    id: String,
    embedding: Vec<f32>,
    metadata: Option<Value>,
    last_seen: String,
}

#[derive(Debug, Serialize)]
struct ManageResponse {
    datasets: Vec<DatasetSummary>,
}

#[derive(Debug, Error)]
enum RustyFaceError {
    #[error("dataset {0} not found")]
    MissingDataset(String),
    #[error("entry {entry} not found in dataset {dataset}")]
    MissingEntry { dataset: String, entry: String },
    #[error("embedding length mismatch: expected {expected}, got {actual}")]
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
        datasets: usize,
        entries: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let plugin_name = std::env::var("BKG_PLUGIN_NAME").unwrap_or_else(|_| "rustyface".into());
    let base_dir = std::env::current_dir()?;
    let config = load_config(base_dir.join("config.json"))
        .await
        .unwrap_or_default();

    let listener = TcpListener::bind("0.0.0.0:0").context("allocate rustyface port")?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let state = AppState {
        plugin_name: plugin_name.clone(),
        port,
        config,
        datasets: Arc::new(RwLock::new(HashMap::new())),
        base_dir,
    };

    fs::create_dir_all(state.datasets_path()).await?;
    load_existing_datasets(&state).await?;
    ensure_default_dataset(&state).await?;

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

async fn load_config(path: PathBuf) -> Result<RustyFaceConfig> {
    if !path.exists() {
        return Ok(RustyFaceConfig::default());
    }
    let bytes = fs::read(&path).await?;
    let config = serde_json::from_slice::<RustyFaceConfig>(&bytes)?;
    Ok(config)
}

impl AppState {
    fn datasets_path(&self) -> PathBuf {
        if self.config.datasets_path.is_absolute() {
            self.config.datasets_path.clone()
        } else {
            self.base_dir.join(&self.config.datasets_path)
        }
    }

    fn dataset_path(&self, dataset: &str) -> PathBuf {
        let mut path = self.datasets_path();
        path.push(format!("{}.json", dataset));
        path
    }

    async fn encode(&self, request: EncodeRequest) -> Result<EncodeResponse> {
        let bytes = BASE64
            .decode(request.image_base64.as_bytes())
            .context("invalid base64 payload")?;
        let embedding = compute_embedding(&bytes)?;
        let checksum = sha256(&embedding);
        let dataset_id = request
            .dataset
            .unwrap_or_else(|| self.config.default_dataset.clone());
        let entry_id = request
            .entry_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        if request.persist {
            self.persist_entry(
                &dataset_id,
                FaceEntry {
                    id: entry_id.clone(),
                    embedding: embedding.clone(),
                    metadata: request.metadata.clone(),
                    last_seen: Utc::now(),
                },
            )
            .await?;
        }

        Ok(EncodeResponse {
            id: entry_id,
            dataset: if request.persist {
                Some(dataset_id)
            } else {
                None
            },
            embedding,
            dimensions: embedding.len(),
            checksum,
        })
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let dataset_id = request
            .dataset
            .unwrap_or_else(|| self.config.default_dataset.clone());
        let dataset = {
            let datasets = self.datasets.read();
            datasets
                .get(&dataset_id)
                .cloned()
                .ok_or_else(|| RustyFaceError::MissingDataset(dataset_id.clone()))?
        };
        let query_embedding = if let Some(values) = request.embedding {
            validate_embedding(&dataset, &values)?;
            normalize_embedding(values)
        } else if let Some(base64) = request.image_base64 {
            let bytes = BASE64
                .decode(base64.as_bytes())
                .context("invalid base64 payload")?;
            compute_embedding(&bytes)?
        } else {
            return Err(anyhow!("either embedding or image_base64 must be provided"));
        };

        let mut scored: Vec<SearchResult> = dataset
            .entries
            .values()
            .map(|entry| {
                let score = cosine_similarity(&query_embedding, &entry.embedding);
                SearchResult {
                    id: entry.id.clone(),
                    score,
                    metadata: entry.metadata.clone(),
                    last_seen: entry.last_seen.to_rfc3339(),
                }
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_k = request.top_k.unwrap_or(5);
        let threshold = request.threshold.unwrap_or(0.0);
        scored.truncate(top_k);
        scored.retain(|entry| entry.score >= threshold);

        Ok(SearchResponse {
            dataset: dataset_id,
            query_embedding,
            results: scored,
        })
    }

    async fn manage(&self, request: DatasetManageRequest) -> Result<ManageResponse> {
        match request.action.as_str() {
            "list" => {
                let datasets = self.datasets.read();
                let summaries = datasets
                    .values()
                    .map(|dataset| DatasetSummary {
                        id: dataset.id.clone(),
                        entries: dataset.entries.len(),
                        created_at: dataset.created_at.to_rfc3339(),
                        updated_at: dataset.updated_at.to_rfc3339(),
                    })
                    .collect();
                Ok(ManageResponse {
                    datasets: summaries,
                })
            }
            "remove-entry" => {
                let dataset_id = request
                    .dataset
                    .clone()
                    .ok_or_else(|| anyhow!("dataset field required"))?;
                let entry_id = request
                    .entry_id
                    .clone()
                    .ok_or_else(|| anyhow!("entry_id field required"))?;
                self.remove_entry(&dataset_id, &entry_id).await?;
                Ok(self.list_datasets())
            }
            "delete-dataset" => {
                let dataset_id = request
                    .dataset
                    .clone()
                    .ok_or_else(|| anyhow!("dataset field required"))?;
                self.delete_dataset(&dataset_id).await?;
                Ok(self.list_datasets())
            }
            "create" => {
                let dataset_id = request
                    .dataset
                    .clone()
                    .unwrap_or_else(|| Uuid::new_v4().to_string());
                self.create_dataset(&dataset_id).await?;
                Ok(self.list_datasets())
            }
            other => Err(anyhow!("unsupported manage action {other}")),
        }
    }

    fn list_datasets(&self) -> ManageResponse {
        let datasets = self.datasets.read();
        let summaries = datasets
            .values()
            .map(|dataset| DatasetSummary {
                id: dataset.id.clone(),
                entries: dataset.entries.len(),
                created_at: dataset.created_at.to_rfc3339(),
                updated_at: dataset.updated_at.to_rfc3339(),
            })
            .collect();
        ManageResponse {
            datasets: summaries,
        }
    }

    async fn persist_entry(&self, dataset_id: &str, entry: FaceEntry) -> Result<()> {
        let mut datasets = self.datasets.write();
        let dataset = datasets
            .get_mut(dataset_id)
            .ok_or_else(|| RustyFaceError::MissingDataset(dataset_id.to_string()))?;
        dataset.updated_at = Utc::now();
        dataset.entries.insert(entry.id.clone(), entry.clone());
        drop(datasets);
        save_dataset(dataset).await
    }

    async fn remove_entry(&self, dataset_id: &str, entry_id: &str) -> Result<()> {
        let mut datasets = self.datasets.write();
        let dataset = datasets
            .get_mut(dataset_id)
            .ok_or_else(|| RustyFaceError::MissingDataset(dataset_id.to_string()))?;
        if dataset.entries.remove(entry_id).is_none() {
            return Err(RustyFaceError::MissingEntry {
                dataset: dataset_id.to_string(),
                entry: entry_id.to_string(),
            }
            .into());
        }
        dataset.updated_at = Utc::now();
        drop(datasets);
        save_dataset(dataset).await
    }

    async fn delete_dataset(&self, dataset_id: &str) -> Result<()> {
        let mut datasets = self.datasets.write();
        if let Some(dataset) = datasets.remove(dataset_id) {
            drop(datasets);
            if dataset.path.exists() {
                fs::remove_file(&dataset.path).await?;
            }
            Ok(())
        } else {
            Err(RustyFaceError::MissingDataset(dataset_id.to_string()).into())
        }
    }

    async fn create_dataset(&self, dataset_id: &str) -> Result<()> {
        let mut datasets = self.datasets.write();
        if datasets.contains_key(dataset_id) {
            return Ok(());
        }
        let dataset = FaceDataset {
            id: dataset_id.to_string(),
            entries: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            path: self.dataset_path(dataset_id),
        };
        save_dataset(&dataset).await?;
        datasets.insert(dataset.id.clone(), dataset);
        Ok(())
    }
}

async fn load_existing_datasets(state: &AppState) -> Result<()> {
    let mut dir = tokio::fs::read_dir(state.datasets_path()).await?;
    while let Some(entry) = dir.next_entry().await? {
        if entry.file_type().await?.is_file() {
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(entry.path()).await?;
            let persisted = serde_json::from_slice::<PersistedDataset>(&bytes)?;
            let dataset = FaceDataset {
                id: persisted.id.clone(),
                entries: persisted
                    .entries
                    .into_iter()
                    .map(|entry| {
                        let parsed = DateTime::parse_from_rfc3339(&entry.last_seen)
                            .unwrap_or_else(|_| Utc::now())
                            .with_timezone(&Utc);
                        (
                            entry.id.clone(),
                            FaceEntry {
                                id: entry.id,
                                embedding: entry.embedding,
                                metadata: entry.metadata,
                                last_seen: parsed,
                            },
                        )
                    })
                    .collect(),
                created_at: DateTime::parse_from_rfc3339(&persisted.created_at)
                    .unwrap_or_else(|_| Utc::now())
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&persisted.updated_at)
                    .unwrap_or_else(|_| Utc::now())
                    .with_timezone(&Utc),
                path: entry.path(),
            };
            state.datasets.write().insert(dataset.id.clone(), dataset);
        }
    }
    Ok(())
}

async fn ensure_default_dataset(state: &AppState) -> Result<()> {
    if !state
        .datasets
        .read()
        .contains_key(&state.config.default_dataset)
    {
        state.create_dataset(&state.config.default_dataset).await?;
    }
    Ok(())
}

async fn save_dataset(dataset: &FaceDataset) -> Result<()> {
    let persisted = PersistedDataset {
        id: dataset.id.clone(),
        created_at: dataset.created_at.to_rfc3339(),
        updated_at: dataset.updated_at.to_rfc3339(),
        entries: dataset
            .entries
            .values()
            .map(|entry| PersistedEntry {
                id: entry.id.clone(),
                embedding: entry.embedding.clone(),
                metadata: entry.metadata.clone(),
                last_seen: entry.last_seen.to_rfc3339(),
            })
            .collect(),
    };
    let tmp_path = dataset.path.with_extension("json.tmp");
    let payload = serde_json::to_vec_pretty(&persisted)?;
    fs::write(&tmp_path, &payload).await?;
    fs::rename(&tmp_path, &dataset.path).await?;
    Ok(())
}

fn compute_embedding(bytes: &[u8]) -> Result<Vec<f32>> {
    let image = image::load_from_memory(bytes)?;
    let normalized = preprocess_image(&image);
    Ok(normalized)
}

fn preprocess_image(image: &DynamicImage) -> Vec<f32> {
    let grayscale: ImageBuffer<Luma<u8>, Vec<u8>> = image.to_luma8();
    let resized = image::imageops::resize(&grayscale, 32, 32, FilterType::CatmullRom);
    let mut values: Vec<f32> = resized
        .pixels()
        .map(|pixel| pixel[0] as f32 / 255.0)
        .collect();
    normalize_inplace(&mut values);
    values
}

fn normalize_inplace(values: &mut [f32]) {
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in values.iter_mut() {
            *value /= norm;
        }
    }
}

fn normalize_embedding(mut values: Vec<f32>) -> Vec<f32> {
    normalize_inplace(&mut values);
    values
}

fn validate_embedding(dataset: &FaceDataset, embedding: &[f32]) -> Result<()> {
    if let Some(entry) = dataset.entries.values().next() {
        if entry.embedding.len() != embedding.len() {
            return Err(RustyFaceError::DimensionMismatch {
                expected: entry.embedding.len(),
                actual: embedding.len(),
            }
            .into());
        }
    }
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    dot
}

fn sha256(values: &[f32]) -> String {
    let mut hasher = sha2::Sha256::new();
    for value in values {
        hasher.update(value.to_le_bytes());
    }
    hex::encode(hasher.finalize())
}

async fn spawn_http_server(state: AppState) -> Result<JoinHandle<()>> {
    let address = format!("0.0.0.0:{}", state.port);
    let router = Router::new()
        .route("/health", get(health_handler))
        .route("/datasets", get(list_datasets_handler))
        .route("/datasets/:id", get(dataset_detail_handler))
        .route("/datasets/:id/search", post(dataset_search_handler))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    Ok(handle)
}

async fn health_handler(State(state): State<AppState>) -> Json<Value> {
    let datasets = state.datasets.read();
    let total_entries: usize = datasets.values().map(|dataset| dataset.entries.len()).sum();
    Json(serde_json::json!({
        "status": "ok",
        "datasets": datasets.len(),
        "entries": total_entries,
    }))
}

async fn list_datasets_handler(State(state): State<AppState>) -> Json<ManageResponse> {
    Json(state.list_datasets())
}

async fn dataset_detail_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<DatasetSummary>, axum::http::StatusCode> {
    let datasets = state.datasets.read();
    let dataset = datasets.get(&id).ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(DatasetSummary {
        id: dataset.id.clone(),
        entries: dataset.entries.len(),
        created_at: dataset.created_at.to_rfc3339(),
        updated_at: dataset.updated_at.to_rfc3339(),
    }))
}

async fn dataset_search_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<SearchResponse>, axum::http::StatusCode> {
    let mut request: SearchRequest =
        serde_json::from_value(payload).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    request.dataset = Some(id);
    state
        .search(request)
        .await
        .map(Json)
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)
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
            "faces.encode".into(),
            "faces.search".into(),
            "dataset.manage".into(),
        ],
        meta: serde_json::json!({ "datasets_path": state.datasets_path(), "default_dataset": state.config.default_dataset }),
    };
    write
        .send(Message::text(serde_json::to_string(&register)?))
        .await?;

    let mut telemetry_write = write.clone();
    let telemetry_state = state.clone();
    tokio::spawn(async move {
        loop {
            let snapshot = capture_telemetry(&telemetry_state);
            let message = BusMessage::Telemetry {
                plugin: telemetry_state.plugin_name.clone(),
                cpu: snapshot.cpu,
                mem_bytes: snapshot.memory,
                datasets: snapshot.datasets,
                entries: snapshot.entries,
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

    send_log(&state, "info", "rustyface plugin registered", &mut write).await?;

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
    datasets: usize,
    entries: usize,
}

fn capture_telemetry(state: &AppState) -> TelemetrySnapshot {
    let mut system = System::new_all();
    system.refresh_processes();
    let pid = std::process::id();
    let process = system.process(sysinfo::Pid::from_u32(pid));
    let cpu = process.map(|p| p.cpu_usage()).unwrap_or_default();
    let memory = process.map(|p| p.memory() * 1024).unwrap_or_default();
    let datasets = state.datasets.read();
    let entries = datasets.values().map(|dataset| dataset.entries.len()).sum();
    TelemetrySnapshot {
        cpu,
        memory,
        datasets: datasets.len(),
        entries,
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
            "faces.encode" => match serde_json::from_value::<EncodeRequest>(payload) {
                Ok(req) => match state.encode(req).await {
                    Ok(result) => {
                        let log = format!("encoded face {}", result.id);
                        send_log(state, "info", &log, write).await.ok();
                        BusMessage::Response {
                            requestId,
                            success: true,
                            data: Some(serde_json::to_value(result)?),
                            error: None,
                        }
                    }
                    Err(err) => {
                        warn!("error" = %err, "encode failed");
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
            "faces.search" => match serde_json::from_value::<SearchRequest>(payload) {
                Ok(req) => match state.search(req).await {
                    Ok(result) => BusMessage::Response {
                        requestId,
                        success: true,
                        data: Some(serde_json::to_value(result)?),
                        error: None,
                    },
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
            "dataset.manage" => match serde_json::from_value::<DatasetManageRequest>(payload) {
                Ok(req) => match state.manage(req).await {
                    Ok(result) => BusMessage::Response {
                        requestId,
                        success: true,
                        data: Some(serde_json::to_value(result)?),
                        error: None,
                    },
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
