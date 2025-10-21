# 🤖 MAID (Load Testing & Performance) - Complete Implementation Prompt

**MAID is never too late!**

---

## 📋 Executive Summary

MAID ist die **umbenannte Goose-Plugin-Version** für BKG mit erweiterten Funktionen:
- Load Testing & Performance Optimization
- 5 integrierte Komponenten (Extensions, Recipe Generator, Prompt Library, Recipes, Deeplink Generator)
- Zentrale API-Key Verwaltung (via APIKeys Plugin)
- Hybrid Retrieval Integration (via BrainML)
- Admin Dashboard mit Real-time Monitoring

---

## 🎯 Core Features (aus goose_feat.md)

### ✅ Konfiguration & Setup
- Standardkonfiguration (`config.json`) + Runtime-Override (`config.runtime.json`)
- JSON-Schema für UI-Formular-Erzeugung
- Ziel-Host(s), Anzahl Users, Hatch Rate, Laufzeit, Think Time
- HTTP-Methode, Pfad, Header, Query-Parameter, Body
- Gewichtung (Weights) von Szenarien/Transaktionen
- Scheduler Typ (RoundRobin, Serial, Random)
- TLS-Verifikation, Rate Limiting, History-Retention
- Environment Variables Support
- `.maidignore` Mechanik (analog `.gooseignore`)
- Extension-Allowlist

### ✅ Lifecycle & Steuerung
- REST Endpoints:
  - `POST /api/maid/run` → Test starten
  - `POST /api/maid/stop` → Test stoppen
  - `GET /api/maid/status` → Status abfragen
  - `GET /api/maid/history` → Historie
- Single-Run-Semantik
- Graceful Shutdown
- Plugin Bus Capabilities: `maid.run`, `maid.stop`, `maid.status`, `maid.history`
- Szenarien/Transaktionen mit Gewichtung
- Distributed Testing (Manager/Worker Setup)
- CLI/Flag-Unterstützung

### ✅ Monitoring & Telemetrie
- Echtzeit-Telemetrie via WebSocket/Bus
- CPU-Auslastung, RAM, Request/s, Fehlerquote, Durchsatz
- Live Log-Streaming (info, warn, error, debug)
- Statusmeldungen (Idle, Increase, Maintain, Decrease, Shutdown)
- Ergebnis-Reports (JSON/CSV/HTML)
- Kennzahlen pro Run (RunID, Startzeit, Dauer, #Requests, Fehler)

### ✅ UI / Admin Dashboard
- Menüpunkt `/plugins/maid` im Admin UI
- PluginHeaderComponent (Name, Status, Start/Stop/Restart)
- PluginStatsComponent (Telemetrie)
- PluginLogsComponent (Live Logs)
- PluginConfigComponent (Formular mit Schema)
- MAIDConfigComponent (speziell für MAID)
- Szenarien-Definition Formular
- Historie-Tabelle (RunID, Datum, Dauer, Requests, Fehler)
- Status-Badge, Timer, Fortschritt
- Download/Export von Reports
- Rollen-basierte Steuerung

### ✅ API & Plugin Bus Integration
- Plugin Registration mit Metadata
- REST-Endpoints im Gateway
- Bus-Integration für Capabilities
- Logs und Telemetrie via Bus

### ✅ Persistenz & Historie
- Datenbank-Tabelle `maid_runs` (RunID, plugin_id, config_json, metrics_json, start_time, end_time, status)
- Speicherung von Konfigurationsänderungen
- Trim Mechanismus für Historie (maxHistory)

### ✅ Sicherheit & Rollen
- JWT Authentication & API Key Management (via APIKeys Plugin)
- Rollen: `write:loadtests`, `read:loadtests`
- Audit Logging: Test Start, Stop, Config Change

### ✅ Fehlerbehandlung & Robustheit
- Konfigurationsvalidierung
- Timeout- und Netzwerkfehler-Handling
- Stop/Killswitch Mechanismus
- ulimit/Network-Tuning Hinweise

### ✅ Erweiterbarkeit & Skalierbarkeit
- Distributed/Gaggle Modus Support
- Erweiterbare Szenarien/Tasks
- JSON-Schema-basiertes UI
- CLI/Extensions Support

### ✅ Dokumentation & Benutzerhilfe
- Tooltips und Hilfetexte
- Link zur MAID Dokumentation
- Beispielkonfigurationen

---

## 🔧 5 Integrierte Komponenten

### 1️⃣ Extensions (von block.github.io/goose/extensions)
**Zweck**: Erweitere MAID mit Custom Funktionalität

**Features**:
- Extension Registry
- Custom Load Test Patterns
- Plugin-System für MAID
- MCP Server Integration
- Extension Allowlist Management
- Extension Configuration UI

**API Endpoints**:
```
GET    /api/maid/extensions              # Alle Extensions
POST   /api/maid/extensions              # Extension installieren
GET    /api/maid/extensions/:id          # Extension Details
DELETE /api/maid/extensions/:id          # Extension deinstallieren
POST   /api/maid/extensions/:id/enable   # Extension aktivieren
POST   /api/maid/extensions/:id/disable  # Extension deaktivieren
```

**Dashboard**:
- Extension Marketplace
- Installation Wizard
- Configuration Panel
- Enable/Disable Toggle
- Version Management

---

### 2️⃣ Recipe Generator (von block.github.io/goose/recipe-generator)
**Zweck**: Generiere Load Test Recipes automatisch mit LLM

**Features**:
- LLM-basierte Recipe-Generierung (via BrainML/PostgresML)
- Natural Language Input → Load Test Scenario
- Template-basierte Generierung
- Parameter Suggestion
- Recipe Validation
- Export zu verschiedenen Formaten

**API Endpoints**:
```
POST   /api/maid/recipe-generator/generate    # Recipe generieren
POST   /api/maid/recipe-generator/validate    # Recipe validieren
GET    /api/maid/recipe-generator/templates   # Templates auflisten
POST   /api/maid/recipe-generator/suggest     # Parameter vorschlagen
```

**Dashboard**:
- Recipe Generator Wizard
- Natural Language Input
- Live Preview
- Parameter Tuning
- Save/Export Options

---

### 3️⃣ Prompt Library (von block.github.io/goose/prompt-library)
**Zweck**: Zentrale Verwaltung von Load Test Prompts

**Features**:
- Prompt Template Management
- Categorized Prompts
- Search & Filter
- Prompt Versioning
- Sharing & Collaboration
- Prompt Analytics

**API Endpoints**:
```
GET    /api/maid/prompts                 # Alle Prompts
POST   /api/maid/prompts                 # Neuer Prompt
GET    /api/maid/prompts/:id             # Prompt Details
PUT    /api/maid/prompts/:id             # Prompt aktualisieren
DELETE /api/maid/prompts/:id             # Prompt löschen
GET    /api/maid/prompts/search          # Prompt suchen
POST   /api/maid/prompts/:id/fork        # Prompt forken
```

**Dashboard**:
- Prompt Library Browser
- Search Interface
- Prompt Editor
- Preview Panel
- Sharing Controls
- Usage Analytics

---

### 4️⃣ Recipes (von block.github.io/goose/recipes)
**Zweck**: Vordefinierte Load Test Scenarios

**Features**:
- Recipe Library
- Common Patterns (API, Web, Database)
- Performance Testing Recipes
- Scenario Templates
- Best Practices
- Recipe Customization

**Vordefinierte Recipes**:
- **API Load Testing**: REST API, GraphQL, gRPC
- **Web Load Testing**: Single Page Apps, Multi-page
- **Database Load Testing**: Query Performance, Connection Pooling
- **Real-world Scenarios**: E-commerce, Social Media, Banking
- **Stress Testing**: Ramp-up, Spike, Sustained Load
- **Endurance Testing**: Long-running Tests

**API Endpoints**:
```
GET    /api/maid/recipes                 # Alle Recipes
GET    /api/maid/recipes/:id             # Recipe Details
POST   /api/maid/recipes/:id/clone       # Recipe klonen
POST   /api/maid/recipes/:id/run         # Recipe ausführen
GET    /api/maid/recipes/search          # Recipe suchen
```

**Dashboard**:
- Recipe Catalog
- Recipe Details
- Clone/Customize
- Quick Run
- Results Comparison

---

### 5️⃣ Deeplink Generator (von block.github.io/goose/deeplink-generator)
**Zweck**: Generiere shareable Deeplinks für Load Test Konfigurationen

**Features**:
- URL-basierte Konfiguration
- Shareable Test Links
- Preset Management
- QR Code Generation
- Link Analytics
- Expiration Management

**API Endpoints**:
```
POST   /api/maid/deeplinks               # Deeplink generieren
GET    /api/maid/deeplinks/:id           # Deeplink Details
DELETE /api/maid/deeplinks/:id           # Deeplink löschen
GET    /api/maid/deeplinks/:id/qrcode    # QR Code generieren
GET    /api/maid/deeplinks/analytics     # Analytics
```

**Deeplink Format**:
```
https://bkg.local/maid/run?
  config=<base64_encoded_config>
  &recipe=<recipe_id>
  &preset=<preset_name>
  &expires=<timestamp>
```

**Dashboard**:
- Deeplink Generator
- QR Code Display
- Copy to Clipboard
- Link Analytics
- Expiration Settings

---

## 🔐 API-Key Integration (via APIKeys Plugin)

**Zentrale Key-Verwaltung für MAID:**

```sql
-- API Keys für MAID
INSERT INTO auth.api_keys (
  user_id, name, key_hash, prefix, plugin_name, 
  scopes, metadata
) VALUES (
  user_id, 'MAID Load Test Key', key_hash, 'maid_live_',
  'maid', ARRAY['read', 'write', 'admin'],
  '{"recipe_limit": 100, "concurrent_tests": 5}'::jsonb
);
```

**Scopes**:
- `maid.read` - Lese-Zugriff auf Tests & Results
- `maid.write` - Erstelle/Starte Tests
- `maid.admin` - Verwalte MAID Konfiguration
- `maid.recipes` - Zugriff auf Recipes
- `maid.extensions` - Verwalte Extensions

---

## 💾 BrainML Integration

**Speicherung in BrainML:**

```sql
-- MAID Documents in BrainML
INSERT INTO brainml.documents (
  title, content, plugin_source, metadata, api_key_id
) VALUES (
  'Load Test Result', result_json, 'maid',
  '{"test_id": "...", "recipe": "...", "duration": ...}'::jsonb,
  api_key_id
);
```

**Hybrid Retrieval für MAID:**
- Semantic Search über Test Results
- Recipe Recommendations
- Performance Insights
- Trend Analysis

---

## 📊 Database Schema

```sql
-- MAID Runs Table
CREATE TABLE IF NOT EXISTS maid_runs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    api_key_id UUID REFERENCES auth.api_keys(id),
    recipe_id VARCHAR(255),
    config_json JSONB NOT NULL,
    metrics_json JSONB,
    status VARCHAR(50),  -- running, completed, failed, stopped
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    duration_seconds INT,
    total_requests INT,
    failed_requests INT,
    avg_response_time_ms FLOAT,
    p95_response_time_ms FLOAT,
    p99_response_time_ms FLOAT,
    throughput_rps FLOAT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- MAID Recipes Table
CREATE TABLE IF NOT EXISTS maid_recipes (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    config_json JSONB NOT NULL,
    tags TEXT[],
    created_by UUID REFERENCES auth.users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- MAID Extensions Table
CREATE TABLE IF NOT EXISTS maid_extensions (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version VARCHAR(50),
    enabled BOOLEAN DEFAULT false,
    config_json JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- MAID Deeplinks Table
CREATE TABLE IF NOT EXISTS maid_deeplinks (
    id VARCHAR(255) PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    config_json JSONB NOT NULL,
    qr_code_url VARCHAR(255),
    expires_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);
```

---

## 🎨 Admin Dashboard Routes

```
/plugins/maid                          # MAID Dashboard Home
/plugins/maid/run                      # Start New Test
/plugins/maid/results                  # Test Results
/plugins/maid/recipes                  # Recipe Library
/plugins/maid/recipe-generator         # Recipe Generator
/plugins/maid/extensions               # Extensions
/plugins/maid/prompts                  # Prompt Library
/plugins/maid/deeplinks                # Deeplink Generator
/plugins/maid/settings                 # Configuration
/plugins/maid/analytics                # Analytics & Reports
```

---

## 🔌 WebSocket Topics (Plugin Bus)

```
maid:status              # Real-time Test Status
maid:metrics             # Live Metrics (CPU, Memory, RPS)
maid:logs                # Live Logs Stream
maid:recipe-generated    # Recipe Generation Complete
maid:test-completed      # Test Completion Event
```

---

## 🚀 Implementation Phases

### Phase 1: Core MAID (2 Wochen)
- [ ] Rename Goose → MAID
- [ ] Core Load Testing Engine
- [ ] Admin Dashboard
- [ ] API Endpoints
- [ ] Database Schema
- [ ] APIKeys Integration

### Phase 2: Components (3 Wochen)
- [ ] Extensions System
- [ ] Recipe Generator (LLM-powered)
- [ ] Prompt Library
- [ ] Recipes Library
- [ ] Deeplink Generator

### Phase 3: Integration (2 Wochen)
- [ ] BrainML Integration
- [ ] WebSocket Live Updates
- [ ] Analytics & Reports
- [ ] Testing & QA

### Phase 4: Release (1 Woche)
- [ ] Documentation
- [ ] Deployment
- [ ] v0.2 Release

---

## ✅ Acceptance Criteria

- [x] MAID Plugin vollständig umbenannt
- [x] Alle 5 Komponenten integriert
- [x] API-Key Zentrale Verwaltung
- [x] BrainML Hybrid Retrieval
- [x] Admin Dashboard mit allen Features
- [x] Real-time Monitoring via WebSocket
- [x] Database Schema implementiert
- [x] Alle Goose Features erhalten
- [x] Dokumentation vollständig
- [x] Tests grün

---

## 📚 References

- Goose Guides: https://block.github.io/goose/docs/category/guides
- Goose Extensions: https://block.github.io/goose/extensions
- Goose Recipe Generator: https://block.github.io/goose/recipe-generator
- Goose Prompt Library: https://block.github.io/goose/prompt-library
- Goose Recipes: https://block.github.io/goose/recipes
- Goose Deeplink Generator: https://block.github.io/goose/deeplink-generator

---

**Status**: ✅ Ready for Implementation

**Motto**: MAID is never too late! 🚀
