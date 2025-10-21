# Deployment

## Prerequisites

- Docker Engine 24+ with BuildKit and `buildx` enabled.
- Rust toolchain (stable) for local builds of `llmserver`.
- Node.js 18+ for API/frontend development.
- Python 3.11+ for RepoAgent plug-in work.

## Container Build

```
docker buildx build . \
  --platform linux/amd64,linux/arm64 \
  -t bkg:latest \
  --build-arg CHAT_MODEL_FILE=Qwen2-0.5B-Instruct-Q5_K_M.gguf \
  --build-arg EMBEDDING_MODEL_FILE=all-MiniLM-L6-v2-ggml-model-f16.gguf \
  --build-arg PROMPT_TEMPLATE=chatml
```

Build arguments control the bundled model artefacts copied into `/srv/models` inside the image. At runtime, the llmserver plug-in resolves the file names from the corresponding environment variables.

## Runtime Environment Variables

| Variable | Default | Description |
| --- | --- | --- |
| `ADMIN_PASSWORD` | (required) | Password used to seed the `admin` user (bcrypt-hashed by the apikeys plug-in). |
| `BKG_API_PORT` | dynamic | Preferred API binding port. Falls back to random available port. |
| `BKG_WEB_PORT` | `43117` | Angular dev server / static hosting port. |
| `BKG_PLUGIN_BUS_PORT` | dynamic | WebSocket bus port exposed by the API host. |
| `CHAT_MODEL_FILE` | `synthetic-chat` | Chat model file passed to llmserver. |
| `EMBEDDING_MODEL_FILE` | `synthetic-embed` | Embedding model file passed to llmserver. |
| `PROMPT_TEMPLATE` | `assistant` | Prompt template identifier consumed by llmserver. |
| `BKG_DATABASE_PATH` | `/data/bkg.db` | SQLite database path shared by host and plug-ins. |

## Launching the Container

```
docker run --rm \
  -p 43117:43117 \
  -p 43119:43119 \
  -e ADMIN_PASSWORD="change-me-now" \
  -e BKG_WEB_PORT=43117 \
  -e BKG_API_PORT=43119 \
  -e BKG_PLUGIN_BUS_PORT=43121 \
  bkg:latest
```

The supervisor inside the container boots the NestJS API. Plug-ins are launched as child processes using the definitions in `plugins/plugins.json`. Logs from each plug-in surface in the admin UI and the container stdout.

## Local Development

1. **Install dependencies**
   ```
   (cd apps/bkg-api && npm install)
   (cd apps/bkg-web && npm install)
   ```
2. **Run the API**
   ```
   cd apps/bkg-api
   npm run start:dev
   ```
3. **Run the frontend**
   ```
   cd apps/bkg-web
   npm start
   ```
4. **Start plug-ins manually** (optional during development)
   ```
   BKG_PLUGIN_BUS_PORT=43121 BKG_PLUGIN_NAME=llmserver plugins/llmserver/start.sh
   BKG_PLUGIN_BUS_PORT=43121 BKG_PLUGIN_NAME=repoagent plugins/repoagent/start.sh
   ADMIN_PASSWORD=devpass BKG_PLUGIN_BUS_PORT=43121 BKG_PLUGIN_NAME=apikeys plugins/apikeys/start.sh
   ```

Ensure `/data` exists (or override `BKG_DATABASE_PATH`) before launching the API locally so SQLite can create the database file.
