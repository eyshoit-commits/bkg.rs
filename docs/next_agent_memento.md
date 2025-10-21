# Phase 1 Hand-off Summary

## Completed Work
- Implemented Candle and RustyFace plugin runtimes with telemetry and log streaming via the plugin bus.
- Extended backend `apps/bkg-api` to expose `/api/plugins` lifecycle endpoints, provide log history and telemetry storage, and serve the `/ws/plugins` WebSocket gateway.
- Updated frontend `apps/bkg-web` to consume the new WebSocket service and display live telemetry/logs on the plugin dashboards.
- Added documentation update (`docs/update/v0.2-phase1.md`) and plugin configuration entries (`plugins/plugins.json`).

## Remaining Tasks
- Implement the model management REST API (`/api/models`) and connect the Admin UI workflows.
- Migrate Angular lint tooling from TSLint to ESLint so that `npm run lint` passes.
- Add automated test coverage for:
  - Backend plugin lifecycle controller, WebSocket gateway routing, and plugin bus telemetry/log persistence.
  - Candle/RustyFace plugin capability flows.
  - Frontend WebSocket client and plugin dashboard telemetry/log handling.
- Validate plugin start scripts on target platforms and ensure no compiled artifacts are committed.

## Testing Gaps
- `npm run lint` currently fails due to missing TSLint builder; must be resolved before CI is green.
- No automated tests exist yet for the new Rust crates, backend WebSocket/controller logic, or Angular components.

## Operational Notes
- Set `BKG_PLUGIN_BUS_PORT` (autodetects if unset) when launching plugins.
- Candle plugin expects JSON linear model manifests (see `plugins/candle/models/sample-linear.json`).
- RustyFace assumes 32×32 grayscale inputs for embeddings; adjust preprocessing if datasets differ.
- Angular WebSocket base URL derives from the `<meta name="bkg-api">` tag, which must be present in deployments.
- Keep `plugins/plugins.json` aligned with database records to avoid configuration mismatches.
