# bkg.rs v0.2 Redesign - Git Commit & Branch Strategy

**Datum**: 2025-10-21  
**Status**: Planning  
**Autor**: lofmas

---

## 🎯 Git Strategy für v0.2 Redesign

### Branch-Struktur

```
main (stable)
├── feature/plugin-registry          # Phase 1: Core Registry
├── feature/candle-plugin            # Phase 1: Candle Integration
├── feature/rustyface-plugin         # Phase 1: RustyFace Integration
├── feature/admin-ui-setup           # Phase 2: Angular AdminUI
├── feature/websocket-integration    # Phase 2: Real-time Logs
├── feature/telemetry-dashboard      # Phase 3: Monitoring
└── release/v0.2                      # Release Branch
```

---

## 📝 Commit-Struktur

### Phase 1: Foundation (Proxy-Fix + Core)

#### Commit 1: Proxy-Konfiguration reparieren
```
commit: fix(build): repair cargo and npm registry access

- Configure Cargo registry mirror for crates.io
- Fix npm registry configuration
- Update .cargo/config.toml with working mirror
- Add npm registry configuration to .npmrc
- Enable CI/CD pipeline for builds

Files:
  .cargo/config.toml (new)
  .npmrc (new)
  .github/workflows/ci.yml (updated)
```

#### Commit 2: Plugin-Registry Core implementieren
```
commit: feat(core): implement plugin registry and lifecycle management

- Add plugin_registry.rs with Plugin trait
- Implement PluginRegistry for registration/lifecycle
- Add plugin_bus.rs for RPC communication
- Implement PluginStatus enum and PluginInfo struct
- Add error handling with PluginError

Files:
  core/src/lib.rs (new)
  core/src/plugin_traits.rs (new)
  core/src/plugin_registry.rs (new)
  core/src/plugin_bus.rs (new)
  core/Cargo.toml (new)
```

#### Commit 3: Candle-Plugin initialisieren
```
commit: feat(plugins): add candle plugin for hugging face integration

- Create candle plugin structure
- Implement Plugin trait for Candle
- Add model loading capability
- Add inference capability
- Integrate with plugin_bus

Files:
  plugins/candle/Cargo.toml (new)
  plugins/candle/src/lib.rs (new)
  plugins/candle/src/plugin.rs (new)
  plugins/candle/src/capabilities/
    ├── model_load.rs (new)
    └── inference.rs (new)
```

#### Commit 4: RustyFace-Plugin initialisieren
```
commit: feat(plugins): add rustyface plugin for face recognition

- Create rustyface plugin structure
- Implement Plugin trait for RustyFace
- Add face detection capability
- Add face embedding capability
- Integrate with plugin_bus

Files:
  plugins/rustyface/Cargo.toml (new)
  plugins/rustyface/src/lib.rs (new)
  plugins/rustyface/src/plugin.rs (new)
  plugins/rustyface/src/capabilities/
    ├── detect.rs (new)
    └── embed.rs (new)
```

---

### Phase 2: Frontend Integration

#### Commit 5: AdminUI Scaffolding
```
commit: feat(frontend): scaffold angular admin-ui with standalone components

- Create admin-ui Angular 17 app
- Setup standalone components
- Configure Tailwind CSS
- Add routing configuration
- Setup lazy loading for plugins

Files:
  apps/admin-ui/ (new)
  apps/admin-ui/src/app/app.component.ts (new)
  apps/admin-ui/src/app/app.routes.ts (new)
  apps/admin-ui/angular.json (new)
  apps/admin-ui/tailwind.config.js (new)
```

#### Commit 6: Core Services & Models
```
commit: feat(admin-ui): add plugin api service and models

- Create PluginApiService for REST communication
- Add WebSocketService for real-time logs
- Implement PluginInfo and PluginStatus models
- Add HTTP interceptors for error handling
- Setup RxJS observables for state management

Files:
  apps/admin-ui/src/app/core/services/plugin-api.service.ts (new)
  apps/admin-ui/src/app/core/services/websocket.service.ts (new)
  apps/admin-ui/src/app/core/models/plugin.model.ts (new)
```

#### Commit 7: Shared Components
```
commit: feat(admin-ui): implement shared plugin components

- Create PluginHeaderComponent (Start/Stop/Restart)
- Create PluginStatsComponent (CPU/RAM/Uptime)
- Create PluginLogsComponent (WebSocket logs)
- Create PluginConfigComponent (Settings)
- Create SidebarComponent (Navigation)

Files:
  apps/admin-ui/src/app/core/components/
    ├── plugin-header.component.ts (new)
    ├── plugin-stats.component.ts (new)
    ├── plugin-logs.component.ts (new)
    ├── plugin-config.component.ts (new)
    └── sidebar.component.ts (new)
```

#### Commit 8: Plugin Dashboards
```
commit: feat(admin-ui): add plugin-specific dashboards

- Create BrainML dashboard
- Create Candle dashboard
- Create RustyFace dashboard
- Create LLMServer dashboard
- Create RepoAgent dashboard
- Create APIKeys dashboard
- Setup lazy loading routes

Files:
  apps/admin-ui/src/app/features/plugins/
    ├── plugins.routes.ts (new)
    ├── brainml/brainml-dashboard.component.ts (new)
    ├── candle/candle-dashboard.component.ts (new)
    ├── rustyface/rustyface-dashboard.component.ts (new)
    ├── llmserver/llmserver-dashboard.component.ts (new)
    ├── repoagent/repoagent-dashboard.component.ts (new)
    └── apikeys/apikeys-dashboard.component.ts (new)
```

#### Commit 9: WebSocket Integration
```
commit: feat(admin-ui): add websocket support for real-time logs and telemetry

- Implement WebSocket connection management
- Add log streaming from plugins
- Add telemetry streaming (CPU/RAM)
- Setup auto-reconnect logic
- Add error handling and graceful degradation

Files:
  apps/admin-ui/src/app/core/services/websocket.service.ts (updated)
  apps/admin-ui/src/app/core/components/plugin-logs.component.ts (updated)
  apps/admin-ui/src/app/core/components/plugin-stats.component.ts (updated)
```

---

### Phase 3: Backend Integration & Testing

#### Commit 10: NestJS API Routes
```
commit: feat(api): add plugin management endpoints

- Create /api/plugins endpoint (list all)
- Create /api/plugins/:id endpoint (get details)
- Create /api/plugins/:id/start endpoint
- Create /api/plugins/:id/stop endpoint
- Create /api/plugins/:id/restart endpoint
- Create /api/plugins/:id/logs endpoint
- Create /api/plugins/:id/telemetry endpoint

Files:
  apps/bkg-api/src/plugins/plugins.controller.ts (new)
  apps/bkg-api/src/plugins/plugins.service.ts (new)
  apps/bkg-api/src/plugins/plugins.module.ts (new)
```

#### Commit 11: WebSocket Gateway
```
commit: feat(api): add websocket gateway for real-time communication

- Create WebSocket gateway for logs
- Create WebSocket gateway for telemetry
- Implement connection management
- Add authentication for WebSocket
- Setup message broadcasting

Files:
  apps/bkg-api/src/websocket/
    ├── plugins.gateway.ts (new)
    └── plugins.gateway.spec.ts (new)
```

#### Commit 12: Integration Tests
```
commit: test(integration): add end-to-end tests for plugin system

- Test plugin registration and lifecycle
- Test API endpoints
- Test WebSocket connections
- Test error handling
- Test concurrent operations

Files:
  apps/bkg-api/test/
    ├── plugins.e2e-spec.ts (new)
    └── websocket.e2e-spec.ts (new)
```

---

### Phase 4: Documentation & Release

#### Commit 13: Documentation Update
```
commit: docs: update documentation for v0.2 plugin system

- Update architecture documentation
- Add plugin development guide
- Add admin-ui setup guide
- Add deployment guide
- Update README

Files:
  docs/next.md (updated)
  docs/architecture/plugin_system_v0.2.md (updated)
  docs/implementation/angular_adminui_setup.md (updated)
  README.md (updated)
```

#### Commit 14: Release v0.2
```
commit: release: v0.2 - plugin system redesign with admin dashboard

- Bump version to 0.2.0
- Update CHANGELOG
- Tag release
- Update Docker image
- Deploy to staging

Files:
  package.json (version bump)
  CHANGELOG.md (new)
  docker/Dockerfile (updated)
  .github/workflows/release.yml (updated)
```

---

## 📊 Git Log Beispiel (nach allen Commits)

```
* commit 14abc123 (HEAD -> main, tag: v0.2.0)
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     release: v0.2 - plugin system redesign with admin dashboard
|
* commit 13def456
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     docs: update documentation for v0.2 plugin system
|
* commit 12ghi789
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     test(integration): add end-to-end tests for plugin system
|
* commit 11jkl012
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(api): add websocket gateway for real-time communication
|
* commit 10mno345
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(api): add plugin management endpoints
|
* commit 9pqr678
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(admin-ui): add websocket support for real-time logs
|
* commit 8stu901
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(admin-ui): add plugin-specific dashboards
|
* commit 7vwx234
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(admin-ui): implement shared plugin components
|
* commit 6yza567
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(admin-ui): add plugin api service and models
|
* commit 5bcd890
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(frontend): scaffold angular admin-ui with standalone components
|
* commit 4efg123
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(plugins): add rustyface plugin for face recognition
|
* commit 3hij456
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(plugins): add candle plugin for hugging face integration
|
* commit 2klm789
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat(core): implement plugin registry and lifecycle management
|
* commit 1nop012
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     fix(build): repair cargo and npm registry access
|
* commit 0qrs345 (tag: v0.1a)
| Author: lofmas <lofmas@bkg.rs>
| Date:   Fri Oct 21 2025
|
|     feat: scaffold bkg platform with plugins and ui
```

---

## 🔀 Branch-Strategie

### Feature Branches

```bash
# Phase 1: Foundation
git checkout -b feature/plugin-registry
git checkout -b feature/candle-plugin
git checkout -b feature/rustyface-plugin

# Phase 2: Frontend
git checkout -b feature/admin-ui-setup
git checkout -b feature/websocket-integration

# Phase 3: Backend
git checkout -b feature/plugin-endpoints
git checkout -b feature/integration-tests

# Phase 4: Release
git checkout -b release/v0.2
```

### Merge Strategy

```bash
# Feature in main mergen
git checkout main
git pull origin main
git merge --no-ff feature/plugin-registry
git push origin main

# Tag für Release
git tag -a v0.2.0 -m "Release v0.2: Plugin System Redesign"
git push origin v0.2.0
```

---

## 📋 Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Typen
- `feat`: Neue Feature
- `fix`: Bug-Fix
- `docs`: Dokumentation
- `test`: Tests
- `refactor`: Code-Refactoring
- `perf`: Performance-Verbesserung
- `chore`: Build, Dependencies, etc.

### Beispiele

```
feat(core): implement plugin registry

- Add Plugin trait definition
- Implement PluginRegistry struct
- Add lifecycle management methods
- Add error handling

Closes #123
```

```
feat(admin-ui): add plugin dashboard components

- Create PluginHeaderComponent
- Create PluginStatsComponent
- Create PluginLogsComponent
- Implement WebSocket integration

Closes #124
```

---

## 🚀 Deployment-Commits

### Staging

```
commit: deploy(staging): v0.2-rc1

- Build Docker image
- Push to registry
- Deploy to staging environment
- Run smoke tests
```

### Production

```
commit: deploy(production): v0.2.0

- Build Docker image
- Push to registry
- Deploy to production
- Update DNS
- Monitor metrics
```

---

## 📊 Commit-Statistik (erwartet)

| Phase | Commits | Zeilen Code | Zeilen Tests |
|-------|---------|------------|--------------|
| Phase 1 | 4 | ~2000 | ~500 |
| Phase 2 | 5 | ~3000 | ~800 |
| Phase 3 | 3 | ~1500 | ~1000 |
| Phase 4 | 2 | ~500 | ~200 |
| **Total** | **14** | **~7000** | **~2500** |

---

## 🔍 Git Commands für Tracking

```bash
# Alle Commits für v0.2
git log --oneline --grep="feat\|fix" v0.1a..v0.2

# Statistik
git log --stat v0.1a..v0.2

# Diff zwischen Versionen
git diff v0.1a v0.2

# Commits pro Author
git shortlog -sn v0.1a..v0.2

# Commits pro Tag
git log --oneline --decorate
```

---

## 📝 CHANGELOG Template

```markdown
# Changelog

## [0.2.0] - 2025-10-21

### Added
- Plugin Registry with lifecycle management
- Candle plugin for Hugging Face integration
- RustyFace plugin for face recognition
- Angular AdminUI with plugin dashboards
- WebSocket support for real-time logs
- Plugin management API endpoints
- Telemetry dashboard

### Fixed
- Cargo registry access (proxy configuration)
- npm registry configuration
- CI/CD pipeline

### Changed
- Refactored plugin architecture
- Updated documentation

### Deprecated
- Old plugin system (v0.1a)

## [0.1a] - 2025-10-15

### Added
- Initial BrainML integration
- Basic plugin system
```

---

**Status**: 📋 Planning  
**Nächste Aktion**: Proxy-Fix → Phase 1 starten  
**Geschätzter Aufwand**: 14 Commits über 4-6 Wochen
