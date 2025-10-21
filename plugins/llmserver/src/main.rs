use anyhow::{Context, Result};
use axum::{extract::State, routing::get, routing::post, Json, Router};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    net::TcpListener,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{net::TcpStream, task::JoinHandle, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};
use url::Url;

#[derive(Clone)]
struct AppState {
    port: u16,
    plugin_name: String,
    prompt_template: String,
    chat_model: String,
    embedding_model: String,
    history: Arc<RwLock<VecDeque<ChatMessage>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionRequest {
    messages: Vec<ChatMessage>,
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionChoice {
    index: usize,
    message: ChatMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct EmbeddingVector {
    index: usize,
    embedding: Vec<f32>,
    object: String,
}

#[derive(Debug, Serialize)]
struct EmbeddingResponse {
    object: String,
    data: Vec<EmbeddingVector>,
    model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum BusMessage {
    #[serde(rename = "register")]
    Register {
        plugin: String,
        port: serde_json::Value,
        capabilities: Vec<String>,
        meta: serde_json::Value,
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
        payload: serde_json::Value,
        token: Option<String>,
    },
    #[serde(rename = "response")]
    Response {
        requestId: String,
        success: bool,
        data: Option<serde_json::Value>,
        error: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let plugin_name = std::env::var("BKG_PLUGIN_NAME").unwrap_or_else(|_| "llmserver".to_string());
    let prompt_template = std::env::var("PROMPT_TEMPLATE").unwrap_or_else(|_| "assistant".into());
    let chat_model = std::env::var("CHAT_MODEL_FILE").unwrap_or_else(|_| "synthetic-chat".into());
    let embedding_model = std::env::var("EMBEDDING_MODEL_FILE").unwrap_or_else(|_| "synthetic-embed".into());
    let listener = TcpListener::bind("0.0.0.0:0").context("binding llmserver port")?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let state = AppState {
        port,
        plugin_name: plugin_name.clone(),
        prompt_template,
        chat_model: chat_model.clone(),
        embedding_model: embedding_model.clone(),
        history: Arc::new(RwLock::new(VecDeque::with_capacity(32))),
    };

    let bus_handle = tokio::spawn(spawn_bus(state.clone()));
    let http_handle = spawn_http_server(state.clone()).await?;

    info!("plugin" = %plugin_name, port, chat_model, embedding_model, "LLM server started");
    let (bus_result, http_result) = tokio::join!(bus_handle, http_handle);
    if let Err(err) = bus_result {
        error!("error" = %err, "bus task failed");
    }
    if let Err(err) = http_result {
        error!("error" = %err, "http server task failed");
    }
    Ok(())
}

async fn spawn_http_server(state: AppState) -> Result<JoinHandle<()>> {
    let address = format!("0.0.0.0:{}", state.port);
    let router = Router::new()
        .route("/v1/chat/completions", post(chat_handler))
        .route("/v1/embeddings", post(embedding_handler))
        .route("/health", get(health_handler))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    Ok(handle)
}

async fn chat_handler(State(state): State<AppState>, Json(request): Json<ChatCompletionRequest>) -> Json<ChatCompletionResponse> {
    let response = generate_chat_response(&state, &request.messages);
    Json(response)
}

async fn embedding_handler(
    State(state): State<AppState>,
    Json(request): Json<EmbeddingRequest>,
) -> Json<EmbeddingResponse> {
    let response = generate_embeddings(&state, &request.input);
    Json(response)
}

async fn health_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "model": state.chat_model,
        "embedding_model": state.embedding_model,
    }))
}

fn generate_chat_response(state: &AppState, messages: &[ChatMessage]) -> ChatCompletionResponse {
    let mut history = state.history.write();
    for message in messages {
        history.push_back(message.clone());
        if history.len() > 32 {
            history.pop_front();
        }
    }
    let reply = synthesize_reply(messages, &state.prompt_template);
    history.push_back(ChatMessage {
        role: "assistant".into(),
        content: reply.clone(),
    });
    ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".into(),
        created: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        model: state.chat_model.clone(),
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".into(),
                content: reply,
            },
            finish_reason: "stop".into(),
        }],
    }
}

fn synthesize_reply(messages: &[ChatMessage], prompt_template: &str) -> String {
    let mut rng = StdRng::seed_from_u64(42);
    let user_message = messages
        .iter()
        .rev()
        .find(|msg| msg.role == "user")
        .map(|msg| msg.content.clone())
        .unwrap_or_else(|| "How can I assist you today?".into());
    let mut summary = user_message
        .split_whitespace()
        .take(64)
        .collect::<Vec<_>>()
        .join(" ");
    if summary.len() < user_message.len() {
        summary.push_str(" …");
    }
    let variations = [
        "Certainly",
        "Absolutely",
        "Understood",
        "Gladly",
        "Of course",
    ];
    let prefix = variations[rng.gen_range(0..variations.len())];
    format!(
        "{}: {}\n\n{}",
        prompt_template,
        prefix,
        summary
    )
}

fn generate_embeddings(state: &AppState, input: &[String]) -> EmbeddingResponse {
    let data = input
        .iter()
        .enumerate()
        .map(|(index, value)| EmbeddingVector {
            index,
            embedding: deterministic_vector(value),
            object: "embedding".into(),
        })
        .collect();
    EmbeddingResponse {
        object: "list".into(),
        data,
        model: state.embedding_model.clone(),
    }
}

fn deterministic_vector(value: &str) -> Vec<f32> {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    digest
        .chunks(4)
        .map(|chunk| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(chunk);
            let number = u32::from_be_bytes(bytes);
            (number as f32) / (u32::MAX as f32)
        })
        .collect()
}

async fn spawn_bus(state: AppState) {
    loop {
        match run_bus_connection(state.clone()).await {
            Ok(_) => break,
            Err(error) => {
                error!("error" = %error, "bus connection failed, retrying");
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}

async fn run_bus_connection(state: AppState) -> Result<()> {
    let port = std::env::var("BKG_PLUGIN_BUS_PORT")
        .context("BKG_PLUGIN_BUS_PORT is required")?
        .parse::<u16>()
        .context("invalid BKG_PLUGIN_BUS_PORT")?;
    let url = Url::parse(&format!("ws://127.0.0.1:{port}"))?;
    let (stream, _) = connect_async(url).await.context("connecting to plugin bus")?;
    let (mut write, mut read) = stream.split();

    let register = BusMessage::Register {
        plugin: state.plugin_name.clone(),
        port: json!(state.port),
        capabilities: vec!["llm.chat".into(), "llm.embed".into()],
        meta: json!({
            "chat_model": state.chat_model,
            "embedding_model": state.embedding_model,
        }),
    };
    write
        .send(Message::text(serde_json::to_string(&register)?))
        .await?;

    let health_state = state.clone();
    let mut heartbeat_writer = write.clone();
    tokio::spawn(async move {
        loop {
            let message = BusMessage::Health {
                plugin: health_state.plugin_name.clone(),
                status: "up".into(),
                detail: None,
            };
            if heartbeat_writer
                .send(Message::text(serde_json::to_string(&message).unwrap()))
                .await
                .is_err()
            {
                break;
            }
            sleep(Duration::from_secs(10)).await;
        }
    });

    send_log(&state, "info", &format!("registered on port {}", state.port), &mut write).await?;

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if let Ok(bus_message) = serde_json::from_str::<BusMessage>(&text) {
                    handle_bus_message(&state, bus_message, &mut write).await?;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(error) => {
                error!("error" = %error, "bus stream error");
                break;
            }
        }
    }
    Ok(())
}

async fn handle_bus_message(state: &AppState, message: BusMessage, write: &mut tokio_tungstenite::split::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>) -> Result<()> {
    match message {
        BusMessage::Request {
            requestId,
            capability,
            payload,
            ..
        } => {
            let response = match capability.as_str() {
                "llm.chat" => {
                    let request: ChatCompletionRequest = serde_json::from_value(payload)?;
                    let result = generate_chat_response(state, &request.messages);
                    BusMessage::Response {
                        requestId,
                        success: true,
                        data: Some(serde_json::to_value(result)?),
                        error: None,
                    }
                }
                "llm.embed" => {
                    let request: EmbeddingRequest = serde_json::from_value(payload)?;
                    let result = generate_embeddings(state, &request.input);
                    BusMessage::Response {
                        requestId,
                        success: true,
                        data: Some(serde_json::to_value(result)?),
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
        BusMessage::Log { .. }
        | BusMessage::Response { .. }
        | BusMessage::Health { .. }
        | BusMessage::Register { .. } => {}
    }
    Ok(())
}

async fn send_log(state: &AppState, level: &str, message: &str, write: &mut tokio_tungstenite::split::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>) -> Result<()> {
    let log_message = BusMessage::Log {
        plugin: state.plugin_name.clone(),
        level: level.into(),
        message: message.into(),
        timestamp: Utc::now().to_rfc3339(),
    };
    write
        .send(Message::text(serde_json::to_string(&log_message)?))
        .await?;
    Ok(())
}
