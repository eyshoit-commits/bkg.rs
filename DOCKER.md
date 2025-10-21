# bkg.rs - Docker Deployment

## 🐳 Docker Compose Schnellstart

### Voraussetzungen

- Docker installiert
- Docker Compose installiert
- Modelle heruntergeladen (siehe unten)

### Modelle herunterladen

```bash
./devops/scripts/download-models.sh
```

Dies lädt herunter:
- **Chat-Modell**: Qwen2-0.5B-Instruct (ca. 350MB)
- **Embedding-Modell**: all-MiniLM-L6-v2 (ca. 22MB)

### Starten mit Docker Compose

```bash
./devops/scripts/docker-start.sh
```

Oder manuell:

```bash
docker compose -f devops/docker/docker-compose.yml up --build
```

### Zugriff

- **Frontend**: http://localhost:43117
- **API**: http://localhost:43119
- **Health Check**: `curl http://localhost:43119/health`

## 📋 Docker Compose Konfiguration

Die `devops/docker/docker-compose.yml` definiert:

| Service | Port | Beschreibung |
|---------|------|-------------|
| bkg | 43117 | Frontend (Angular) |
| bkg | 43119 | Backend API (NestJS) |
| bkg | 43121 | Plugin Bus |

### Volumes

- `bkg-data`: Persistente Datenbank und Logs
- `./models`: GGUF-Modelldateien (read-only)

### Environment-Variablen

```yaml
ADMIN_PASSWORD=change-me
CHAT_MODEL_FILE=Qwen2-0.5B-Instruct-Q5_K_M.gguf
EMBEDDING_MODEL_FILE=all-MiniLM-L6-v2-ggml-model-f16.gguf
PROMPT_TEMPLATE=chatml
NODE_ENV=production
```

## 🛑 Stoppen

```bash
docker compose -f devops/docker/docker-compose.yml down
```

Mit Volume-Löschung:

```bash
docker compose -f devops/docker/docker-compose.yml down -v
```

## 📊 Logs anschauen

```bash
docker compose -f devops/docker/docker-compose.yml logs -f bkg
```

## 🔧 Neu bauen

```bash
docker compose -f devops/docker/docker-compose.yml build --no-cache
```

## 🐳 Manueller Docker Build

```bash
docker buildx build . \
  -f devops/docker/Dockerfile \
  --platform linux/amd64,linux/arm64 \
  -t bkg:latest \
  --build-arg CHAT_MODEL_FILE=Qwen2-0.5B-Instruct-Q5_K_M.gguf \
  --build-arg EMBEDDING_MODEL_FILE=all-MiniLM-L6-v2-ggml-model-f16.gguf \
  --build-arg PROMPT_TEMPLATE=chatml
```

## 📦 Image-Größe

Das Docker-Image enthält:
- Rust-Build von llmserver
- Node.js Backend (NestJS)
- Angular Frontend (kompiliert)
- Python RepoAgent
- SQLite Datenbank
- GGUF-Modelle

**Geschätzte Größe**: 2-3GB (abhängig von Modellen)

## 🔐 Sicherheit

- **Non-root User**: Container läuft als `bkg` (UID 1000)
- **Minimale Systemrechte**: Nur notwendige Pakete
- **Secrets**: `ADMIN_PASSWORD` über Umgebungsvariablen
- **Health Check**: Automatische Überwachung

## 🚀 Production Deployment

Für Production:

1. **Passwort ändern**:
   ```bash
   export ADMIN_PASSWORD="your-secure-password"
   ```

2. **Ports anpassen** (in devops/docker/docker-compose.yml):
   ```yaml
   ports:
     - "80:43117"      # Frontend
     - "8000:43119"    # API
   ```

3. **Volumes persistent machen**:
   ```bash
   docker compose -f devops/docker/docker-compose.yml up -d
   ```

4. **Monitoring einrichten**:
   ```bash
   docker compose -f devops/docker/docker-compose.yml logs -f
   ```

## 🐛 Troubleshooting

### Port bereits in Benutzung

```bash
# Finde den Container
docker ps | grep bkg

# Stoppe ihn
docker compose -f devops/docker/docker-compose.yml down
```

### Modelle nicht gefunden

```bash
# Überprüfe Modell-Verzeichnis
ls -lh models/

# Lade neu herunter
./download-models.sh
```

### Build schlägt fehl

```bash
# Neu bauen ohne Cache
docker compose -f devops/docker/docker-compose.yml build --no-cache

# Mit Verbose-Output
docker compose -f devops/docker/docker-compose.yml build --verbose
```

## 📚 Weitere Infos

- **Docker Compose Docs**: https://docs.docker.com/compose/
- **Dockerfile**: `docker/Dockerfile`
- **Entwicklung**: `DEV_SETUP.md`
- **Schnellstart**: `QUICK_START.md`

---

**Status**: ✅ Produktionsreif  
**Letztes Update**: 21.10.2025
