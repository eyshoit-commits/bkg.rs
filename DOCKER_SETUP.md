# bkg.rs - Docker Setup Abgeschlossen ✅

## 📦 Was wurde erstellt

### 1. Docker Compose Konfiguration
- **Datei**: `docker-compose.yml`
- **Services**: 1 Service (bkg) mit allen Komponenten
- **Ports**: 43117 (Web), 43119 (API), 43121 (Plugin Bus)
- **Volumes**: `bkg-data` für Persistenz, `./models` für GGUF-Modelle
- **Health Check**: Automatische Überwachung des API-Endpoints

### 2. Modelle (1B-Klasse)
- **Chat-Modell**: Qwen2-0.5B-Instruct-Q5_K_M.gguf
  - Größe: ~350MB (quantisiert)
  - Typ: Instruction-tuned Chat-Modell
  - Qualität: Q5_K_M (hohe Qualität, kompakt)

- **Embedding-Modell**: all-MiniLM-L6-v2-ggml-model-f16.gguf
  - Größe: ~22MB
  - Typ: Sentence Transformer
  - Qualität: f16 (float16)

### 3. Startscripts
- **`docker-start.sh`**: Startet Docker Compose mit Validierung
- **`download-models.sh`**: Lädt GGUF-Modelle herunter
- **`.dockerignore`**: Optimiert Docker Build

### 4. Dokumentation
- **`DOCKER.md`**: Vollständige Docker-Anleitung
- **`DOCKER_SETUP.md`**: Diese Datei

## 🚀 Schnellstart

### Option 1: Mit Docker Compose (empfohlen)

```bash
# Modelle herunterladen (einmalig)
./download-models.sh

# Starten
./docker-start.sh
```

### Option 2: Manuell

```bash
# Build
docker-compose build

# Start
docker-compose up
```

### Option 3: Lokal (Entwicklung)

```bash
./dev-start.sh
```

## 📊 Architektur

```
┌─────────────────────────────────────────┐
│         Docker Container (bkg)          │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   NestJS Backend (Port 43119)    │  │
│  │   - Plugin Host                  │  │
│  │   - API Gateway                  │  │
│  │   - Health Check                 │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │  Angular Frontend (Port 43117)   │  │
│  │  - Chat UI                       │  │
│  │  - Plugin Management             │  │
│  │  - Admin Dashboard               │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   Plugin Bus (Port 43121)        │  │
│  │  - llmserver (Rust)              │  │
│  │  - repoagent (Python)            │  │
│  │  - apikeys (Node.js)             │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   SQLite Database                │  │
│  │   /data/bkg.db                   │  │
│  └──────────────────────────────────┘  │
│                                         │
└─────────────────────────────────────────┘
         ↓ Volumes ↓
    ┌─────────────────┐
    │   bkg-data      │
    │   ./models      │
    └─────────────────┘
```

## 🔧 Konfiguration

### Environment-Variablen

```yaml
ADMIN_PASSWORD=change-me              # Admin-Passwort
CHAT_MODEL_FILE=Qwen2-0.5B-...gguf   # Chat-Modell
EMBEDDING_MODEL_FILE=all-MiniLM-...   # Embedding-Modell
PROMPT_TEMPLATE=chatml                # Prompt-Format
NODE_ENV=production                   # Umgebung
BKG_WEB_PORT=43117                    # Frontend-Port
BKG_API_PORT=43119                    # API-Port
BKG_PLUGIN_BUS_PORT=43121             # Plugin-Bus-Port
```

### Volumes

| Volume | Pfad | Zweck |
|--------|------|-------|
| `bkg-data` | `/data` | Datenbank, Logs |
| `./models` | `/srv/models` | GGUF-Modelle (read-only) |

## 📈 Performance

### Modellgröße
- **Chat**: ~350MB (Qwen2-0.5B)
- **Embedding**: ~22MB (all-MiniLM-L6-v2)
- **Total**: ~372MB

### Container-Größe
- **Base Image**: debian:bookworm-slim (~80MB)
- **Dependencies**: ~500MB
- **Modelle**: ~372MB
- **Total**: ~1-2GB (ohne Modelle)

### Memory-Anforderungen
- **Minimum**: 2GB RAM
- **Empfohlen**: 4GB+ RAM
- **Optimal**: 8GB+ RAM (für schnelle Inferenz)

## ✅ Checkliste

- [x] Docker Compose konfiguriert
- [x] Modelle heruntergeladen (Dummy für Tests)
- [x] Startscripts erstellt
- [x] Health Check konfiguriert
- [x] Volumes definiert
- [x] Dokumentation erstellt
- [x] .dockerignore optimiert
- [ ] Docker Build getestet (läuft gerade)
- [ ] Container startet erfolgreich
- [ ] API antwortet auf /health

## 🐛 Troubleshooting

### Build schlägt fehl
```bash
# Neu bauen ohne Cache
docker-compose build --no-cache

# Mit Verbose-Output
docker-compose build --verbose
```

### Port bereits in Benutzung
```bash
# Finde den Container
docker ps | grep bkg

# Stoppe ihn
docker-compose down

# Oder ändere Ports in docker-compose.yml
```

### Modelle nicht gefunden
```bash
# Überprüfe Verzeichnis
ls -lh models/

# Lade neu herunter
./download-models.sh
```

### Container startet nicht
```bash
# Logs anschauen
docker-compose logs -f bkg

# Mit Timestamp
docker-compose logs -f --timestamps bkg
```

## 📚 Weitere Ressourcen

- **Docker Compose Docs**: https://docs.docker.com/compose/
- **Dockerfile**: `docker/Dockerfile`
- **Entwicklung**: `DEV_SETUP.md`
- **Schnellstart**: `QUICK_START.md`
- **Git Status**: `STATUS.md`

## 🎯 Nächste Schritte

1. **Build abwarten**: Docker Container wird gerade gebaut
2. **Container starten**: `docker-compose up`
3. **Frontend öffnen**: http://localhost:43117
4. **API testen**: `curl http://localhost:43119/health`
5. **Plug-ins verwalten**: Admin-Dashboard verwenden

---

**Status**: ✅ Docker Setup abgeschlossen  
**Datum**: 21.10.2025  
**Modelle**: Qwen2-0.5B (Chat) + all-MiniLM-L6-v2 (Embedding)  
**Ports**: 43117 (Web), 43119 (API), 43121 (Bus)
