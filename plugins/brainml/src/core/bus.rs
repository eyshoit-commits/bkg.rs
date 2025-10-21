use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, instrument};
use uuid::Uuid;

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
    writer
        .send(Message::Text(
            serde_json::to_string(&register)
                .map_err(|err| BusError::Registration(format!("{err}")))?,
        ))
        .await
        .map_err(|err| BusError::Connection(format!("{err}")))?;

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(cmd) = receiver.recv() => {
                    match cmd {
                        OutboundCommand::Respond { request_id, payload } => {
                            let message = match payload {
                                Ok(data) => OutgoingMessage::Response {
                                    requestId: request_id,
                                    success: true,
                                    data: Some(data),
                                    error: None,
                                },
                                Err(error) => OutgoingMessage::Response {
                                    requestId: request_id,
                                    success: false,
                                    data: None,
                                    error: Some(error),
                                },
                            };
                            if let Err(err) = writer
                                .send(Message::Text(serde_json::to_string(&message).unwrap()))
                                .await
                            {
                                error!(%request_id, error = %err, "failed to send response");
                            }
                        }
                        OutboundCommand::Log { level, message } => {
                            let log = OutgoingMessage::Log {
                                plugin: plugin.clone(),
                                level,
                                message,
                                timestamp: Utc::now().to_rfc3339(),
                            };
                            if let Err(err) = writer
                                .send(Message::Text(serde_json::to_string(&log).unwrap()))
                                .await
                            {
                                error!(error = %err, "failed to send log message");
                            }
                        }
                        OutboundCommand::Health { status, detail } => {
                            let health = OutgoingMessage::Health {
                                plugin: plugin.clone(),
                                status,
                                detail,
                            };
                            if let Err(err) = writer
                                .send(Message::Text(serde_json::to_string(&health).unwrap()))
                                .await
                            {
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
                        Ok(IncomingMessage::Response { .. }) => {
                            // ignore responses
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
