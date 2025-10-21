use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseScheduleEntry {
    pub name: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub path: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub query: HashMap<String, String>,
    #[serde(default)]
    pub think_time_ms: Option<u64>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseSettings {
    #[serde(rename = "defaultTarget")]
    pub default_target: String,
    #[serde(default = "default_users", rename = "users")]
    pub users: u32,
    #[serde(default = "default_hatch_rate", rename = "hatchRate")]
    pub hatch_rate: u32,
    #[serde(default = "default_run_time", rename = "runTimeSeconds")]
    pub run_time_seconds: u64,
    #[serde(default = "default_timeout", rename = "timeoutSeconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_startup_time", rename = "startupTimeSeconds")]
    pub startup_time_seconds: u64,
    #[serde(default = "default_graceful_stop", rename = "gracefulStopSeconds")]
    pub graceful_stop_seconds: u64,
    #[serde(default, rename = "throttleRps")]
    pub throttle_rps: Option<u32>,
    #[serde(default, rename = "globalHeaders")]
    pub global_headers: HashMap<String, String>,
    #[serde(default = "default_verify_tls", rename = "verifyTls")]
    pub verify_tls: bool,
    #[serde(default = "default_sticky_cookies", rename = "stickyCookies")]
    pub sticky_cookies: bool,
    #[serde(default = "default_follow_redirects", rename = "followRedirects")]
    pub follow_redirects: bool,
    #[serde(default = "default_max_history", rename = "maxHistory")]
    pub max_history: usize,
    #[serde(default)]
    pub schedule: Vec<GooseScheduleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GooseRunRequest {
    #[serde(default, rename = "target")]
    pub target: Option<String>,
    #[serde(default, rename = "users")]
    pub users: Option<u32>,
    #[serde(default, rename = "hatchRate")]
    pub hatch_rate: Option<u32>,
    #[serde(default, rename = "runTimeSeconds")]
    pub run_time_seconds: Option<u64>,
    #[serde(default, rename = "timeoutSeconds")]
    pub timeout_seconds: Option<u64>,
    #[serde(default, rename = "globalHeaders")]
    pub global_headers: Option<HashMap<String, String>>,
    #[serde(default, rename = "verifyTls")]
    pub verify_tls: Option<bool>,
    #[serde(default, rename = "startupTimeSeconds")]
    pub startup_time_seconds: Option<u64>,
    #[serde(default, rename = "gracefulStopSeconds")]
    pub graceful_stop_seconds: Option<u64>,
    #[serde(default, rename = "throttleRps")]
    pub throttle_rps: Option<u32>,
    #[serde(default, rename = "stickyCookies")]
    pub sticky_cookies: Option<bool>,
    #[serde(default, rename = "followRedirects")]
    pub follow_redirects: Option<bool>,
    #[serde(default)]
    pub schedule: Option<Vec<GooseScheduleEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GooseRunState {
    Idle,
    Starting,
    Running,
    Stopping,
    Completed,
    Failed,
}

impl Default for GooseRunState {
    fn default() -> Self {
        GooseRunState::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GooseMetricsSnapshot {
    #[serde(rename = "totalRequests")]
    pub total_requests: u64,
    #[serde(rename = "successRequests")]
    pub success_requests: u64,
    #[serde(rename = "failedRequests")]
    pub failed_requests: u64,
    #[serde(rename = "requestsPerSecond")]
    pub requests_per_second: f64,
    #[serde(rename = "averageLatencyMs")]
    pub average_latency_ms: f64,
    #[serde(rename = "p95LatencyMs")]
    pub p95_latency_ms: f64,
    #[serde(rename = "p99LatencyMs")]
    pub p99_latency_ms: f64,
    #[serde(rename = "bytesSent")]
    pub bytes_sent: u64,
    #[serde(rename = "bytesReceived")]
    pub bytes_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GooseRunResponse {
    pub status: GooseRunState,
    #[serde(rename = "runId")]
    pub run_id: Option<Uuid>,
    #[serde(rename = "message")]
    pub message: String,
    #[serde(rename = "metrics")]
    pub metrics: GooseMetricsSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseStatus {
    pub status: GooseRunState,
    #[serde(rename = "runId")]
    pub run_id: Option<Uuid>,
    #[serde(rename = "startedAt")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(rename = "finishedAt")]
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(rename = "durationSeconds")]
    pub duration_seconds: f64,
    #[serde(rename = "settings")]
    pub settings: GooseEffectiveSettings,
    #[serde(rename = "metrics")]
    pub metrics: GooseMetricsSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseEffectiveSettings {
    #[serde(rename = "target")]
    pub target: String,
    #[serde(rename = "users")]
    pub users: u32,
    #[serde(rename = "hatchRate")]
    pub hatch_rate: u32,
    #[serde(rename = "runTimeSeconds")]
    pub run_time_seconds: u64,
    #[serde(rename = "timeoutSeconds")]
    pub timeout_seconds: u64,
    #[serde(rename = "verifyTls")]
    pub verify_tls: bool,
    #[serde(default = "default_startup_time", rename = "startupTimeSeconds")]
    pub startup_time_seconds: u64,
    #[serde(default = "default_graceful_stop", rename = "gracefulStopSeconds")]
    pub graceful_stop_seconds: u64,
    #[serde(default, rename = "throttleRps")]
    pub throttle_rps: Option<u32>,
    #[serde(default = "default_sticky_cookies", rename = "stickyCookies")]
    pub sticky_cookies: bool,
    #[serde(default = "default_follow_redirects", rename = "followRedirects")]
    pub follow_redirects: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseRunSummary {
    #[serde(rename = "runId")]
    pub run_id: Uuid,
    #[serde(rename = "startedAt")]
    pub started_at: DateTime<Utc>,
    #[serde(rename = "finishedAt")]
    pub finished_at: DateTime<Utc>,
    #[serde(rename = "durationSeconds")]
    pub duration_seconds: f64,
    #[serde(rename = "status")]
    pub status: GooseRunState,
    #[serde(rename = "settings")]
    pub settings: GooseEffectiveSettings,
    #[serde(rename = "metrics")]
    pub metrics: GooseMetricsSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GooseRunSummaryList {
    pub runs: Vec<GooseRunSummary>,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_weight() -> u32 {
    1
}

fn default_users() -> u32 {
    10
}

fn default_hatch_rate() -> u32 {
    5
}

fn default_run_time() -> u64 {
    60
}

fn default_timeout() -> u64 {
    30
}

fn default_startup_time() -> u64 {
    5
}

fn default_graceful_stop() -> u64 {
    10
}

fn default_sticky_cookies() -> bool {
    true
}

fn default_follow_redirects() -> bool {
    true
}

fn default_verify_tls() -> bool {
    true
}

fn default_max_history() -> usize {
    20
}
