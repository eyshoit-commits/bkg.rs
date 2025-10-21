# bkg.rs - Struktur-Analyse & v0.2 Erweiterung

**Datum**: 2025-10-21  
**Status**: Analysis Complete  
**Autor**: Cascade AI

---

## 📊 Aktuelle Struktur-Analyse

### 1. Backend (apps/bkg-api)

#### Aktuelle Komponenten:
```
apps/bkg-api/src/
├── app.module.ts              # Root Module
├── app.service.ts             # App Service
├── main.ts                    # Bootstrap
├── auth/                      # Authentication
│   ├── auth.controller.ts
│   └── dto.ts
├── admin/                     # Admin Panel
│   └── admin.controller.ts
├── chat/                      # Chat Features
│   ├── chat.controller.ts
│   ├── chat.controller.spec.ts
│   └── dto.ts
├── health/                    # Health Check
│   └── health.controller.ts
├── plugins/                   # Plugin System
│   ├── plugin.types.ts
│   ├── plugin.service.ts
│   ├── plugin.service.spec.ts
│   ├── plugin-bus.service.ts
│   └── plugin.module.ts
├── storage/                   # Database
│   ├── database.module.ts
│   └── database.service.ts
└── common/                    # Shared
    └── guards/
        ├── api-key.guard.ts
        └── api-key.guard.spec.ts
```

#### Aktuelle Features:
- ✅ Plugin-Registrierung & Lifecycle
- ✅ Plugin-Bus (RPC-Kommunikation)
- ✅ Auth & API-Keys
- ✅ Chat-Endpoints
- ✅ Health-Check
- ✅ Admin-Panel

#### Fehlende Features für v0.2:
- ❌ Plugin Management API (/api/plugins)
- ❌ WebSocket Gateway für Logs
- ❌ Telemetrie-Endpoints
- ❌ Plugin Config Management
- ❌ Model Management API
- ❌ Graceful Shutdown Handler

---

### 2. Frontend (apps/bkg-web)

#### Aktuelle Komponenten:
```
apps/bkg-web/src/app/
├── app.component.ts           # Root Component
├── app-routing.module.ts      # Routing
├── models/
│   └── api.models.ts
├── components/
│   ├── admin/
│   │   └── admin.component.ts
│   ├── chat/
│   │   └── chat.component.ts
│   └── plugins/
│       └── plugins.component.ts
└── services/
    ├── api.service.ts
    ├── auth.service.ts
    └── auth.interceptor.ts
```

#### Aktuelle Features:
- ✅ Chat UI
- ✅ Admin Panel
- ✅ Plugin Display
- ✅ Auth Service
- ✅ API Interceptor

#### Fehlende Features für v0.2:
- ❌ Plugin Dashboard (pro Plugin)
- ❌ Real-time Logs (WebSocket)
- ❌ Telemetrie-Anzeige
- ❌ Plugin Controls (Start/Stop/Restart)
- ❌ Model Management UI
- ❌ Standalone Components
- ❌ Signal-based State Management

---

### 3. Plugins

#### Aktuelle Plugins:
```
plugins/
├── llmserver/              # Rust LLM Server
│   ├── src/
│   ├── Cargo.toml
│   └── start.sh
├── repoagent/              # Python Code Analysis
│   ├── src/
│   ├── requirements.txt
│   └── start.sh
├── apikeys/                # Node.js Auth
│   ├── src/
│   ├── package.json
│   └── start.sh
├── brainml/                # Rust Brain ML
│   ├── src/
│   ├── Cargo.toml
│   └── start.sh
└── plugins.json            # Plugin Registry
```

#### Aktuelle Capabilities:
- ✅ llm.chat, llm.embed
- ✅ repo.analyze, repo.patch
- ✅ auth.login, auth.createKey, auth.revokeKey, auth.listKeys, auth.validate
- ✅ brainml.index, brainml.query, brainml.train, brainml.stats, brainml.admin

#### Fehlende Plugins für v0.2:
- ❌ candle/ (Hugging Face ML)
- ❌ rustyface/ (Face Recognition)

---

## 🎯 v0.2 Erweiterung - Neue Struktur

### Backend-Erweiterung (apps/bkg-api)

```
apps/bkg-api/src/
├── app.module.ts
├── app.service.ts
├── main.ts
│
├── auth/                      # Existing
│   ├── auth.controller.ts
│   └── dto.ts
│
├── admin/                     # Existing
│   └── admin.controller.ts
│
├── chat/                      # Existing
│   ├── chat.controller.ts
│   ├── chat.controller.spec.ts
│   └── dto.ts
│
├── health/                    # Existing
│   └── health.controller.ts
│
├── plugins/                   # EXPANDED
│   ├── plugin.types.ts
│   ├── plugin.service.ts
│   ├── plugin.service.spec.ts
│   ├── plugin-bus.service.ts
│   ├── plugin.module.ts
│   ├── plugins.controller.ts              # NEW
│   ├── plugins.gateway.ts                 # NEW (WebSocket)
│   ├── dto/                               # NEW
│   │   ├── plugin-info.dto.ts
│   │   ├── plugin-telemetry.dto.ts
│   │   ├── plugin-config.dto.ts
│   │   └── plugin-logs.dto.ts
│   ├── services/                          # NEW
│   │   ├── plugin-telemetry.service.ts
│   │   ├── plugin-config.service.ts
│   │   └── plugin-logs.service.ts
│   └── spec/                              # NEW
│       ├── plugins.controller.spec.ts
│       └── plugins.gateway.spec.ts
│
├── models/                    # NEW
│   ├── models.controller.ts
│   ├── models.service.ts
│   └── dto/
│       ├── model-info.dto.ts
│       └── model-download.dto.ts
│
├── websocket/                 # NEW
│   ├── websocket.gateway.ts
│   ├── websocket.service.ts
│   └── websocket.gateway.spec.ts
│
├── storage/                   # Existing
│   ├── database.module.ts
│   └── database.service.ts
│
├── common/                    # EXPANDED
│   ├── guards/
│   │   ├── api-key.guard.ts
│   │   ├── api-key.guard.spec.ts
│   │   └── ws-auth.guard.ts               # NEW
│   ├── filters/                           # NEW
│   │   └── ws-exception.filter.ts
│   ├── decorators/                        # NEW
│   │   └── ws-auth.decorator.ts
│   ├── interceptors/                      # NEW
│   │   └── logging.interceptor.ts
│   └── utils/                             # NEW
│       ├── plugin-utils.ts
│       └── telemetry-utils.ts
│
└── config/                    # NEW
    ├── plugin.config.ts
    ├── websocket.config.ts
    └── telemetry.config.ts
```

#### Neue Features:
- ✅ PluginsController (/api/plugins)
- ✅ WebSocket Gateway (Real-time Logs)
- ✅ Telemetrie-Service
- ✅ Config Management
- ✅ Model Management
- ✅ Logging Service
- ✅ Error Handling

---

### Frontend-Erweiterung (apps/bkg-web)

```
apps/bkg-web/src/app/
├── app.component.ts
├── app-routing.module.ts
│
├── core/                      # NEW
│   ├── services/
│   │   ├── plugin-api.service.ts
│   │   ├── websocket.service.ts
│   │   ├── telemetry.service.ts
│   │   └── plugin-store.service.ts
│   ├── components/
│   │   ├── plugin-header.component.ts
│   │   ├── plugin-stats.component.ts
│   │   ├── plugin-logs.component.ts
│   │   ├── plugin-config.component.ts
│   │   └── sidebar.component.ts
│   ├── guards/
│   │   └── auth.guard.ts
│   └── models/
│       └── plugin.model.ts
│
├── features/                  # NEW
│   └── plugins/
│       ├── plugins.routes.ts
│       ├── brainml/
│       │   └── brainml-dashboard.component.ts
│       ├── candle/
│       │   └── candle-dashboard.component.ts
│       ├── rustyface/
│       │   └── rustyface-dashboard.component.ts
│       ├── llmserver/
│       │   └── llmserver-dashboard.component.ts
│       ├── repoagent/
│       │   └── repoagent-dashboard.component.ts
│       └── apikeys/
│           └── apikeys-dashboard.component.ts
│
├── shared/                    # NEW
│   ├── components/
│   │   ├── status-badge.component.ts
│   │   ├── loading-spinner.component.ts
│   │   └── error-alert.component.ts
│   ├── pipes/
│   │   ├── uptime.pipe.ts
│   │   └── bytes.pipe.ts
│   └── directives/
│       └── auto-scroll.directive.ts
│
├── models/                    # Existing
│   └── api.models.ts
│
├── components/                # Existing
│   ├── admin/
│   │   └── admin.component.ts
│   ├── chat/
│   │   └── chat.component.ts
│   └── plugins/
│       └── plugins.component.ts
│
└── services/                  # Existing
    ├── api.service.ts
    ├── auth.service.ts
    └── auth.interceptor.ts
```

#### Neue Features:
- ✅ Plugin Dashboard System
- ✅ WebSocket Integration
- ✅ Telemetrie-Anzeige
- ✅ Real-time Logs
- ✅ Standalone Components
- ✅ Signal-based State Management
- ✅ Shared Components & Pipes

---

### Plugin-Erweiterung

#### Neue Plugins:

**1. Candle Plugin**
```
plugins/candle/
├── src/
│   ├── lib.rs
│   ├── plugin.rs
│   ├── capabilities/
│   │   ├── model_load.rs
│   │   ├── inference.rs
│   │   └── quantize.rs
│   └── models/
│       └── candle_model.rs
├── Cargo.toml
├── start.sh
└── README.md
```

**Capabilities:**
- candle.model.load
- candle.model.infer
- candle.model.quantize

**2. RustyFace Plugin**
```
plugins/rustyface/
├── src/
│   ├── lib.rs
│   ├── plugin.rs
│   ├── capabilities/
│   │   ├── detect.rs
│   │   ├── embed.rs
│   │   └── recognize.rs
│   └── models/
│       └── face_model.rs
├── Cargo.toml
├── start.sh
└── README.md
```

**Capabilities:**
- rustyface.face.detect
- rustyface.face.embed
- rustyface.face.recognize

#### Updated plugins.json:

```json
[
  {
    "name": "llmserver",
    "description": "Rust-based LLM server with chat and embedding capabilities",
    "entrypoint": "start.sh",
    "autostart": true,
    "capabilities": ["llm.chat", "llm.embed"],
    "healthcheck": { "path": "/health", "intervalSeconds": 15 }
  },
  {
    "name": "repoagent",
    "description": "Python RepoAgent integration",
    "entrypoint": "start.sh",
    "autostart": false,
    "capabilities": ["repo.analyze", "repo.patch"],
    "healthcheck": { "path": "/health", "intervalSeconds": 30 }
  },
  {
    "name": "apikeys",
    "description": "Authentication and API key management",
    "entrypoint": "start.sh",
    "autostart": true,
    "capabilities": ["auth.login", "auth.createKey", "auth.revokeKey", "auth.listKeys", "auth.validate"],
    "healthcheck": { "path": "/health", "intervalSeconds": 15 }
  },
  {
    "name": "brainml",
    "description": "Brain ML indexing and querying",
    "entrypoint": "start.sh",
    "autostart": false,
    "capabilities": ["brainml.index", "brainml.query", "brainml.train", "brainml.stats", "brainml.admin"],
    "healthcheck": { "path": "/health", "intervalSeconds": 15 }
  },
  {
    "name": "candle",
    "description": "Hugging Face Candle ML integration",
    "entrypoint": "start.sh",
    "autostart": false,
    "capabilities": ["candle.model.load", "candle.model.infer", "candle.model.quantize"],
    "healthcheck": { "path": "/health", "intervalSeconds": 15 }
  },
  {
    "name": "rustyface",
    "description": "Face recognition and embedding",
    "entrypoint": "start.sh",
    "autostart": false,
    "capabilities": ["rustyface.face.detect", "rustyface.face.embed", "rustyface.face.recognize"],
    "healthcheck": { "path": "/health", "intervalSeconds": 15 }
  }
]
```

---

## 📈 Neue API-Endpoints

### Plugin Management

```
GET    /api/plugins                           # List all plugins
GET    /api/plugins/:id                       # Get plugin details
POST   /api/plugins/:id/start                 # Start plugin
POST   /api/plugins/:id/stop                  # Stop plugin
POST   /api/plugins/:id/restart               # Restart plugin
GET    /api/plugins/:id/status                # Get plugin status
GET    /api/plugins/:id/logs                  # Get logs (WebSocket)
GET    /api/plugins/:id/telemetry             # Get telemetry (WebSocket)
POST   /api/plugins/:id/config                # Update config
GET    /api/plugins/:id/capabilities          # Get capabilities
```

### Model Management

```
GET    /api/models                            # List all models
GET    /api/models/:id                        # Get model details
POST   /api/models/download                   # Download model
DELETE /api/models/:id                        # Delete model
POST   /api/models/:id/validate               # Validate model
```

### WebSocket Endpoints

```
WS     /ws/plugins/:id/logs                   # Real-time logs
WS     /ws/plugins/:id/telemetry              # Real-time telemetry
WS     /ws/plugins/:id/status                 # Real-time status
```

---

## 🔄 Datenfluss v0.2

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Angular 17)                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Plugin Dashboard (Lazy Loaded per Plugin)           │  │
│  │  ├── PluginHeaderComponent (Controls)                │  │
│  │  ├── PluginStatsComponent (Telemetry)               │  │
│  │  ├── PluginLogsComponent (WebSocket Logs)           │  │
│  │  └── PluginConfigComponent (Settings)               │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
           ↓ HTTP/REST + WebSocket ↓
┌─────────────────────────────────────────────────────────────┐
│                    Backend (NestJS)                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  PluginsController (/api/plugins)                    │  │
│  │  ├── GET /api/plugins                               │  │
│  │  ├── POST /api/plugins/:id/start                    │  │
│  │  └── POST /api/plugins/:id/stop                     │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  WebSocket Gateway (Real-time)                       │  │
│  │  ├── /ws/plugins/:id/logs                           │  │
│  │  ├── /ws/plugins/:id/telemetry                      │  │
│  │  └── /ws/plugins/:id/status                         │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  PluginService (Lifecycle Management)                │  │
│  │  ├── startPlugin()                                  │  │
│  │  ├── stopPlugin()                                   │  │
│  │  └── getPluginStatus()                              │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
           ↓ Plugin Bus (RPC) ↓
┌─────────────────────────────────────────────────────────────┐
│                    Plugins (Rust/Python/Node)               │
│  ├── llmserver (Chat/Embedding)                             │
│  ├── candle (ML Models)                                     │
│  ├── rustyface (Face Recognition)                           │
│  ├── brainml (Indexing/Query)                               │
│  ├── repoagent (Code Analysis)                              │
│  └── apikeys (Authentication)                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 📁 Neue Folder-Struktur (Zusammenfassung)

### Backend
```
✅ plugins/plugins.controller.ts              (NEW)
✅ plugins/plugins.gateway.ts                 (NEW - WebSocket)
✅ plugins/dto/                               (NEW)
✅ plugins/services/                          (NEW)
✅ models/                                    (NEW)
✅ websocket/                                 (NEW)
✅ common/guards/ws-auth.guard.ts             (NEW)
✅ common/filters/                            (NEW)
✅ common/decorators/                         (NEW)
✅ common/interceptors/                       (NEW)
✅ config/                                    (NEW)
```

### Frontend
```
✅ core/services/                             (NEW)
✅ core/components/                           (NEW)
✅ core/guards/                               (NEW)
✅ features/plugins/                          (NEW)
✅ shared/components/                         (NEW)
✅ shared/pipes/                              (NEW)
✅ shared/directives/                         (NEW)
```

### Plugins
```
✅ plugins/candle/                            (NEW)
✅ plugins/rustyface/                         (NEW)
✅ plugins/plugins.json                       (UPDATED)
```

---

## 🎯 Implementation Priority

### Phase 1: Foundation
1. ✅ Proxy-Fix (Cargo/npm)
2. ✅ Backend: PluginsController
3. ✅ Backend: WebSocket Gateway
4. ✅ Frontend: Core Services
5. ✅ Frontend: Shared Components

### Phase 2: Integration
1. ✅ Frontend: Plugin Dashboards
2. ✅ Backend: Telemetrie-Service
3. ✅ Backend: Config Management
4. ✅ Frontend: WebSocket Integration
5. ✅ Plugins: Candle

### Phase 3: Polish
1. ✅ Plugins: RustyFace
2. ✅ Tests & Documentation
3. ✅ Error Handling
4. ✅ Performance Optimization
5. ✅ Release v0.2

---

**Status**: ✅ **STRUKTUR-ANALYSE COMPLETE**  
**Neue Komponenten**: 30+  
**Neue Features**: 15+  
**Neue Plugins**: 2  
**Zeilen Code (erwartet)**: ~7000
