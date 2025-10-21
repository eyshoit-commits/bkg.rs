use std::{env, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use tokio::{net::TcpListener, select, signal, sync::broadcast, task::JoinSet};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

mod bus;
mod config;
mod manager;
mod models;
mod telemetry;

use bus::PluginBusClient;
use config::{load_plugin_config, GoosePluginConfig};
use manager::{GooseRunRequest, GooseStatus, LoadTestManager};
use models::{GooseRunResponse, GooseRunSummaryList};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let config_path = env::var("BKG_PLUGIN_CONFIG_PATH").ok().map(PathBuf::from);
    let plugin_config = load_plugin_config(config_path.clone())
        .context("failed to load goose plugin configuration")?;

    let http_listener = TcpListener::bind(("0.0.0.0", 0))
        .await
        .context("failed to bind goose HTTP listener")?;
    let http_addr: SocketAddr = http_listener.local_addr()?;

    let history_path = env::var("BKG_PLUGIN_HISTORY_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            config_path
                .as_ref()
                .and_then(|path| path.parent().map(|dir| dir.join("history.json")))
        })
        .or_else(|| Some(PathBuf::from("history.json")));

    let manager = LoadTestManager::new(
        plugin_config.settings.clone(),
        config_path.clone(),
        history_path,
    );

    let (shutdown_tx, _) = broadcast::channel::<()>(4);
    let http_shutdown = shutdown_tx.clone();

    let (bus_client, mut bus_events) = PluginBusClient::connect(
        plugin_config.name.clone(),
        http_addr.port(),
        plugin_config.capabilities.clone(),
        plugin_config.config_schema.clone(),
    )
    .await
    .context("failed to connect to plugin bus")?;

    bus_client.log_info("goose plugin runtime started").await;

    let shared_state = AppState {
        manager: manager.clone(),
        bus: bus_client.clone(),
        config: plugin_config.clone(),
    };

    let http_app = Router::new()
        .route("/health", get(health))
        .route("/v1/goose/status", get(http_status))
        .route("/v1/goose/history", get(http_history))
        .route("/v1/goose/run", post(http_run))
        .route("/v1/goose/stop", post(http_stop))
        .with_state(shared_state.clone());

    let mut tasks = JoinSet::new();

    tasks.spawn(async move {
        if let Err(error) = axum::serve(http_listener, http_app)
            .with_graceful_shutdown(wait_for_shutdown(http_shutdown.clone()))
            .await
        {
            error!(%error, "goose HTTP server terminated unexpectedly");
        }
    });

    let telemetry_manager = manager.clone();
    let telemetry_bus = bus_client.clone();
    let telemetry_shutdown = shutdown_tx.clone();
    tasks.spawn(async move {
        if let Err(err) =
            telemetry::run_telemetry_loop(telemetry_bus, telemetry_manager, telemetry_shutdown)
                .await
        {
            error!(error = %err, "telemetry loop failed");
        }
    });

    let heartbeat_bus = bus_client.clone();
    let heartbeat_shutdown = shutdown_tx.clone();
    tasks.spawn(async move {
        if let Err(err) = bus::run_heartbeat_loop(heartbeat_bus, heartbeat_shutdown).await {
            error!(error = %err, "heartbeat loop failed");
        }
    });

    let request_manager = manager.clone();
    let request_bus = bus_client.clone();
    let request_shutdown = shutdown_tx.clone();
    tasks.spawn(async move {
        while let Some(event) = bus_events.recv().await {
            match event {
                bus::BusEvent::Request {
                    request_id,
                    capability,
                    payload,
                } => {
                    if let Err(err) = bus::handle_request(
                        &request_bus,
                        &request_manager,
                        request_id,
                        capability,
                        payload,
                    )
                    .await
                    {
                        error!(error = %err, "failed to handle bus request");
                    }
                }
            }
        }
        let _ = request_shutdown.send(());
    });

    let shutdown_signal = wait_for_shutdown(shutdown_tx.clone());

    select! {
        _ = shutdown_signal => {
            info!("received shutdown signal");
        }
        result = tasks.join_next() => {
            if let Some(Err(err)) = result {
                error!(error = %err, "task failed");
            }
        }
    }

    bus_client.log_info("goose plugin runtime stopping").await;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}

#[derive(Clone)]
struct AppState {
    manager: LoadTestManager,
    bus: PluginBusClient,
    config: GoosePluginConfig,
}

type AppResult<T> = std::result::Result<T, axum::http::StatusCode>;

async fn wait_for_shutdown(notify: broadcast::Sender<()>) {
    let mut shutdown_rx = notify.subscribe();
    select! {
        _ = signal::ctrl_c() => {
            warn!("received ctrl+c, shutting down");
        }
        _ = shutdown_rx.recv() => {
            info!("shutdown requested");
        }
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn http_status(State(state): State<AppState>) -> AppResult<Json<GooseStatus>> {
    let status = state
        .manager
        .status()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(status))
}

async fn http_history(State(state): State<AppState>) -> AppResult<Json<GooseRunSummaryList>> {
    let history = state
        .manager
        .history()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(GooseRunSummaryList { runs: history }))
}

async fn http_run(
    State(state): State<AppState>,
    Json(request): Json<GooseRunRequest>,
) -> AppResult<Json<GooseRunResponse>> {
    let result = state
        .manager
        .start_run(request)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    state.bus.log_info("goose run started via REST API").await;
    Ok(Json(result))
}

async fn http_stop(State(state): State<AppState>) -> AppResult<Json<GooseRunResponse>> {
    let result = state
        .manager
        .stop_run()
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    state.bus.log_info("goose run stopped via REST API").await;
    Ok(Json(result))
}
