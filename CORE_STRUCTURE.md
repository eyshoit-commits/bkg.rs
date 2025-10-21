# bkg.rs v0.2 - Core Architecture

**Status**: SIMPLIFIED & OPTIMIZED  
**Datum**: 2025-10-21

---

## 🎯 Vereinfachte Core-Struktur

```
bkg.rs/
│
├── core/                             # ALLES WICHTIGE HIER
│   ├── backend/                      # Backend Services
│   │   ├── gateway/                  # REST API + WebSocket
│   │   ├── ml-engine/                # ML Services
│   │   ├── auth/                     # JWT + API Keys
│   │   ├── vector/                   # Vector Store
│   │   └── shared/                   # Shared Types
│   │
│   ├── frontend/                     # Frontend
│   │   └── admin-ui/                 # Angular 18
│   │
│   ├── plugins/                      # Plugins
│   │   ├── brainml/
│   │   ├── candle/
│   │   ├── rustyface/
│   │   ├── llmserver/
│   │   ├── repoagent/
│   │   ├── apikeys/
│   │   └── plugins.json
│   │
│   ├── database/                     # Database
│   │   ├── migrations/
│   │   └── schema.sql
│   │
│   └── config/                       # Configuration
│       ├── docker-compose.yml
│       ├── Cargo.workspace.toml
│       └── package.json
│
├── devops/                           # DevOps & Deployment
│   ├── docker/
│   │   ├── Dockerfile
│   │   └── supervisord.conf
│   ├── .devcontainer/
│   │   ├── devcontainer.json
│   │   └── post-create.sh
│   ├── .github/workflows/
│   │   ├── ci-cleanup.yml
│   │   ├── ci-build.yml
│   │   └── cd-deploy.yml
│   └── scripts/
│       ├── cleanup-workspace.sh
│       ├── setup-dev.sh
│       └── deploy.sh
│
├── docs/                             # Documentation
│   ├── README.md
│   ├── INDEX.md
│   ├── CODEX_SYSTEM_PROMPT.md
│   ├── FOLDER_STRUCTURE_V0.2.md
│   ├── CORE_STRUCTURE.md
│   ├── architecture/
│   ├── implementation/
│   ├── update/
│   └── changelog/
│
├── models/                           # ML Models
│   ├── all-MiniLM-L6-v2-ggml-model-f16.gguf
│   └── Qwen2-0.5B-Instruct-Q5_K_M.gguf
│
├── .gitignore
├── README.md
├── LICENSE
│
└── [Approval & Guide Dateien]
    ├── APPROVAL.md
    ├── APPROVAL_COMBINED.json
    ├── DOWNLOAD_GUIDE.md
    ├── PATCH_TROUBLESHOOTING.md
    ├── V0.2_IMPLEMENTATION_GUIDE.md
    └── FINAL_SUMMARY.md
```

---

## 🏗️ Core Backend Services

### `core/backend/gateway/`
```
gateway/
├── src/
│   ├── main.rs
│   ├── routes/
│   │   ├── plugins.rs        # GET /api/plugins
│   │   ├── models.rs         # GET /api/models
│   │   ├── admin.rs          # GET /admin/*
│   │   └── auth.rs           # POST /auth/*
│   ├── ws/
│   │   ├── mod.rs            # WebSocket Hub
│   │   └── handlers.rs       # Event Handlers
│   ├── middleware/
│   ├── config.rs
│   └── lib.rs
├── Cargo.toml
└── tests/
```

**Endpoints:**
- `GET /api/plugins` - List all plugins
- `POST /api/plugins/{id}/start` - Start plugin
- `WS /ws/plugins` - WebSocket stream

---

### `core/backend/ml-engine/`
```
ml-engine/
├── src/
│   ├── lib.rs
│   ├── candle_plugin.rs      # Hugging Face Candle
│   ├── rustyface_plugin.rs   # Face Recognition
│   ├── brainml_bridge.rs     # BrainML Integration
│   └── models/
├── Cargo.toml
└── tests/
```

**Capabilities:**
- `candle.model.load` - Load ML model
- `rustyface.face.detect` - Detect faces
- `brainml.index` - Index documents

---

### `core/backend/auth/`
```
auth/
├── src/
│   ├── lib.rs
│   ├── jwt.rs                # JWT Token Management
│   ├── api_keys.rs           # API Key Storage & Rotation
│   ├── permissions.rs        # Permission Checking
│   └── roles.rs              # Role Definitions
├── Cargo.toml
└── tests/
```

**Features:**
- JWT (HS256/RS256)
- API Key Rotation
- Role-based Access Control (RBAC)
- Permission Checking

---

### `core/backend/vector/`
```
vector/
├── src/
│   ├── lib.rs
│   ├── collections.rs        # Vector Collections
│   ├── snapshots.rs          # Backup/Restore
│   └── queries.rs            # Vector Search
├── Cargo.toml
└── migrations/
```

**Features:**
- Vector Indexing
- Collections Management
- Snapshot/Restore
- Vector Search

---

### `core/backend/shared/`
```
shared/
├── src/
│   ├── lib.rs
│   ├── types.rs              # DTOs & Models
│   ├── errors.rs             # Error Types
│   ├── config.rs             # Configuration
│   ├── telemetry.rs          # Metrics & Logging
│   └── utils.rs              # Helper Functions
├── Cargo.toml
└── tests/
```

**Exports:**
- `PluginInfo`, `PluginStatus`, `Capability`
- `ApiError`, `ValidationError`
- `Config`, `TelemetryMetrics`

---

## 🌐 Core Frontend

### `core/frontend/admin-ui/`
```
admin-ui/
├── src/app/
│   ├── app.routes.ts         # Main Routing
│   │
│   ├── core/
│   │   ├── services/
│   │   │   ├── plugin-api.service.ts
│   │   │   ├── ws.service.ts
│   │   │   ├── auth.service.ts
│   │   │   └── telemetry.service.ts
│   │   └── guards/
│   │       └── auth.guard.ts
│   │
│   ├── shared/
│   │   ├── components/
│   │   │   ├── plugin-header.component.ts
│   │   │   ├── plugin-stats.component.ts
│   │   │   ├── plugin-logs.component.ts
│   │   │   └── status-badge.component.ts
│   │   └── pipes/
│   │       ├── uptime.pipe.ts
│   │       └── bytes.pipe.ts
│   │
│   ├── features/
│   │   ├── dashboard/
│   │   │   └── dashboard.component.ts
│   │   └── plugins/
│   │       ├── plugins.routes.ts
│   │       ├── brainml-dashboard.component.ts
│   │       ├── candle-dashboard.component.ts
│   │       ├── rustyface-dashboard.component.ts
│   │       ├── llmserver-dashboard.component.ts
│   │       ├── repoagent-dashboard.component.ts
│   │       └── apikeys-dashboard.component.ts
│   │
│   └── stores/
│       ├── plugin.store.ts
│       ├── auth.store.ts
│       └── telemetry.store.ts
│
├── package.json
├── angular.json
├── tsconfig.json
└── tailwind.config.js
```

**Routes:**
- `/` - Dashboard
- `/plugins` - Plugin List
- `/plugins/brainml` - BrainML Dashboard
- `/plugins/candle` - Candle Dashboard
- `/plugins/rustyface` - RustyFace Dashboard
- `/plugins/llmserver` - LLMServer Dashboard
- `/plugins/repoagent` - RepoAgent Dashboard
- `/plugins/apikeys` - APIKeys Dashboard
- `/settings` - Settings
- `/users` - User Management
- `/api-keys` - API Key Management

---

## 🔌 Core Plugins

### `core/plugins/`
```
plugins/
├── brainml/
│   ├── src/
│   ├── Cargo.toml
│   ├── config.json
│   ├── start.sh
│   └── README.md
│
├── candle/                   # NEW
│   ├── src/
│   ├── Cargo.toml
│   ├── config.json
│   ├── start.sh
│   └── README.md
│
├── rustyface/                # NEW
│   ├── src/
│   ├── Cargo.toml
│   ├── config.json
│   ├── start.sh
│   └── README.md
│
├── llmserver/
│   ├── src/
│   ├── Cargo.toml
│   ├── config.json
│   ├── start.sh
│   └── README.md
│
├── repoagent/
│   ├── repoagent/
│   ├── pyproject.toml
│   ├── config.json
│   ├── start.sh
│   └── README.md
│
├── apikeys/
│   ├── src/
│   ├── package.json
│   ├── config.json
│   ├── start.sh
│   └── README.md
│
└── plugins.json              # Plugin Registry
```

**6 Plugins:**
1. **brainml** - Indexing & Query
2. **candle** - ML Models (Hugging Face)
3. **rustyface** - Face Recognition
4. **llmserver** - Chat & Embeddings
5. **repoagent** - Code Analysis
6. **apikeys** - Authentication

---

## 💾 Core Database

### `core/database/`
```
database/
├── migrations/
│   ├── 001_init_schema.sql
│   ├── 002_add_api_keys.sql
│   ├── 003_add_models.sql
│   ├── 004_add_embeddings.sql
│   └── 005_add_analytics.sql
│
├── schema.sql                # Full Schema
│
└── seeds/
    └── initial_data.sql
```

**Tables:**
- `users` - User Management
- `api_keys` - API Key Storage
- `models` - Model Registry
- `documents` - Document Storage
- `embeddings` - Vector Embeddings
- `analytics_events` - Event Tracking

---

## ⚙️ Core Configuration

### `core/config/`
```
config/
├── docker-compose.yml        # Development
├── docker-compose.prod.yml   # Production
├── Cargo.workspace.toml      # Rust Workspace
└── package.json              # Root Node Config
```

---

## 🚀 DevOps

### `devops/docker/`
```
docker/
├── Dockerfile                # Multi-stage Build
├── supervisord.conf          # Process Management
└── start.sh                  # Entrypoint
```

### `devops/.devcontainer/`
```
.devcontainer/
├── devcontainer.json         # VSCode Config
└── post-create.sh            # Setup Script
```

### `devops/.github/workflows/`
```
.github/workflows/
├── ci-cleanup.yml            # Workspace Cleanup
├── ci-build.yml              # Build Pipeline
├── ci-test.yml               # Test Pipeline
└── cd-deploy.yml             # Deployment
```

### `devops/scripts/`
```
scripts/
├── cleanup-workspace.sh      # Workspace Cleanup
├── setup-dev.sh              # Development Setup
├── build-docker.sh           # Docker Build
└── deploy.sh                 # Deployment
```

---

## 📊 Vergleich: Struktur-Optionen

| Option | Struktur | Vorteile | Nachteile |
|--------|----------|----------|-----------|
| **v0.1a (Aktuell)** | `apps/` + `plugins/` | Einfach | Monolith, schwer zu skalieren |
| **v0.2 (Original)** | `backend/` + `frontend/` | Microservices | Komplex, viele Ordner |
| **Core (EMPFOHLEN)** | `core/` + `devops/` | Übersichtlich, modular | Mittel |

---

## 🎯 Core-Struktur Vorteile

✅ **Übersichtlich**
- Alles Wichtige in `core/`
- DevOps separiert
- Klare Trennung

✅ **Modular**
- Backend Services isoliert
- Frontend unabhängig
- Plugins austauschbar

✅ **Skalierbar**
- Microservices-Ready
- Separate Deployments
- Independent Scaling

✅ **Wartbar**
- Klare Struktur
- Konsistente Naming
- Einfach zu navigieren

---

## 🔄 Migration: v0.1a → Core

### Schritt 1: Core-Struktur erstellen
```bash
mkdir -p core/{backend,frontend,plugins,database,config}
mkdir -p devops/{docker,.devcontainer,.github/workflows,scripts}
```

### Schritt 2: Code migrieren
```bash
# Backend
cp -r apps/bkg-api/src/* core/backend/gateway/src/

# Frontend
cp -r apps/bkg-web/src/* core/frontend/admin-ui/src/

# Plugins
cp -r plugins/* core/plugins/

# Database
cp -r database/* core/database/

# DevOps
cp -r docker/* devops/docker/
cp -r .devcontainer/* devops/.devcontainer/
cp -r .github/* devops/.github/
cp -r scripts/* devops/scripts/
```

### Schritt 3: Alte Struktur entfernen
```bash
rm -rf apps/ docker/ .devcontainer/ .github/ scripts/
git commit -m "refactor: migrate to core structure"
```

---

## 📌 Wichtige Dateien

**Root Level:**
- `README.md` - Project Overview
- `Cargo.workspace.toml` - Rust Workspace
- `package.json` - Root Node Config
- `docker-compose.yml` - Dev Environment

**Documentation:**
- `docs/CODEX_SYSTEM_PROMPT.md` - Codex AI Prompt
- `docs/CORE_STRUCTURE.md` - This File
- `docs/FOLDER_STRUCTURE_V0.2.md` - Detailed Structure
- `docs/GIT_REDESIGN.md` - Git Strategy

**Approval:**
- `APPROVAL.md` - Approval Checklist
- `APPROVAL_COMBINED.json` - Approval Status
- `DOWNLOAD_GUIDE.md` - Download Instructions

---

## 🎯 Implementation Timeline

### Week 1-2: Foundation
- [ ] Create core/ structure
- [ ] Setup Cargo Workspace
- [ ] Setup npm Workspaces
- [ ] Migrate backend services

### Week 3-5: Integration
- [ ] Migrate frontend
- [ ] Setup plugins
- [ ] Configure database
- [ ] Setup DevOps

### Week 6-7: Testing
- [ ] Unit tests
- [ ] Integration tests
- [ ] E2E tests

### Week 8: Release
- [ ] Documentation
- [ ] Docker images
- [ ] v0.2.0 Release

---

**Status**: ✅ **CORE STRUCTURE READY**

_Vereinfachte, übersichtliche Architektur für bkg.rs v0.2_  
_Alles Wichtige in `core/`, DevOps separiert_
