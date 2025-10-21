# Goose Plug-in Runtime

## Overview

The Goose plug-in provides HTTP load-testing capabilities inside the bkg plug-in architecture. It wraps the Goose runtime in a standalone process that is launched and supervised by the gateway, exposing both REST controls and plug-in bus capabilities.

* **Location:** `core/plugins/goose/`
* **Primary entrypoint:** `core/plugins/goose/src/main.rs`
* **Configuration schema:** published to the plug-in bus and consumed by the Admin UI (`GooseConfigComponent`)
* **Gateway surface:** `core/backend/gateway/src/goose/goose.controller.ts`
* **Admin UI:** `core/frontend/admin-ui/src/app/features/plugins/goose/goose-config.component.{ts,html,css}`

## Capabilities

| Capability | Description | REST proxy |
|------------|-------------|------------|
| `goose.run` | Start a load test with the provided scenario | `POST /api/goose/run` |
| `goose.stop` | Request graceful shutdown of the active load test | `POST /api/goose/stop` |
| `goose.status` | Retrieve live status, metrics, and effective settings | `GET /api/goose/status` |
| `goose.history` | Fetch recent run summaries (bounded by `maxHistory`) | `GET /api/goose/history` |

All capability invocations are streamed through the plug-in bus, allowing lifecycle automation and UI integration without direct process coupling.

## Runtime Composition

* **Axum HTTP server:** Serves health, status, history, run, and stop endpoints for internal supervision and future automations.
* **Plugin bus bridge:** `bus.rs` handles registration, log emission, heartbeat, telemetry, and capability dispatch.
* **Load manager:** `manager.rs` orchestrates Goose executions, enforces single-run semantics, records metrics, and trims history.
* **Telemetry loop:** `telemetry.rs` captures CPU/memory data and total request counts via `sysinfo`, publishing them over the bus.
* **History persistence:** `manager.rs` stores bounded run summaries on disk (`history.json`, overridable via `BKG_PLUGIN_HISTORY_PATH`).
* **Configuration loader:** `config.rs` merges runtime overrides (`config.runtime.json`) with the embedded defaults from `config.json`, while emitting a JSON schema for UI forms (startup delay, graceful stop, throttle, cookie/redirect toggles).

## Configuration Flow

1. The gateway writes an updated `config.runtime.json` whenever operators adjust settings in the Admin UI.
2. The plug-in reads `BKG_PLUGIN_CONFIG_PATH` on start to locate that runtime config; if unavailable it falls back to the embedded default.
3. `config.rs` merges missing properties with defaults and advertises the schema to the plug-in bus during registration, enabling rich form rendering.
4. The Angular component binds to the schema-backed settings, offering controls for targets, users, hatch rates, request headers, query parameters, think time, TLS verification, startup delay, graceful stop, throttle, sticky cookies, redirect handling, and history retention.
5. When operators click **Run** or **Stop**, the Admin UI invokes the gateway REST proxies, which forward capabilities to the running plug-in.

### Key configuration switches

| Field | Purpose | Default |
|-------|---------|---------|
| `startupTimeSeconds` | Delay before spawning the first virtual user (lets dependencies warm up). | `5` |
| `gracefulStopSeconds` | Grace period before forcefully cancelling active requests on stop. | `10` |
| `throttleRps` | Global request-per-second cap (`0`/`null` disables throttling). | `0` |
| `stickyCookies` | Enables persistent cookies per virtual user. | `true` |
| `followRedirects` | Allows HTTP redirects to be followed. | `true` |
| `maxHistory` | Number of run summaries persisted to `history.json`. | `20` |

## Files of Interest

* `core/plugins/goose/src/main.rs` — bootstraps the runtime, wiring HTTP routes, telemetry, and bus event handling.
* `core/plugins/goose/src/manager.rs` — manages load-test lifecycle, metrics collection, throttled execution plan derivation, and history retention.
* `core/plugins/goose/src/models.rs` — shared DTOs for requests, responses, metrics, and status reporting.
* `core/plugins/goose/src/telemetry.rs` — periodic telemetry sampling loop.
* `core/plugins/goose/src/bus.rs` — WebSocket integration with the plug-in bus.
* `core/plugins/goose/config.json` — embedded default configuration (copied into the runtime schema).
* `core/plugins/goose/src/config.rs` — merges runtime overrides with defaults and emits the JSON schema (including throttle, graceful stop, cookie, and redirect options).
* `core/frontend/admin-ui/src/app/features/plugins/goose/goose-config.component.ts` — Angular configuration and control panel.
* `core/backend/gateway/src/goose/goose.controller.ts` — API endpoints for Auth-guarded Goose control.

## Admin UI Integration

The Goose tab in the Admin UI exposes:

* Live status cards showing run state, duration, throttled effective settings (startup delay, graceful stop, throttle, cookies, redirects), and request metrics.
* A schedule editor with per-request method, path, headers, query parameters, body, weight, and think time controls.
* Global header management and toggles for TLS verification, sticky cookies, redirect handling, startup delay, graceful stop, throttle, and history retention.
* Action buttons to trigger **Run**, **Stop**, and refresh operations when the plug-in is running.
* History tables listing previous runs with run IDs, timings, and aggregated metrics.

All UI interactions persist settings through the gateway back to `config.runtime.json`, ensuring restarts honour the latest configuration.

## Outstanding Tasks

* Generate `core/plugins/goose/Cargo.lock` using `cargo generate-lockfile --manifest-path core/plugins/goose/Cargo.toml` once network access to crates.io is available. (Current attempts fail with HTTP 403 due to proxy restrictions.)
* Execute `cargo check --manifest-path core/plugins/goose/Cargo.toml` and integration tests after the lockfile is present to validate the runtime end-to-end.

