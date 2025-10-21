# bkg.rs v0.2 - Neue Folder-Struktur

**Status**: APPROVED FOR IMPLEMENTATION  
**Datum**: 2025-10-21

---

## 📁 Aktuelle Struktur (v0.1a)

```
bkg.rs/
├── apps/
│   ├── bkg-api/              # NestJS Backend
│   └── bkg-web/              # Angular 17 Frontend
├── plugins/
│   ├── llmserver/            # Rust LLM
│   ├── repoagent/            # Python Code Analysis
│   ├── apikeys/              # Node.js Auth
│   ├── brainml/              # Rust Brain ML
│   ├── candle/               # Rust ML (NEW)
│   └── rustyface/            # Rust Face Recognition (NEW)
├── docker/
├── models/
├── docs/
└── scripts/
```

---

## 🎯 Neue Struktur für v0.2 (EMPFOHLEN)

### Option A: Microservices-Struktur (EMPFOHLEN)

```
bkg.rs/
│
├── backend/                          # Alle Backend-Services
│   ├── gateway/                      # REST API + WebSocket Hub
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── routes/
│   │   │   │   ├── plugins.rs        # Plugin Endpoints
│   │   │   │   ├── models.rs         # Model Endpoints
│   │   │   │   ├── admin.rs          # Admin Endpoints
│   │   │   │   └── auth.rs           # Auth Endpoints
│   │   │   ├── ws/
│   │   │   │   ├── mod.rs            # WebSocket Hub
│   │   │   │   ├── handlers.rs
│   │   │   │   └── topics.rs
│   │   │   ├── middleware/
│   │   │   ├── config.rs
│   │   │   └── lib.rs
│   │   ├── Cargo.toml
│   │   ├── Cargo.lock
│   │   └── tests/
│   │
│   ├── ml-engine/                    # ML Services (Candle, RustyFace, BrainML)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── candle_plugin.rs
│   │   │   ├── rustyface_plugin.rs
│   │   │   ├── brainml_bridge.rs
│   │   │   └── models/
│   │   ├── Cargo.toml
│   │   └── tests/
│   │
│   ├── vector-store/                 # Vector Database
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── collections.rs
│   │   │   ├── snapshots.rs
│   │   │   └── queries.rs
│   │   ├── Cargo.toml
│   │   └── migrations/
│   │
│   ├── auth-service/                 # JWT + API Keys
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── jwt.rs
│   │   │   ├── api_keys.rs
│   │   │   ├── permissions.rs
│   │   │   └── roles.rs
│   │   ├── Cargo.toml
│   │   └── tests/
│   │
│   └── shared/                       # Shared Types & Utils
│       ├── src/
│       │   ├── lib.rs
│       │   ├── types.rs              # DTOs, Models
│       │   ├── errors.rs             # Error Types
│       │   ├── config.rs             # Configuration
│       │   ├── telemetry.rs          # Metrics & Logging
│       │   └── utils.rs              # Helper Functions
│       ├── Cargo.toml
│       └── tests/
│
├── frontend/                         # Angular Frontend
│   ├── admin-ui/                     # Main Admin Dashboard
│   │   ├── src/
│   │   │   ├── app/
│   │   │   │   ├── app.component.ts
│   │   │   │   ├── app.routes.ts     # Main Routing
│   │   │   │   │
│   │   │   │   ├── core/             # Core Services & Guards
│   │   │   │   │   ├── services/
│   │   │   │   │   │   ├── plugin-api.service.ts
│   │   │   │   │   │   ├── ws.service.ts
│   │   │   │   │   │   ├── auth.service.ts
│   │   │   │   │   │   └── telemetry.service.ts
│   │   │   │   │   ├── guards/
│   │   │   │   │   │   └── auth.guard.ts
│   │   │   │   │   └── models/
│   │   │   │   │       └── plugin.model.ts
│   │   │   │   │
│   │   │   │   ├── shared/           # Shared Components & Pipes
│   │   │   │   │   ├── components/
│   │   │   │   │   │   ├── plugin-header.component.ts
│   │   │   │   │   │   ├── plugin-stats.component.ts
│   │   │   │   │   │   ├── plugin-logs.component.ts
│   │   │   │   │   │   ├── plugin-config.component.ts
│   │   │   │   │   │   ├── status-badge.component.ts
│   │   │   │   │   │   └── confirm-dialog.component.ts
│   │   │   │   │   ├── pipes/
│   │   │   │   │   │   ├── uptime.pipe.ts
│   │   │   │   │   │   └── bytes.pipe.ts
│   │   │   │   │   └── directives/
│   │   │   │   │       └── auto-scroll.directive.ts
│   │   │   │   │
│   │   │   │   ├── features/         # Feature Modules
│   │   │   │   │   ├── dashboard/
│   │   │   │   │   │   └── dashboard.component.ts
│   │   │   │   │   ├── plugins/
│   │   │   │   │   │   ├── plugins.routes.ts
│   │   │   │   │   │   ├── brainml/
│   │   │   │   │   │   │   └── brainml-dashboard.component.ts
│   │   │   │   │   │   ├── candle/
│   │   │   │   │   │   │   └── candle-dashboard.component.ts
│   │   │   │   │   │   ├── rustyface/
│   │   │   │   │   │   │   └── rustyface-dashboard.component.ts
│   │   │   │   │   │   ├── llmserver/
│   │   │   │   │   │   │   └── llmserver-dashboard.component.ts
│   │   │   │   │   │   ├── repoagent/
│   │   │   │   │   │   │   └── repoagent-dashboard.component.ts
│   │   │   │   │   │   └── apikeys/
│   │   │   │   │   │       └── apikeys-dashboard.component.ts
│   │   │   │   │   ├── settings/
│   │   │   │   │   │   └── settings.component.ts
│   │   │   │   │   ├── users/
│   │   │   │   │   │   └── users.component.ts
│   │   │   │   │   └── api-keys/
│   │   │   │   │       └── api-keys.component.ts
│   │   │   │   │
│   │   │   │   ├── stores/           # State Management (Signals)
│   │   │   │   │   ├── plugin.store.ts
│   │   │   │   │   ├── auth.store.ts
│   │   │   │   │   └── telemetry.store.ts
│   │   │   │   │
│   │   │   │   └── app.config.ts
│   │   │   │
│   │   │   ├── main.ts
│   │   │   ├── styles.css
│   │   │   └── index.html
│   │   │
│   │   ├── package.json
│   │   ├── package-lock.json
│   │   ├── angular.json
│   │   ├── tsconfig.json
│   │   ├── tailwind.config.js
│   │   └── karma.conf.js
│   │
│   └── web-ui/                       # (DEPRECATED - nur für Migration)
│       └── [alte Struktur]
│
├── plugins/                          # Plugin Implementations
│   ├── brainml/
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   ├── config.json
│   │   ├── start.sh
│   │   └── README.md
│   │
│   ├── candle/                       # NEW
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── plugin.rs
│   │   │   ├── capabilities/
│   │   │   │   ├── model_load.rs
│   │   │   │   ├── inference.rs
│   │   │   │   └── quantize.rs
│   │   │   └── models/
│   │   ├── Cargo.toml
│   │   ├── config.json
│   │   ├── start.sh
│   │   └── README.md
│   │
│   ├── rustyface/                    # NEW
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── plugin.rs
│   │   │   ├── capabilities/
│   │   │   │   ├── detect.rs
│   │   │   │   ├── embed.rs
│   │   │   │   └── recognize.rs
│   │   │   └── models/
│   │   ├── Cargo.toml
│   │   ├── config.json
│   │   ├── start.sh
│   │   └── README.md
│   │
│   ├── llmserver/
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   ├── config.json
│   │   ├── start.sh
│   │   └── README.md
│   │
│   ├── repoagent/
│   │   ├── repoagent/
│   │   ├── pyproject.toml
│   │   ├── config.json
│   │   ├── start.sh
│   │   └── README.md
│   │
│   ├── apikeys/
│   │   ├── src/
│   │   ├── package.json
│   │   ├── config.json
│   │   ├── start.sh
│   │   └── README.md
│   │
│   └── plugins.json                  # Plugin Registry
│
├── database/                         # Database
│   ├── migrations/
│   │   ├── 001_init_schema.sql
│   │   ├── 002_add_api_keys.sql
│   │   ├── 003_add_models.sql
│   │   └── ...
│   ├── schema.sql
│   └── seeds/
│       └── initial_data.sql
│
├── docker/                           # Docker
│   ├── Dockerfile                    # Multi-stage build
│   ├── docker-compose.yml            # Development setup
│   ├── docker-compose.prod.yml       # Production setup
│   ├── supervisord.conf              # Process management
│   └── start.sh                      # Entrypoint
│
├── .devcontainer/                    # VSCode Devcontainer
│   ├── devcontainer.json
│   └── post-create.sh
│
├── .github/                          # GitHub
│   ├── workflows/
│   │   ├── ci-cleanup.yml
│   │   ├── ci-build.yml
│   │   ├── ci-test.yml
│   │   └── cd-deploy.yml
│   └── ISSUE_TEMPLATE/
│
├── docs/                             # Documentation
│   ├── README.md
│   ├── INDEX.md
│   ├── next.md                       # Roadmap
│   ├── STRUCTURE_ANALYSIS.md
│   ├── GIT_REDESIGN.md
│   ├── CODEX_SYSTEM_PROMPT.md
│   ├── DOWNLOAD_GUIDE.md
│   ├── PATCH_TROUBLESHOOTING.md
│   ├── architecture/
│   │   └── plugin_system_v0.2.md
│   ├── implementation/
│   │   └── angular_adminui_setup.md
│   ├── update/
│   │   ├── v0.1a.md
│   │   ├── v0.1a.json
│   │   └── v0.2-phase1.md
│   ├── deployment.md
│   ├── operations.md
│   └── changelog/
│       └── CHANGELOG.md
│
├── scripts/                          # Build & Utility Scripts
│   ├── cleanup-workspace.sh
│   ├── setup-dev.sh
│   ├── build-docker.sh
│   ├── deploy.sh
│   └── download-models.sh
│
├── models/                           # ML Models (GGUF)
│   ├── all-MiniLM-L6-v2-ggml-model-f16.gguf
│   ├── Qwen2-0.5B-Instruct-Q5_K_M.gguf
│   └── README.md
│
├── .gitignore
├── .gitattributes
├── Cargo.workspace.toml              # Rust Workspace
├── package.json                      # Root Node Config
├── docker-compose.yml
├── README.md
├── APPROVAL.md
├── APPROVAL_COMBINED.json
├── FINAL_SUMMARY.md
├── V0.2_IMPLEMENTATION_GUIDE.md
└── LICENSE
```

---

## 🔄 Migration Path (v0.1a → v0.2)

### Phase 1: Struktur vorbereiten

```bash
# 1. Neue Verzeichnisse erstellen
mkdir -p backend/{gateway,ml-engine,vector-store,auth-service,shared}
mkdir -p frontend/admin-ui
mkdir -p database/migrations
mkdir -p .devcontainer
mkdir -p .github/workflows

# 2. Alte Struktur beibehalten (für jetzt)
# apps/ bleibt als Legacy
# plugins/ wird zu backend/ml-engine + plugins/

# 3. Neue Dateien committen
git add backend/ frontend/ database/ .devcontainer/ .github/
git commit -m "refactor: prepare v0.2 folder structure"
```

### Phase 2: Code migrieren

```bash
# 1. Gateway Service
cp apps/bkg-api/src/* backend/gateway/src/

# 2. Frontend
cp apps/bkg-web/src/* frontend/admin-ui/src/

# 3. Plugins
cp plugins/* backend/ml-engine/plugins/

# 4. Database
cp database/migrations/* database/migrations/
```

### Phase 3: Alte Struktur entfernen

```bash
# Nach vollständiger Migration
rm -rf apps/
git commit -m "refactor: remove legacy apps/ directory"
```

---

## 📊 Vergleich: Alt vs Neu

| Aspekt | v0.1a | v0.2 |
|--------|-------|------|
| **Backend** | `apps/bkg-api/` | `backend/gateway/` + Services |
| **Frontend** | `apps/bkg-web/` | `frontend/admin-ui/` |
| **Plugins** | `plugins/` (flat) | `backend/ml-engine/` + `plugins/` |
| **Services** | 1 (NestJS) | 5 (Rust Microservices) |
| **Routing** | Single App | Feature-based Routing |
| **State** | RxJS | Signals |
| **Database** | Implicit | Explicit `database/` |
| **DevOps** | Docker only | Docker + Devcontainer + CI/CD |

---

## 🎯 Folder-Struktur Prinzipien

### ✅ Modularität
- Jeder Service = eigenes Verzeichnis
- Klare Abhängigkeiten
- Einfach zu erweitern

### ✅ Skalierbarkeit
- Microservices-Architektur
- Separate Deployments
- Independent Scaling

### ✅ Wartbarkeit
- Klare Struktur
- Konsistente Naming
- Dokumentation co-located

### ✅ Testbarkeit
- Tests neben Code
- Separate Test-Verzeichnisse
- CI/CD Integration

---

## 📁 Workspace Root Files

```
bkg.rs/
├── Cargo.workspace.toml              # Rust Workspace Config
├── package.json                      # Root Node Config
├── docker-compose.yml                # Dev Environment
├── docker-compose.prod.yml           # Production
├── .gitignore
├── .gitattributes
├── README.md
├── LICENSE
│
├── APPROVAL.md                       # Approval Checklist
├── APPROVAL_COMBINED.json            # Approval Status
├── CODEX_SYSTEM_PROMPT.md            # Codex AI Prompt
├── DOWNLOAD_GUIDE.md                 # Download Instructions
├── FINAL_SUMMARY.md                  # Final Summary
├── FOLDER_STRUCTURE_V0.2.md          # This File
├── PATCH_TROUBLESHOOTING.md          # Troubleshooting
├── V0.2_IMPLEMENTATION_GUIDE.md      # Implementation Guide
└── [weitere Docs]
```

---

## 🚀 Implementation Order

### Week 1-2 (Phase 1)
1. ✅ Folder-Struktur erstellen
2. ✅ Cargo Workspace konfigurieren
3. ✅ Backend Services scaffolden
4. ✅ Plugin Registry implementieren

### Week 3-5 (Phase 2)
1. ✅ Frontend Admin-UI scaffolden
2. ✅ Shared Components erstellen
3. ✅ WebSocket Integration
4. ✅ Plugin Dashboards

### Week 6-7 (Phase 3)
1. ✅ API Endpoints implementieren
2. ✅ Model Management
3. ✅ Integration Tests

### Week 8 (Phase 4)
1. ✅ Release v0.2
2. ✅ Dokumentation finalisieren
3. ✅ Docker Images bauen

---

## 📌 Wichtige Hinweise

1. **Cargo Workspace**: Alle Rust-Services in einem Workspace
2. **npm Workspaces**: Frontend + Root Package
3. **Shared Code**: `backend/shared/` für alle Services
4. **Database Migrations**: Versioniert in `database/migrations/`
5. **Plugin Registry**: `plugins/plugins.json` bleibt zentral

---

**Status**: ✅ **READY FOR RESTRUCTURING**

_Neue Folder-Struktur für bkg.rs v0.2_  
_Microservices-Architektur mit klarer Modularität_
