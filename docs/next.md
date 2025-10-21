# bkg.rs - Nächste Phase: Modulares Plug-in-Framework mit Admin-Dashboard

**Datum:** 2025-10-21  
**Status:** Planning Phase  
**Autor:** lofmas + Cascade AI

---

## 📊 Aktueller Status (v0.1a)

### Fortschritt
- **64%** Projektfortschritt nach BrainML-Integration
- ✅ BrainML-Plug-in vollständig integriert
- ✅ Adapter-Layer zu braindb & llm-Plug-ins fertiggestellt
- ✅ Dokumentationspipeline erweitert
- ⚠️ Proxy-/Registry-Blocker (Cargo, npm)
- ⚠️ Persistenz-/Snapshot-Validierung ausstehend

### Aktive Module
- `core/backend/gateway` (NestJS Plug-in-Host)
- `core/core/plugins/llmserver` (Rust LLM)
- `core/core/plugins/repoagent` (Python Code-Analyse)
- `core/core/plugins/apikeys` (Node.js Auth)
- `core/core/plugins/brainml` (Rust Index/Query/Train)

### Fehler & Blocker
- `cargo clippy` & `cargo test` scheitern an 403-Proxy-Fehlern
- `npm install` blockiert durch Registry-403
- CI-Pipeline deaktiviert (an Proxy-Fix gekoppelt)

---

## 🎯 Vision: Nächste Phase (v0.2)

### Ziele

1. **Integration von Candle (Hugging Face)**
   - Native ML-Modelle in Rust (schnell, ohne Python)
   - Plug-in-basierte Architektur

2. **Integration von RustyFace**
   - Bridge zu HF-Models
   - Lokale Modelverwaltung
   - Embedding & Inferenz

3. **Refactoring der Ordnerstruktur**
   - Jedes Plug-in in eigenem Ordner
   - Klare Trennung API / Frontend

4. **Angular-basiertes Admin-Dashboard**
   - Dynamisches Plug-in-Management
   - Hot-Swap, Start/Stop, Restart
   - Model-Download & Konfiguration
   - Echtzeit-Logs via WebSocket

5. **Einheitliches Admin-Framework**
   - Gemeinsame Komponenten (PluginCard, PluginPanel, StatusChip, LogStream)
   - Telemetrie-Integration (CPU/RAM/Throughput)

---

## 🧩 Architektur-Übersicht

### Backend-Layer (Rust)

```
core/
├── plugin_bus.rs          # RPC-Bus / dynamic dispatch
├── plugin_registry.rs     # Registrierung, Status, Lifecycle
└── plugin_traits.rs       # Gemeinsame Traits

plugins/
├── brainml/               # Index, Query, Train, Stats, Admin
├── candle/                # Hugging Face Candle Integration
├── rustyface/             # Face Recognition & Embedding
├── llmserver/             # Chat & Embedding (bestehend)
├── repoagent/             # Code-Analyse (bestehend)
└── apikeys/               # Auth & Key-Verwaltung (bestehend)
```

**Plug-in-Schnittstelle:**

```rust
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn restart(&mut self) -> Result<()>;
    fn status(&self) -> PluginStatus;
    
    fn capabilities(&self) -> Vec<Capability>;
    fn admin_routes(&self) -> Vec<AdminRoute>;
    fn config(&self) -> PluginConfig;
}
```

### Frontend-Layer (Angular 17)

```
core/frontend/admin-ui/src/app/
├── plugins/
│   ├── brainml/
│   ├── candle/
│   ├── rustyface/
│   ├── llmserver/
│   ├── repoagent/
│   └── apikeys/
├── shared/
│   ├── components/
│   │   ├── plugin-card.component.ts
│   │   ├── plugin-panel.component.ts
│   │   ├── plugin-logs.component.ts
│   │   ├── plugin-telemetry.component.ts
│   │   └── status-chip.component.ts
│   ├── services/
│   │   ├── plugin.service.ts
│   │   ├── websocket.service.ts
│   │   └── telemetry.service.ts
│   └── models/
│       └── plugin.model.ts
└── admin/
    ├── plugin-dashboard.component.ts
    └── plugin-manager.component.ts
```

---

## 🛠️ Technische Umsetzung

### Backend-APIs

```
GET    /api/core/plugins                    # Alle Plug-ins + Status
GET    /api/core/plugins/:id                # Plug-in-Details
POST   /api/core/plugins/:id/start          # Plug-in starten
POST   /api/core/plugins/:id/stop           # Plug-in stoppen
POST   /api/core/plugins/:id/restart        # Plug-in neu starten
GET    /api/core/plugins/:id/logs           # Plug-in-Logs (WebSocket)
GET    /api/core/plugins/:id/telemetry      # CPU/RAM/Throughput
POST   /api/core/plugins/:id/config         # Konfiguration aktualisieren
POST   /api/core/plugins/:id/models/download # Modell herunterladen
```

### Frontend-State-Management

```typescript
// Angular Signals für Plug-in-Zustände
export const pluginStore = signalStore(
  withState(() => ({
    plugins: [] as PluginState[],
    selectedPlugin: null as PluginState | null,
    loading: false,
    error: null as string | null,
  })),
  withMethods((store) => ({
    loadPlugins: async () => { /* ... */ },
    startPlugin: async (id: string) => { /* ... */ },
    stopPlugin: async (id: string) => { /* ... */ },
    subscribeToLogs: (id: string) => { /* WebSocket */ },
  }))
);
```

### WebSocket-Integration

```typescript
// Real-time Logs & Telemetry
ws://localhost:43121/ws/core/plugins/:id/logs
ws://localhost:43121/ws/core/plugins/:id/telemetry
```

---

## 📦 Refactor-Plan als JSON (für Codex AI)

```json
{
  "task": "refactor_bkg_plugins_v0.2",
  "timestamp": "2025-10-21T07:18:00Z",
  "author": "lofmas",
  "phase": "planning",
  "description": "Refactor des bkg.rs-Systems zu modularem Plug-in-Framework mit Angular-Admin-Dashboard und Hot-Swap-Verwaltung",
  
  "current_state": {
    "progress": "64%",
    "active_plugins": ["brainml", "llmserver", "repoagent", "apikeys"],
    "blockers": ["proxy_registry_403", "npm_install_403", "ci_disabled"],
    "tech_stack": {
      "backend": "TypeScript (NestJS) + Rust (plugins)",
      "frontend": "Angular 17 + Tailwind CSS",
      "database": "SQLite",
      "deployment": "Docker Compose"
    }
  },
  
  "objectives": [
    "Integration von Hugging Face Candle als Rust-Plug-in",
    "Integration von RustyFace als Plug-in",
    "Refactoring der Ordnerstruktur für modulare Plugins",
    "Angular 17 Admin-Dashboard mit Hot-Swap Plug-in-Verwaltung",
    "Echtzeit-Logs und Telemetrie via WebSocket",
    "Proxy-/Registry-Konfiguration reparieren"
  ],
  
  "backend": {
    "language": "Rust + TypeScript (NestJS)",
    "core_modules": [
      "plugin_bus.rs (RPC-Bus / dynamic dispatch)",
      "plugin_registry.rs (Registrierung, Status, Lifecycle)",
      "plugin_traits.rs (Gemeinsame Traits)"
    ],
    "new_plugins": [
      {
        "name": "candle",
        "description": "Hugging Face Candle Integration",
        "capabilities": ["model.load", "model.infer", "model.quantize"],
        "language": "Rust"
      },
      {
        "name": "rustyface",
        "description": "Face Recognition & Embedding",
        "capabilities": ["face.detect", "face.embed", "face.recognize"],
        "language": "Rust"
      }
    ],
    "existing_plugins": ["brainml", "llmserver", "repoagent", "apikeys"],
    "apis": [
      "GET /api/core/plugins",
      "POST /api/core/plugins/:id/start|stop|restart",
      "GET /api/core/plugins/:id/logs",
      "GET /api/core/plugins/:id/telemetry",
      "POST /api/core/plugins/:id/config",
      "POST /api/core/plugins/:id/models/download"
    ]
  },
  
  "frontend": {
    "framework": "Angular 17",
    "state_management": "Angular Signals + Signal Store",
    "styling": "Tailwind CSS + HeadlessUI",
    "shared_components": [
      "PluginCard",
      "PluginPanel",
      "PluginLogs",
      "PluginTelemetry",
      "StatusChip"
    ],
    "plugin_modules": [
      "brainml",
      "candle",
      "rustyface",
      "llmserver",
      "repoagent",
      "apikeys"
    ],
    "features": [
      "Dynamic Plugin Tabs",
      "Start/Stop/Restart Controls",
      "Model Download & Configuration",
      "Real-time Logs (WebSocket)",
      "CPU/RAM/Throughput Telemetry",
      "Admin Panels pro Plug-in"
    ]
  },
  
  "next_steps": [
    {
      "id": "fix_proxy_build",
      "title": "Proxy-Konfiguration reparieren",
      "description": "Cargo und npm Registry-Zugang für vollständige Builds",
      "priority": "CRITICAL",
      "effort": "2h"
    },
    {
      "id": "candle_plugin_init",
      "title": "Candle-Plug-in initialisieren",
      "description": "Basis-Struktur, Traits, erste Modelle",
      "priority": "HIGH",
      "effort": "4h"
    },
    {
      "id": "rustyface_plugin_bridge",
      "title": "RustyFace-Plug-in als Bridge",
      "description": "HF-Models, lokale Verwaltung, Embedding",
      "priority": "HIGH",
      "effort": "6h"
    },
    {
      "id": "plugin_registry_refactor",
      "title": "Plugin-Registry & Hot-Swap",
      "description": "Dynamic loading, lifecycle management",
      "priority": "HIGH",
      "effort": "8h"
    },
    {
      "id": "admin_ui_plugin_panels",
      "title": "Angular Admin-Dashboard",
      "description": "Plug-in-Tabs, Controls, Logs, Telemetrie",
      "priority": "HIGH",
      "effort": "12h"
    },
    {
      "id": "websocket_integration",
      "title": "WebSocket für Logs & Telemetrie",
      "description": "Real-time Streams, Signal-Integration",
      "priority": "MEDIUM",
      "effort": "4h"
    },
    {
      "id": "telemetry_integration",
      "title": "Telemetrie & Monitoring",
      "description": "CPU/RAM/Throughput, OpenTelemetry/Prometheus",
      "priority": "MEDIUM",
      "effort": "6h"
    },
    {
      "id": "rag_integration_tests",
      "title": "RAG-Integration Tests",
      "description": "End-to-End Tests für BrainML + neue Plug-ins",
      "priority": "MEDIUM",
      "effort": "8h"
    },
    {
      "id": "reactivate_ci",
      "title": "CI-Pipeline reaktivieren",
      "description": "GitHub Actions mit Format/Lint/Test",
      "priority": "MEDIUM",
      "effort": "4h"
    }
  ],
  
  "timeline": {
    "phase_1": {
      "name": "Foundation (1-2 Wochen)",
      "tasks": ["fix_proxy_build", "plugin_registry_refactor", "candle_plugin_init"]
    },
    "phase_2": {
      "name": "Integration (2-3 Wochen)",
      "tasks": ["rustyface_plugin_bridge", "admin_ui_plugin_panels", "websocket_integration"]
    },
    "phase_3": {
      "name": "Polish & Testing (1-2 Wochen)",
      "tasks": ["telemetry_integration", "rag_integration_tests", "reactivate_ci"]
    }
  },
  
  "risks": [
    {
      "risk": "Proxy-Registry-Blocker",
      "impact": "HIGH",
      "mitigation": "Lokale Mirror oder VPN-Konfiguration"
    },
    {
      "risk": "Persistenz-Snapshots nicht validiert",
      "impact": "MEDIUM",
      "mitigation": "Snapshot-Hash-Vergleiche implementieren"
    },
    {
      "risk": "Fehlende Telemetrie",
      "impact": "MEDIUM",
      "mitigation": "OpenTelemetry/Prometheus Integration"
    },
    {
      "risk": "Hot-Swap unter Last",
      "impact": "MEDIUM",
      "mitigation": "Graceful Shutdown, Connection Draining"
    }
  ],
  
  "success_criteria": [
    "Alle 6 Plug-ins starten/stoppen/restarten via API",
    "Angular Admin-Dashboard zeigt Live-Status aller Plug-ins",
    "WebSocket-Logs und Telemetrie funktionieren",
    "Candle & RustyFace Plug-ins produktionsreif",
    "CI-Pipeline grün (Format, Lint, Tests)",
    "Dokumentation aktualisiert"
  ]
}
```

---

## 📋 Ordnerstruktur (v0.2)

```
/home/wind/devel/bkg.rs/
├── core/                          # Neue Rust-Core
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── plugin_bus.rs
│   │   ├── plugin_registry.rs
│   │   ├── plugin_traits.rs
│   │   └── models/
│   │       └── plugin.rs
│   └── tests/
│
├── core/
│   ├── backend/
│   │   └── gateway/               # NestJS Backend & Plug-in-Host
│   ├── frontend/
│   │   └── admin-ui/              # Angular Admin Dashboard
│   ├── plugins/
│   │   ├── apikeys/
│   │   ├── brainml/
│   │   ├── candle/
│   │   ├── llmserver/
│   │   ├── repoagent/
│   │   └── rustyface/
│   ├── database/
│   └── config/
│
├── devops/
│   ├── docker/
│   │   ├── Dockerfile
│   │   └── docker-compose.yml
│   ├── scripts/
│   │   ├── docker-start.sh
│   │   └── download-models.sh
│   └── .devcontainer/             # VS Code Container Setup
│       └── devcontainer.json
│
├── docs/
│   ├── next.md                    # Diese Datei
│   ├── update/
│   │   └── v0.1a.md
│   └── architecture/
│       └── plugin_system.md
│
├── models/
└── README.md
```

---

## 🚀 Erste Schritte

### 1. Proxy-Blocker beheben (CRITICAL)
```bash
# Cargo Registry konfigurieren
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "crates-io-mirror"

[source.crates-io-mirror]
registry = "https://github.com/rust-lang/crates.io-index"
EOF

# npm Registry konfigurieren
npm config set registry https://registry.npmjs.org/
```

### 2. Core-Struktur aufbauen
```bash
cd /home/wind/devel/bkg.rs
# Core-Struktur bereits erstellt
cd core
# Cargo.toml: plugin_bus, plugin_registry, plugin_traits
```

### 3. Candle-Plug-in initialisieren
```bash
cd /home/wind/devel/bkg.rs/core/plugins
cargo new candle
# Cargo.toml: candle, tch-rs, huggingface-hub
```

### 4. Admin-UI scaffolden
```bash
cd /home/wind/devel/bkg.rs/core/frontend
ng new admin-ui --standalone --routing --style=css
cd admin-ui
ng add @ngrx/signals
```

---

## 📚 Referenzen

- **v0.1a Plan**: `docs/update/v0.1a.md`
- **Aktuelle Architektur**: `AGeNT.md`
- **Docker Setup**: `DOCKER.md`
- **Entwicklung**: `DEV_SETUP.md`

---

**Status**: 📋 Planning Phase  
**Nächste Aktion**: Proxy-Konfiguration reparieren (CRITICAL)  
**Geschätzter Aufwand**: 4-6 Wochen für v0.2  
**Zielversion**: v0.2 (November 2025)

_Automatisch generiert durch Codex AI Planning System_
