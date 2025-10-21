# Plug-in Development

## Shared Contract

All plug-ins integrate with the NestJS host through the WebSocket plug-in bus. Implementations must:

1. Connect to `ws://<host>:<BKG_PLUGIN_BUS_PORT>`.
2. Immediately send a `register` payload containing:
   - `plugin` – unique name (matches directory name and configuration entry).
   - `port` – listening port or `'internal'` for headless services.
   - `capabilities` – array of capability identifiers (e.g. `llm.chat`).
   - `meta` – optional metadata advertised to the control plane.
3. Emit periodic `health` messages every 10 seconds to signal readiness.
4. Handle `request` messages and reply with `response` envelopes using the provided `requestId`.
5. Stream logs back to the host via `log` envelopes for observability in the admin UI.

The bus protocol is intentionally lightweight JSON, enabling clients in any language.

## Capability Naming

Capabilities map directly to features exposed through the API gateway:

- `llm.chat`, `llm.embed` – conversational and embedding operations.
- `repo.analyze`, `repo.patch` – repository analytics and patch application.
- `auth.*` – authentication, key lifecycle, and scope validation.

When introducing a new capability, add it to `PluginCapability` (`core/backend/gateway/src/plugins/plugin.types.ts`) to ensure type safety.

## Plug-in Configuration Lifecycle

Plug-in configuration is stored in SQLite (`plugins` table) and mirrored to `core/plugins/plugins.json` for human inspection. The admin UI writes updates through `POST /admin/plugins/:name/config`, which:

1. Persists the config via `PluginService.saveConfig`.
2. Synchronises the in-memory state and bus metadata.
3. Allows restarting the plug-in to pick up changes.

Each plug-in directory should include:

- `config.json` – plug-in specific configuration (parsed by the plug-in itself).
- `start.sh` – executable wrapper invoked by the host to start the plug-in.
- Runtime assets (Cargo project, Python package, or Node project).

## Control Centre Navigation

The Angular admin UI exposes a unified control centre under `core/frontend/admin-ui/src/app/features/plugins/`. The structure maps one-to-one to the plug-in catalog and enables quick access to individual dashboards:

- `plugin-list/` renders `/plugins` with the complete inventory, lifecycle buttons, and capability badges.
- `plugin-dashboard/` powers `/plugins/:name`, combining log streaming, configuration editing, and curated feature descriptions for every plug-in (brainml, candle, rustyface, llmserver, repoagent, apikeys).

Every dashboard surfaces the capabilities advertised over the plug-in bus and wires the lifecycle controls (`start`, `stop`, `restart`) back to the NestJS host. Feature cards summarise the operational responsibilities (e.g. BrainML hybrid search, Candle inference) so operators immediately understand the levers available. Log streaming and configuration editors share the same layout across all plug-ins, which keeps hot-swapping predictable when new services join the platform.

## Plug-in Reference Implementations

### llmserver (Rust)

- Uses Axum to expose OpenAI-compatible `/v1/chat/completions` and `/v1/embeddings` endpoints.
- Generates deterministic embeddings and templated chat responses as a stand-in until real models are mounted under `/srv/models`.
- Registers capabilities `llm.chat` and `llm.embed` and relays log + health status.
- Build command: `cargo build --release` (handled by `start.sh`).

### repoagent (Python)

- FastAPI service delivering repository scanning and patch application operations.
- Uses `websockets` for bus connectivity and `aiofiles` for non-blocking file access.
- Accepts JSON payloads describing repository paths and returns structured summaries.
- Runs inside a per-plug-in virtual environment initialised by `start.sh`.

### apikeys (NodeJS)

- Headless plug-in responsible for users, sessions, and API keys.
- Seeds the `admin` user using the `ADMIN_PASSWORD` environment variable and stores hashes with bcrypt.
- Validates scopes by matching request paths to required capabilities (chat, embeddings, admin).
- Issues session tokens (24-hour TTL) and API keys using cryptographically secure random bytes.

## Adding New Plug-ins

1. Create a new directory under `core/plugins/<name>` with executable `start.sh` and configuration files.
2. Extend `core/plugins/plugins.json` with the plug-in definition (entrypoint, capabilities, autostart flag).
3. Implement the bus handshake, log streaming, and capability handlers.
4. Document the plug-in and add automated coverage where applicable.
5. Update the Angular admin UI if new capabilities need dedicated controls.
