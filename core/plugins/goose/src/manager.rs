use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use hdrhistogram::Histogram;
use parking_lot::{Mutex as ParkingMutex, RwLock};
use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::SmallRng, SeedableRng};
use reqwest::{header::HeaderMap, redirect::Policy, Method};
use serde_json::{from_str, to_writer_pretty};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    config::load_plugin_config,
    models::{
        GooseEffectiveSettings, GooseMetricsSnapshot, GooseRunRequest, GooseRunResponse,
        GooseRunState, GooseRunSummary, GooseSettings, GooseStatus,
    },
};

#[derive(Clone)]
pub struct LoadTestManager {
    inner: Arc<RwLock<ManagerState>>,
    history: Arc<RwLock<Vec<GooseRunSummary>>>,
    defaults: Arc<RwLock<GooseSettings>>,
    config_path: Option<PathBuf>,
    history_path: Option<PathBuf>,
}

struct ManagerState {
    state: GooseRunState,
    active: Option<ActiveRun>,
    last_summary: Option<GooseRunSummary>,
}

struct ActiveRun {
    run_id: Uuid,
    settings: GooseEffectiveSettings,
    started_at: DateTime<Utc>,
    cancel: CancellationToken,
    metrics: Arc<Metrics>,
    plan: Arc<ExecutionPlan>,
    join_handle: Option<JoinHandle<()>>,
}

#[derive(Clone)]
struct ExecutionPlan {
    target: String,
    users: u32,
    hatch_rate: u32,
    run_time: Duration,
    timeout: Duration,
    verify_tls: bool,
    startup_time: Duration,
    graceful_stop: Duration,
    throttle_rps: Option<u32>,
    global_headers: HashMap<String, String>,
    sticky_cookies: bool,
    follow_redirects: bool,
    schedule: Vec<ScheduledRequest>,
    weights: Vec<u32>,
    max_history: usize,
}

struct Throttle {
    rate: usize,
    semaphore: Arc<Semaphore>,
}

impl Throttle {
    fn spawn(rate: u32, cancel: CancellationToken) -> Arc<Self> {
        let rate = rate.max(1) as usize;
        let semaphore = Arc::new(Semaphore::new(rate));
        let throttle = Arc::new(Self {
            rate,
            semaphore: semaphore.clone(),
        });
        let worker = throttle.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let available = worker.semaphore.available_permits();
                        if available < worker.rate {
                            worker.semaphore.add_permits(worker.rate - available);
                        }
                    }
                    _ = cancel.cancelled() => {
                        break;
                    }
                }
            }
        });
        throttle
    }

    async fn acquire(&self) -> Option<OwnedSemaphorePermit> {
        match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => Some(permit),
            Err(_) => None,
        }
    }
}

#[derive(Clone)]
struct ScheduledRequest {
    name: String,
    method: Method,
    path: String,
    weight: u32,
    body: Option<String>,
    headers: HashMap<String, String>,
    query: HashMap<String, String>,
    think_time: Duration,
}

struct Metrics {
    start_instant: Instant,
    histogram: ParkingMutex<Histogram<u64>>, // microseconds
    total: ParkingMutex<u64>,
    success: ParkingMutex<u64>,
    failed: ParkingMutex<u64>,
    bytes_sent: ParkingMutex<u64>,
    bytes_received: ParkingMutex<u64>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            start_instant: Instant::now(),
            histogram: ParkingMutex::new(Histogram::new(3).expect("failed to create histogram")),
            total: ParkingMutex::new(0),
            success: ParkingMutex::new(0),
            failed: ParkingMutex::new(0),
            bytes_sent: ParkingMutex::new(0),
            bytes_received: ParkingMutex::new(0),
        }
    }

    fn record(&self, duration: Duration, success: bool, sent: u64, received: u64) {
        let micros = duration.as_micros() as u64;
        {
            let mut histogram = self.histogram.lock();
            let _ = histogram.record(micros.max(1));
        }
        {
            let mut total = self.total.lock();
            *total += 1;
        }
        if success {
            let mut ok = self.success.lock();
            *ok += 1;
        } else {
            let mut failed = self.failed.lock();
            *failed += 1;
        }
        if sent > 0 {
            let mut bytes = self.bytes_sent.lock();
            *bytes += sent;
        }
        if received > 0 {
            let mut bytes = self.bytes_received.lock();
            *bytes += received;
        }
    }

    fn snapshot(&self) -> GooseMetricsSnapshot {
        let elapsed = self.start_instant.elapsed().as_secs_f64().max(0.000_001);
        let histogram = self.histogram.lock();
        let total = *self.total.lock();
        let success = *self.success.lock();
        let failed = *self.failed.lock();
        let bytes_sent = *self.bytes_sent.lock();
        let bytes_received = *self.bytes_received.lock();
        let mean = if total > 0 {
            histogram.mean() / 1000.0
        } else {
            0.0
        };
        let p95 = if total > 0 {
            histogram.value_at_quantile(0.95) as f64 / 1000.0
        } else {
            0.0
        };
        let p99 = if total > 0 {
            histogram.value_at_quantile(0.99) as f64 / 1000.0
        } else {
            0.0
        };
        GooseMetricsSnapshot {
            total_requests: total,
            success_requests: success,
            failed_requests: failed,
            requests_per_second: total as f64 / elapsed,
            average_latency_ms: mean,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            bytes_sent,
            bytes_received,
        }
    }
}

impl LoadTestManager {
    pub fn new(
        settings: GooseSettings,
        config_path: Option<PathBuf>,
        history_path: Option<PathBuf>,
    ) -> Self {
        let history_data = load_history_from_disk(history_path.as_ref());
        Self {
            inner: Arc::new(RwLock::new(ManagerState {
                state: GooseRunState::Idle,
                active: None,
                last_summary: None,
            })),
            history: Arc::new(RwLock::new(history_data)),
            defaults: Arc::new(RwLock::new(settings)),
            config_path,
            history_path,
        }
    }

    pub async fn start_run(&self, request: GooseRunRequest) -> Result<GooseRunResponse> {
        let defaults = self.refresh_defaults().context("failed to load defaults")?;
        let plan = ExecutionPlan::from_request(&defaults, &request)?;
        let mut inner = self.inner.write();
        if matches!(
            inner.state,
            GooseRunState::Running | GooseRunState::Starting | GooseRunState::Stopping
        ) {
            return Err(anyhow!("a run is already in progress"));
        }
        inner.state = GooseRunState::Starting;
        let run_id = Uuid::new_v4();
        let metrics = Arc::new(Metrics::new());
        let cancel = CancellationToken::new();
        let started_at = Utc::now();
        let settings = GooseEffectiveSettings {
            target: plan.target.clone(),
            users: plan.users,
            hatch_rate: plan.hatch_rate,
            run_time_seconds: plan.run_time.as_secs(),
            timeout_seconds: plan.timeout.as_secs(),
            verify_tls: plan.verify_tls,
            startup_time_seconds: plan.startup_time.as_secs(),
            graceful_stop_seconds: plan.graceful_stop.as_secs(),
            throttle_rps: plan.throttle_rps,
            sticky_cookies: plan.sticky_cookies,
            follow_redirects: plan.follow_redirects,
        };
        let manager = self.clone();
        let plan_arc = Arc::new(plan);
        let plan_for_task = plan_arc.clone();
        let cancel_for_task = cancel.clone();
        let metrics_for_task = metrics.clone();
        let join_handle = tokio::spawn(async move {
            if let Err(err) = execute_load_test(
                run_id,
                plan_for_task,
                metrics_for_task,
                cancel_for_task.clone(),
            )
            .await
            {
                error!(error = %err, "goose run failed");
                manager
                    .finish_run(run_id, GooseRunState::Failed, None)
                    .await;
            } else {
                manager
                    .finish_run(run_id, GooseRunState::Completed, None)
                    .await;
            }
        });
        inner.active = Some(ActiveRun {
            run_id,
            settings,
            started_at,
            cancel,
            metrics,
            plan: plan_arc,
            join_handle: Some(join_handle),
        });
        inner.state = GooseRunState::Running;
        drop(inner);
        info!(%run_id, "started goose load test");
        let status = self.status().await?;
        Ok(GooseRunResponse {
            status: status.status,
            run_id: status.run_id,
            message: "Goose run started".to_string(),
            metrics: status.metrics,
        })
    }

    pub async fn stop_run(&self) -> Result<GooseRunResponse> {
        let join_handle = {
            let mut inner = self.inner.write();
            let active = inner
                .active
                .as_mut()
                .ok_or_else(|| anyhow!("no active run"))?;
            if matches!(inner.state, GooseRunState::Stopping) {
                return Err(anyhow!("run already stopping"));
            }
            inner.state = GooseRunState::Stopping;
            let graceful_stop = active.plan.graceful_stop;
            if graceful_stop.is_zero() {
                active.cancel.cancel();
            } else {
                let cancel_clone = active.cancel.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(graceful_stop).await;
                    cancel_clone.cancel();
                });
            }
            active.join_handle.take()
        };
        if let Some(handle) = join_handle {
            let _ = handle.await;
        }
        let status = self.status().await?;
        Ok(GooseRunResponse {
            status: status.status,
            run_id: status.run_id,
            message: "Goose run stopped".to_string(),
            metrics: status.metrics,
        })
    }

    pub async fn status(&self) -> Result<GooseStatus> {
        let inner = self.inner.read();
        let (status, run_id, started_at, finished_at, settings, metrics) =
            if let Some(active) = inner.active.as_ref() {
                let metrics = active.metrics.snapshot();
                (
                    inner.state.clone(),
                    Some(active.run_id),
                    Some(active.started_at),
                    None,
                    active.settings.clone(),
                    metrics,
                )
            } else if let Some(summary) = inner.last_summary.as_ref() {
                (
                    summary.status.clone(),
                    Some(summary.run_id),
                    Some(summary.started_at),
                    Some(summary.finished_at),
                    summary.settings.clone(),
                    summary.metrics.clone(),
                )
            } else {
                let defaults = self.defaults.read().clone();
                (
                    GooseRunState::Idle,
                    None,
                    None,
                    None,
                    GooseEffectiveSettings {
                        target: defaults.default_target,
                        users: defaults.users,
                        hatch_rate: defaults.hatch_rate,
                        run_time_seconds: defaults.run_time_seconds,
                        timeout_seconds: defaults.timeout_seconds,
                        verify_tls: defaults.verify_tls,
                        startup_time_seconds: defaults.startup_time_seconds,
                        graceful_stop_seconds: defaults.graceful_stop_seconds,
                        throttle_rps: defaults.throttle_rps.and_then(|value| {
                            if value > 0 {
                                Some(value)
                            } else {
                                None
                            }
                        }),
                        sticky_cookies: defaults.sticky_cookies,
                        follow_redirects: defaults.follow_redirects,
                    },
                    GooseMetricsSnapshot::default(),
                )
            };
        let duration_seconds = match (started_at, finished_at) {
            (Some(start), Some(end)) => (end - start).num_milliseconds() as f64 / 1000.0,
            (Some(start), None) => (Utc::now() - start).num_milliseconds() as f64 / 1000.0,
            _ => 0.0,
        };
        Ok(GooseStatus {
            status,
            run_id,
            started_at,
            finished_at,
            duration_seconds,
            settings,
            metrics,
        })
    }

    pub async fn history(&self) -> Result<Vec<GooseRunSummary>> {
        Ok(self.history.read().clone())
    }

    async fn finish_run(
        &self,
        run_id: Uuid,
        final_state: GooseRunState,
        summary_override: Option<GooseRunSummary>,
    ) {
        let mut inner = self.inner.write();
        let (completion, max_history) = if let Some(summary) = summary_override {
            let max = self.defaults.read().max_history.max(1);
            (summary, max)
        } else if let Some(active) = inner.active.take() {
            let finished_at = Utc::now();
            let metrics = active.metrics.snapshot();
            let max_history = active.plan.max_history.max(1);
            (
                GooseRunSummary {
                    run_id,
                    started_at: active.started_at,
                    finished_at,
                    duration_seconds: (finished_at - active.started_at).num_milliseconds() as f64
                        / 1000.0,
                    status: final_state.clone(),
                    settings: active.settings,
                    metrics,
                },
                max_history,
            )
        } else {
            return;
        };
        inner.last_summary = Some(completion.clone());
        inner.state = final_state;
        drop(inner);
        let mut history = self.history.write();
        history.push(completion.clone());
        if history.len() > max_history {
            let overflow = history.len() - max_history;
            history.drain(0..overflow);
        }
        drop(history);
        if let Err(err) = self.persist_history() {
            warn!(error = %err, "failed to persist goose history");
        }
    }

    fn refresh_defaults(&self) -> Result<GooseSettings> {
        if let Some(path) = &self.config_path {
            let config = load_plugin_config(Some(path.clone()))?;
            *self.defaults.write() = config.settings.clone();
        }
        Ok(self.defaults.read().clone())
    }

    fn persist_history(&self) -> Result<()> {
        let Some(path) = self.history_path.as_ref() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create history directory at {}", parent.display())
                })?;
            }
        }
        let history = self.history.read().clone();
        let wrapper = GooseRunSummaryList { runs: history };
        let file = File::create(path)
            .with_context(|| format!("failed to open history file at {}", path.display()))?;
        let mut writer = BufWriter::new(file);
        to_writer_pretty(&mut writer, &wrapper)
            .with_context(|| format!("failed to serialize history to {}", path.display()))?;
        Ok(())
    }
}

fn load_history_from_disk(path: Option<&PathBuf>) -> Vec<GooseRunSummary> {
    let Some(path) = path else {
        return Vec::new();
    };
    match fs::read_to_string(path) {
        Ok(content) => {
            if content.trim().is_empty() {
                Vec::new()
            } else {
                match from_str::<GooseRunSummaryList>(&content) {
                    Ok(wrapper) => wrapper.runs,
                    Err(err) => {
                        warn!(error = %err, file = %path.display(), "failed to parse goose history");
                        Vec::new()
                    }
                }
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(err) => {
            warn!(error = %err, file = %path.display(), "failed to load goose history");
            Vec::new()
        }
    }
}

impl ExecutionPlan {
    fn from_request(defaults: &GooseSettings, request: &GooseRunRequest) -> Result<Self> {
        let target = request
            .target
            .clone()
            .or_else(|| Some(defaults.default_target.clone()))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("target URL missing"))?;
        let users = request.users.unwrap_or(defaults.users).max(1);
        let hatch_rate = request.hatch_rate.unwrap_or(defaults.hatch_rate).max(1);
        let run_time = Duration::from_secs(
            request
                .run_time_seconds
                .unwrap_or(defaults.run_time_seconds)
                .max(1),
        );
        let timeout = Duration::from_secs(
            request
                .timeout_seconds
                .unwrap_or(defaults.timeout_seconds)
                .max(1),
        );
        let verify_tls = request.verify_tls.unwrap_or(defaults.verify_tls);
        let startup_time = Duration::from_secs(
            request
                .startup_time_seconds
                .unwrap_or(defaults.startup_time_seconds),
        );
        let graceful_stop = Duration::from_secs(
            request
                .graceful_stop_seconds
                .unwrap_or(defaults.graceful_stop_seconds),
        );
        let throttle_rps = request
            .throttle_rps
            .or(defaults.throttle_rps)
            .and_then(|value| if value == 0 { None } else { Some(value) });
        let global_headers = request
            .global_headers
            .clone()
            .unwrap_or_else(|| defaults.global_headers.clone());
        let sticky_cookies = request.sticky_cookies.unwrap_or(defaults.sticky_cookies);
        let follow_redirects = request
            .follow_redirects
            .unwrap_or(defaults.follow_redirects);
        let schedule_source = request
            .schedule
            .clone()
            .unwrap_or_else(|| defaults.schedule.clone());
        if schedule_source.is_empty() {
            return Err(anyhow!("schedule must contain at least one request"));
        }
        let mut schedule = Vec::new();
        for entry in schedule_source {
            let method = Method::from_bytes(entry.method.to_ascii_uppercase().as_bytes())
                .with_context(|| format!("invalid method {}", entry.method))?;
            let think_time = entry.think_time_ms.unwrap_or(0);
            schedule.push(ScheduledRequest {
                name: entry.name.clone(),
                method,
                path: entry.path.clone(),
                weight: entry.weight.max(1),
                body: entry.body.clone(),
                headers: entry.headers.clone(),
                query: entry.query.clone(),
                think_time: Duration::from_millis(think_time),
            });
        }
        let weights = schedule
            .iter()
            .map(|entry| entry.weight)
            .collect::<Vec<_>>();
        Ok(Self {
            target,
            users,
            hatch_rate,
            run_time,
            timeout,
            verify_tls,
            startup_time,
            graceful_stop,
            throttle_rps,
            global_headers,
            sticky_cookies,
            follow_redirects,
            schedule,
            weights,
            max_history: defaults.max_history.max(1),
        })
    }
}

async fn execute_load_test(
    run_id: Uuid,
    plan: Arc<ExecutionPlan>,
    metrics: Arc<Metrics>,
    cancel: CancellationToken,
) -> Result<()> {
    let mut client_builder = reqwest::Client::builder()
        .timeout(plan.timeout)
        .danger_accept_invalid_certs(!plan.verify_tls);
    if plan.sticky_cookies {
        client_builder = client_builder.cookie_store(true);
    }
    if !plan.follow_redirects {
        client_builder = client_builder.redirect(Policy::none());
    }
    let client = client_builder
        .build()
        .context("failed to build http client")?;
    let client = Arc::new(client);
    let start = Instant::now();
    if !plan.startup_time.is_zero() {
        tokio::select! {
            _ = tokio::time::sleep(plan.startup_time) => {},
            _ = cancel.cancelled() => {
                return Ok(());
            }
        }
    }
    let cancel_on_timeout = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(plan.run_time).await;
        cancel_on_timeout.cancel();
    });
    let throttle = plan
        .throttle_rps
        .map(|rate| Throttle::spawn(rate, cancel.clone()));
    let mut handles = Vec::new();
    for idx in 0..plan.users {
        let client = client.clone();
        let plan = plan.clone();
        let metrics = metrics.clone();
        let cancel = cancel.clone();
        let throttle = throttle.clone();
        let handle = tokio::spawn(async move {
            let mut rng = SmallRng::from_entropy();
            let dist = WeightedIndex::new(plan.weights.clone()).expect("invalid weights");
            if idx > 0 {
                let delay = Duration::from_secs_f64(idx as f64 / plan.hatch_rate as f64);
                tokio::time::sleep(delay).await;
            }
            loop {
                if cancel.is_cancelled() {
                    break;
                }
                let choice = dist.sample(&mut rng);
                if let Some(request) = plan.schedule.get(choice) {
                    let _permit: Option<OwnedSemaphorePermit> =
                        if let Some(throttle) = throttle.as_ref() {
                            throttle.acquire().await
                        } else {
                            None
                        };
                    if let Err(err) =
                        execute_single_request(&client, request, &plan, &metrics).await
                    {
                        error!(error = %err, "request failed");
                    }
                    if !request.think_time.is_zero() {
                        tokio::select! {
                            _ = tokio::time::sleep(request.think_time) => {}
                            _ = cancel.cancelled() => break,
                        }
                    }
                }
                if cancel.is_cancelled() {
                    break;
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
    info!(%run_id, elapsed = ?start.elapsed(), "goose run complete");
    Ok(())
}

async fn execute_single_request(
    client: &reqwest::Client,
    request: &ScheduledRequest,
    plan: &ExecutionPlan,
    metrics: &Metrics,
) -> Result<()> {
    let mut url = plan.target.clone();
    if !url.ends_with('/') && !request.path.starts_with('/') {
        url.push('/');
    }
    url.push_str(request.path.trim_start_matches('/'));
    let mut builder = client.request(request.method.clone(), &url);
    if !plan.global_headers.is_empty() || !request.headers.is_empty() {
        let mut headers = HeaderMap::new();
        for (key, value) in &plan.global_headers {
            headers.insert(key.parse()?, value.parse()?);
        }
        for (key, value) in &request.headers {
            headers.insert(key.parse()?, value.parse()?);
        }
        builder = builder.headers(headers);
    }
    if !request.query.is_empty() {
        builder = builder.query(&request.query);
    }
    let mut sent_bytes = 0u64;
    if let Some(body) = &request.body {
        sent_bytes = body.len() as u64;
        builder = builder.body(body.clone());
    }
    let start = Instant::now();
    let response = builder.send().await;
    match response {
        Ok(response) => {
            let status = response.status();
            let bytes = response.bytes().await.unwrap_or_default();
            metrics.record(
                start.elapsed(),
                status.is_success(),
                sent_bytes,
                bytes.len() as u64,
            );
            Ok(())
        }
        Err(err) => {
            metrics.record(start.elapsed(), false, sent_bytes, 0);
            Err(anyhow!(err))
        }
    }
}
