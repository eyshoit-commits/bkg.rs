use std::{env, sync::Arc, time::Duration};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc, Mutex as TokioMutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

use crate::{
    manager::{GooseRunRequest, LoadTestManager},
    models::GooseRunSummaryList,
};

#[derive(Debug)]
pub enum BusEvent {
    Request {
        request_id: String,
        capability: String,
        payload: Value,
    },
}

#[derive(Clone)]
pub struct PluginBusClient {
    plugin: Arc<String>,
    sender: Arc<TokioMutex<mpsc::UnboundedSender<Value>>>,
}

impl PluginBusClient {
    pub async fn connect(
        plugin: String,
        port: u16,
        capabilities: Vec<String>,
        config_schema: Value,
    ) -> Result<(Self, mpsc::Receiver<BusEvent>)> {
        let bus_port = env::var("BKG_PLUGIN_BUS_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(43121);
        let url = format!("ws://127.0.0.1:{bus_port}");
        let (ws_stream, _) = connect_async(&url)
            .await
            .with_context(|| format!("failed to connect to plugin bus at {url}"))?;
        let (mut writer, mut reader) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Value>();
        let (event_tx, event_rx) = mpsc::channel::<BusEvent>(32);
        let client = PluginBusClient {
            plugin: Arc::new(plugin.clone()),
            sender: Arc::new(TokioMutex::new(tx)),
        };
        let registration = json!({
            "type": "register",
            "plugin": plugin,
            "port": port,
            "capabilities": capabilities,
            "configSchema": config_schema,
        });
        client.send_raw(registration).await?;
        let writer_plugin = client.plugin.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let payload = Message::Text(message.to_string());
                if let Err(err) = writer.send(payload).await {
                    error!(error = %err, plugin = %writer_plugin, "failed to write plugin bus message");
                    break;
                }
            }
        });
        let event_plugin = client.plugin.clone();
        tokio::spawn(async move {
            while let Some(message) = reader.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Err(err) = handle_incoming(&event_tx, &text).await {
                            warn!(error = %err, plugin = %event_plugin, "failed to handle bus message");
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        info!(plugin = %event_plugin, "bus connection closed: {:?}", frame);
                        break;
                    }
                    Ok(other) => {
                        info!(plugin = %event_plugin, "ignoring bus frame: {other:?}");
                    }
                    Err(err) => {
                        error!(error = %err, plugin = %event_plugin, "plugin bus read error");
                        break;
                    }
                }
            }
        });
        info!(plugin = %client.plugin.as_ref(), "connected to plugin bus on {url}");
        Ok((client, event_rx))
    }

    pub async fn log_info(&self, message: &str) {
        let _ = self
            .send_raw(json!({
                "type": "log",
                "plugin": self.plugin.as_ref(),
                "level": "info",
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            }))
            .await;
    }

    pub async fn log_warn(&self, message: &str) {
        let _ = self
            .send_raw(json!({
                "type": "log",
                "plugin": self.plugin.as_ref(),
                "level": "warn",
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            }))
            .await;
    }

    pub async fn send_response(&self, request_id: String, data: Value) -> Result<()> {
        self.send_raw(json!({
            "type": "response",
            "requestId": request_id,
            "success": true,
            "data": data,
        }))
        .await
    }

    pub async fn send_error(&self, request_id: String, error: &str) -> Result<()> {
        self.send_raw(json!({
            "type": "response",
            "requestId": request_id,
            "success": false,
            "error": error,
        }))
        .await
    }

    pub async fn send_health(&self) -> Result<()> {
        self.send_raw(json!({
            "type": "health",
            "plugin": self.plugin.as_ref(),
            "status": "up",
        }))
        .await
    }

    pub async fn send_telemetry(&self, cpu: f64, mem_bytes: u64, entries: u64) -> Result<()> {
        self.send_raw(json!({
            "type": "telemetry",
            "plugin": self.plugin.as_ref(),
            "cpu": cpu,
            "mem_bytes": mem_bytes,
            "entries": entries,
        }))
        .await
    }

    async fn send_raw(&self, value: Value) -> Result<()> {
        let sender = self.sender.lock().await;
        sender
            .send(value)
            .map_err(|err| anyhow!("failed to queue bus message: {err}"))
    }
}

async fn handle_incoming(event_tx: &mpsc::Sender<BusEvent>, payload: &str) -> Result<()> {
    let message: Value = serde_json::from_str(payload)?;
    if message.get("type").and_then(Value::as_str) == Some("request") {
        let request_id = message
            .get("requestId")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("requestId missing"))?
            .to_string();
        let capability = message
            .get("capability")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("capability missing"))?
            .to_string();
        let payload = message.get("payload").cloned().unwrap_or(Value::Null);
        event_tx
            .send(BusEvent::Request {
                request_id,
                capability,
                payload,
            })
            .await
            .context("failed to forward bus request")?;
    }
    Ok(())
}

pub async fn handle_request(
    bus: &PluginBusClient,
    manager: &LoadTestManager,
    request_id: String,
    capability: String,
    payload: Value,
) -> Result<()> {
    match capability.as_str() {
        "goose.run" => {
            let request: GooseRunRequest = serde_json::from_value(payload)?;
            match manager.start_run(request).await {
                Ok(response) => {
                    bus.send_response(request_id, serde_json::to_value(response)?)
                        .await?
                }
                Err(err) => bus.send_error(request_id, &err.to_string()).await?,
            }
        }
        "goose.stop" => match manager.stop_run().await {
            Ok(response) => {
                bus.send_response(request_id, serde_json::to_value(response)?)
                    .await?
            }
            Err(err) => bus.send_error(request_id, &err.to_string()).await?,
        },
        "goose.status" => match manager.status().await {
            Ok(status) => {
                bus.send_response(request_id, serde_json::to_value(status)?)
                    .await?
            }
            Err(err) => bus.send_error(request_id, &err.to_string()).await?,
        },
        "goose.history" => match manager.history().await {
            Ok(history) => {
                let list = GooseRunSummaryList { runs: history };
                bus.send_response(request_id, serde_json::to_value(list)?)
                    .await?;
            }
            Err(err) => bus.send_error(request_id, &err.to_string()).await?,
        },
        other => {
            bus.send_error(request_id, &format!("unsupported capability {other}"))
                .await?;
        }
    }
    Ok(())
}

pub async fn run_heartbeat_loop(
    client: PluginBusClient,
    shutdown: broadcast::Sender<()>,
) -> Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_secs(10));
    let mut shutdown_rx = shutdown.subscribe();
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(err) = client.send_health().await {
                    warn!(error = %err, "failed to send heartbeat");
                }
            }
            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }
    Ok(())
}
