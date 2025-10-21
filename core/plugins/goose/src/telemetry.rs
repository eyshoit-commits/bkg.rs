use std::time::Duration;

use anyhow::Result;
use sysinfo::{CpuRefreshKind, RefreshKind, System, SystemExt};
use tokio::sync::broadcast;
use tracing::warn;

use crate::{bus::PluginBusClient, manager::LoadTestManager};

pub async fn run_telemetry_loop(
    bus: PluginBusClient,
    manager: LoadTestManager,
    shutdown: broadcast::Sender<()>,
) -> Result<()> {
    let mut system = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(),
    );
    let mut ticker = tokio::time::interval(Duration::from_secs(10));
    let mut shutdown_rx = shutdown.subscribe();
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                system.refresh_cpu();
                system.refresh_memory();
                let cpu = system.global_cpu_info().cpu_usage() as f64;
                let mem_bytes = system.used_memory() * 1024;
                match manager.status().await {
                    Ok(status) => {
                        let entries = status.metrics.total_requests;
                        if let Err(err) = bus.send_telemetry(cpu, mem_bytes, entries).await {
                            warn!(error = %err, "failed to push telemetry");
                        }
                    }
                    Err(err) => {
                        warn!(error = %err, "failed to collect status for telemetry");
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }
    Ok(())
}
