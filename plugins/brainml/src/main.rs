use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::Router;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::oneshot;
use tracing::{error, info};

use brainml::adapters::braindb::{BraindbClient, NullBraindbClient};
use brainml::adapters::llm::{LlmClient, NullLlmClient};
use brainml::core::bus::{channel, start_bus, Handler, OutboundCommand};
use brainml::core::config::{BrainmlConfig, BrainmlConfigLoader};
use brainml::core::pipeline::PipelineManager;
use brainml::util::tracing::init_tracing;

mod plugin_interface {
    use super::*;

    #[async_trait]
    pub trait BkgPlugin {
        async fn init(&mut self, config: BrainmlConfig) -> Result<()>;
        fn routes(&self) -> Router;
        async fn shutdown(&mut self) -> Result<()>;
        async fn migrations(&self) -> Result<()> {
            Ok(())
        }
    }
}

use plugin_interface::BkgPlugin;

struct BrainmlPlugin {
    name: String,
    state: brainml::api::AppState,
    bus_handle: Option<tokio::task::JoinHandle<()>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl BrainmlPlugin {
    fn new(name: String, state: brainml::api::AppState) -> Self {
        Self {
            name,
            state,
            bus_handle: None,
            server_handle: None,
            shutdown_tx: None,
        }
    }
}

#[async_trait]
impl BkgPlugin for BrainmlPlugin {
    async fn init(&mut self, config: BrainmlConfig) -> Result<()> {
        let router = self.routes();
        let port_env = std::env::var("PLUGIN_PORT").ok();
        let port = if let Some(port_str) = port_env {
            port_str.parse::<u16>().context("invalid PLUGIN_PORT")?
        } else {
            config.port
        };
        let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
        let listener = TcpListener::bind(addr).await?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);
        let server = axum::serve(listener, router).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });
        self.server_handle = Some(tokio::spawn(async move {
            if let Err(err) = server.await {
                error!(error = %err, "http server terminated");
            }
        }));
        info!(port, "brainml http server listening");

        let handlers = build_handlers(self.state.clone());
        let bus_url = bus_endpoint(&config);
        let (tx, rx) = channel();
        let bus_handle = start_bus(
            self.name.clone(),
            &bus_url,
            port,
            capabilities(),
            serde_json::json!({"version": env!("CARGO_PKG_VERSION")}),
            rx,
            tx.clone(),
            handlers,
        )
        .await
        .map_err(|err| anyhow::anyhow!("failed to start bus: {err}"))?;
        self.bus_handle = Some(bus_handle);
        Ok(())
    }

    fn routes(&self) -> Router {
        brainml::api::router(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.server_handle.take() {
            handle.await.ok();
        }
        if let Some(handle) = self.bus_handle.take() {
            handle.await.ok();
        }
        Ok(())
    }
}

fn capabilities() -> Vec<String> {
    vec![
        "brainml.index".into(),
        "brainml.query".into(),
        "brainml.train".into(),
        "brainml.stats".into(),
        "brainml.admin".into(),
    ]
}

fn bus_endpoint(config: &BrainmlConfig) -> String {
    if let Ok(port) = std::env::var("BKG_PLUGIN_BUS_PORT") {
        format!("ws://127.0.0.1:{port}")
    } else {
        config.bus.clone()
    }
}

fn build_handlers(state: brainml::api::AppState) -> Arc<HashMap<String, Arc<Handler>>> {
    let mut map: HashMap<String, Arc<Handler>> = HashMap::new();
    for capability in capabilities() {
        let state_clone = state.clone();
        let handler: Arc<Handler> = match capability.as_str() {
            "brainml.index" => Arc::new(move |_id, _capability, payload, _token| {
                let state = state_clone.clone();
                Box::pin(async move {
                    let request: brainml::core::schema::IndexRequest =
                        serde_json::from_value(payload).map_err(|err| err.to_string())?;
                    let response = state
                        .process_index(request)
                        .await
                        .map_err(|err| err.to_string())?;
                    serde_json::to_value(response).map_err(|err| err.to_string())
                })
            }),
            "brainml.query" => Arc::new(move |_id, _capability, payload, _token| {
                let state = state_clone.clone();
                Box::pin(async move {
                    let request: brainml::core::schema::QueryRequest =
                        serde_json::from_value(payload).map_err(|err| err.to_string())?;
                    let response = state
                        .process_query(request)
                        .await
                        .map_err(|err| err.to_string())?;
                    serde_json::to_value(response).map_err(|err| err.to_string())
                })
            }),
            "brainml.train" => Arc::new(move |_id, _capability, payload, _token| {
                let state = state_clone.clone();
                Box::pin(async move {
                    let request: brainml::core::schema::TrainRequest =
                        serde_json::from_value(payload).map_err(|err| err.to_string())?;
                    let response = state
                        .process_train(request)
                        .await
                        .map_err(|err| err.to_string())?;
                    serde_json::to_value(response).map_err(|err| err.to_string())
                })
            }),
            "brainml.stats" => Arc::new(move |_id, _capability, _payload, _token| {
                let state = state_clone.clone();
                Box::pin(async move {
                    let stats = state.braindb.stats().await.map_err(|err| err.to_string())?;
                    serde_json::to_value(stats).map_err(|err| err.to_string())
                })
            }),
            "brainml.admin" => Arc::new(move |_id, _capability, _payload, _token| {
                let state = state_clone.clone();
                Box::pin(async move {
                    let status = brainml::core::schema::AdminStatus {
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        uptime_seconds: state.start_time.elapsed().as_secs(),
                        capabilities: capabilities(),
                    };
                    serde_json::to_value(status).map_err(|err| err.to_string())
                })
            }),
            other => {
                let error = format!("unsupported capability {other}");
                Arc::new(move |_id, _capability, _payload, _token| {
                    Box::pin(async move { Err(error.clone()) })
                })
            }
        };
        map.insert(capability, handler);
    }
    Arc::new(map)
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let plugin_name = std::env::var("BKG_PLUGIN_NAME").unwrap_or_else(|_| "brainml".to_string());
    let config_path = std::env::var("BRAINML_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            path.push("config.json");
            path
        });
    let config = BrainmlConfigLoader::load(&config_path)?;
    let braindb: Arc<dyn BraindbClient> = Arc::new(NullBraindbClient::default());
    let llm: Arc<dyn LlmClient> = Arc::new(NullLlmClient::default());

    let state = brainml::api::AppState {
        braindb,
        llm,
        pipeline: PipelineManager::default(),
        config: config.clone(),
        start_time: std::time::Instant::now(),
    };

    let mut plugin = BrainmlPlugin::new(plugin_name, state);
    plugin.init(config).await?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            let _ = shutdown_tx.send(());
        }
    });
    let _ = shutdown_rx.await;
    plugin.shutdown().await?;
    Ok(())
}
