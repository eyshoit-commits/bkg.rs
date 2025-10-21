# 📁 Finale Projektstruktur v0.2

## Überblick

```
bkg.rs/
├── core/                          # ✅ Hauptanwendung
│   ├── backend/
│   │   └── gateway/               # NestJS API (Port 43119)
│   │       ├── src/
│   │       ├── dist/
│   │       └── package.json
│   ├── frontend/
│   │   └── admin-ui/              # Angular Web UI
│   │       ├── src/
│   │       ├── dist/
│   │       └── package.json
│   ├── plugins/                   # ✅ Alle Plugins
│   │   ├── apikeys/               # Auth Plugin (Node.js)
│   │   ├── brainml/               # Hybrid Search (Rust)
│   │   ├── candle/                # ML Inference (Rust)
│   │   ├── llmserver/             # LLM Server (Rust)
│   │   ├── repoagent/             # Code Analysis (Python)
│   │   ├── rustyface/             # Face Recognition (Rust)
│   │   └── plugins.json           # Plugin Registry
│   ├── config/                    # Zentralisierte Configs
│   └── database/                  # SQLite Storage
├── devops/                        # ✅ Deployment & Ops
│   ├── docker/
│   │   ├── Dockerfile             # Multi-stage Build
│   │   ├── docker-compose.yml     # Container Orchestration
│   │   ├── supervisord.conf       # Process Management
│   │   └── start.sh               # Entrypoint
│   └── scripts/                   # Helper Scripts
├── docs/                          # ✅ Dokumentation
│   ├── architecture/              # Design Docs
│   ├── implementation/            # Implementation Guides
│   ├── update/                    # Version Updates
│   └── CORE_STRUCTURE.md          # This File
├── models/                        # ✅ GGUF Modelldateien
├── .devcontainer/                 # VS Code Dev Container
└── data/                          # Runtime Data (SQLite, Logs)
```

## ✅ Cleanup durchgeführt

| Gelöscht | Grund |
|----------|-------|
| `plugins/` (alt) | Umgezogen zu `core/plugins/` |
| `scripts/` (alt) | Umgezogen zu `devops/scripts/` |
| `devops/.devcontainer/` | Doppelt (auch in `.devcontainer/`) |

## 🎯 Ports

| Service | Port | Status |
|---------|------|--------|
| Web UI | 43119 | ✅ Läuft |
| API | 43119 | ✅ Läuft |
| Plugin Bus | 43121 | ✅ Läuft |

## 📦 Build & Run

```bash
# Docker Build
docker compose -f devops/docker/docker-compose.yml build

# Docker Run
docker compose -f devops/docker/docker-compose.yml up -d

# Zugriff
curl http://localhost:43119/health
```

## 🔄 Git Status

- **Branch**: `codex/implement-modular-plugin-system-kqgaao`
- **Letzter Commit**: `6bc5c85` (cleanup: remove duplicate directories)
- **Status**: ✅ Sauber & bereit zum Mergen

---

**Nächste Schritte:**
1. ✅ Zu `main` mergen
2. 🔧 Plugins debuggen
3. 📝 Login implementieren
