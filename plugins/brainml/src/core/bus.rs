use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::{
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream};
use tracing::{error, info, instrument};
use uuid::Uuid;

type WebSocketWriter =
    futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

#[derive(Debug, Error)]
pub enum BusError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("registration rejected: {0}")]
    Registration(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutgoingMessage {
    #[serde(rename = "register")]
    Register {
        plugin: String,
        port: serde_json::Value,
        capabilities: Vec<String>,
        meta: serde_json::Value,
    },
    #[serde(rename = "request")]
    Request {
        requestId: Uuid,
        capability: String,
        payload: serde_json::Value,
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
    #[serde(rename = "response")]
    Response {
        requestId: Uuid,
        success: bool,
        data: Option<serde_json::Value>,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    #[serde(rename = "request")]
    Request {
        requestId: Uuid,
        capability: String,
        payload: serde_json::Value,
        token: Option<String>,
    },
    #[serde(rename = "response")]
    Response {
        requestId: Uuid,
        success: bool,
        data: Option<serde_json::Value>,
        error: Option<String>,
    },
}

#[derive(Debug)]
pub enum OutboundCommand {
    Respond {
        request_id: Uuid,
        payload: Result<serde_json::Value, String>,
    },
    Invoke {
        request_id: Uuid,
        capability: String,
        payload: serde_json::Value,
        responder: oneshot::Sender<Result<serde_json::Value, String>>,
    },
    Log {
        level: String,
        message: String,
    },
    Health {
        status: String,
        detail: Option<String>,
    },
}

pub type HandlerFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send>>;

pub type Handler =
    dyn Fn(Uuid, String, serde_json::Value, Option<String>) -> HandlerFuture + Send + Sync;

pub fn channel() -> (
    mpsc::Sender<OutboundCommand>,
    mpsc::Receiver<OutboundCommand>,
) {
    mpsc::channel(256)
}

#[instrument(skip_all)]
pub async fn start_bus(
    plugin: String,
    bus_url: &str,
    port: u16,
    capabilities: Vec<String>,
    meta: serde_json::Value,
    mut receiver: mpsc::Receiver<OutboundCommand>,
    sender: mpsc::Sender<OutboundCommand>,
    handlers: Arc<HashMap<String, Arc<Handler>>>,
) -> Result<JoinHandle<()>, BusError> {
    let (ws_stream, _) = connect_async(bus_url)
        .await
        .map_err(|err| BusError::Connection(format!("{err}")))?;
    let (mut writer, mut reader) = ws_stream.split();
    let register = OutgoingMessage::Register {
        plugin: plugin.clone(),
        port: serde_json::Value::Number(port.into()),
        capabilities: capabilities.clone(),
        meta,
    };
    let register_text =
        serde_json::to_string(&register).map_err(|err| BusError::Registration(format!("{err}")))?;
    writer
        .send(Message::Text(register_text))
        .await
        .map_err(|err| BusError::Connection(format!("{err}")))?;

    let pending: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Result<serde_json::Value, String>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let pending_map = Arc::clone(&pending);
    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(cmd) = receiver.recv() => {
                    match cmd {
                        OutboundCommand::Respond { request_id, payload } => {
                            let (success, data, error_message) = match payload {
                                Ok(data) => (true, Some(data), None),
                                Err(err) => (false, None, Some(err)),
                            };
                            if let Err(err) = send_outgoing(&mut writer, &OutgoingMessage::Response {
                                requestId: request_id,
                                success,
                                data,
                                error: error_message,
                            }).await {
                                error!(%request_id, error = %err, "failed to send response");
                            }
                        }
                        OutboundCommand::Invoke { request_id, capability, payload, responder } => {
                            pending_map.lock().await.insert(request_id, responder);
                            if let Err(err) = send_outgoing(&mut writer, &OutgoingMessage::Request {
                                requestId: request_id,
                                capability,
                                payload,
                            }).await {
                                error!(%request_id, error = %err, "failed to send invoke request");
                                if let Some(sender) = pending_map.lock().await.remove(&request_id) {
                                    let _ = sender.send(Err(format!("transport error: {err}")));
                                }
                            }
                        }
                        OutboundCommand::Log { level, message } => {
                            let log = OutgoingMessage::Log {
                                plugin: plugin.clone(),
                                level,
                                message,
                                timestamp: Utc::now().to_rfc3339(),
                            };
                            if let Err(err) = send_outgoing(&mut writer, &log).await {
                                error!(error = %err, "failed to send log message");
                            }
                        }
                        OutboundCommand::Health { status, detail } => {
                            let health = OutgoingMessage::Health {
                                plugin: plugin.clone(),
                                status,
                                detail,
                            };
                            if let Err(err) = send_outgoing(&mut writer, &health).await {
                                error!(error = %err, "failed to send health message");
                            }
                        }
                    }
                }
                Some(Ok(Message::Text(text))) = reader.next() => {
                    match serde_json::from_str::<IncomingMessage>(&text) {
                        Ok(IncomingMessage::Request { requestId, capability, payload, token }) => {
                            if let Some(handler) = handlers.get(&capability) {
                                let handler = handler.clone();
                                let sender = sender.clone();
                                tokio::spawn(async move {
                                    let result = handler(requestId, capability.clone(), payload.clone(), token).await;
                                    let command = OutboundCommand::Respond {
                                        request_id: requestId,
                                        payload: result,
                                    };
                                    if let Err(err) = sender.send(command).await {
                                        error!(%requestId, error = %err, "failed to send handler response");
                                    }
                                });
                            } else {
                                let _ = sender
                                    .send(OutboundCommand::Respond {
                                        request_id: requestId,
                                        payload: Err(format!("no handler for {capability}")),
                                    })
                                    .await;
                            }
                        }
                        Ok(IncomingMessage::Response { requestId, success, data, error }) => {
                            if let Some(responder) = pending_map.lock().await.remove(&requestId) {
                                let result = if success {
                                    Ok(data.unwrap_or(serde_json::Value::Null))
                                } else {
                                    Err(error.unwrap_or_else(|| "unknown error".to_string()))
                                };
                                if responder.send(result).is_err() {
                                    error!(%requestId, "failed to deliver invoke response");
                                }
                            }
                        }
                        Err(err) => error!(error = %err, "failed to parse bus message"),
                    }
                }
                else => {
                    info!("plugin bus loop finished");
                    break;
                }
            }
        }
    });
    Ok(handle)
}

async fn send_outgoing(
    writer: &mut WebSocketWriter,
    message: &OutgoingMessage,
) -> Result<(), String> {
    match serde_json::to_string(message) {
        Ok(text) => writer
            .send(Message::Text(text))
            .await
            .map_err(|err| err.to_string()),
        Err(err) => Err(err.to_string()),
    }
}
