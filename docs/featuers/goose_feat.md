Hier ist eine **komplette Liste aller Features**, die beim Re-Coding des Goose-Plug-ins importiert bzw. wiederhergestellt werden **müssen**, damit keine Funktionalität vergessen wird. Die Liste basiert auf offiziellen Goose-Dokumenten. ([book.goose.rs][1])

---

### 🧮 Vollständige Feature-Liste

#### Konfiguration & Setup

* Standardkonfiguration (`config.json`) plus Runtime-Override (`config.runtime.json`)
* JSON-Schema für UI-Formular-Erzeugung
* Konfigurierbare Parameter:

  * Ziel-Host(s) / Base URL
  * Anzahl Users (GooseUsers)
  * Hatch Rate (Users/sec)
  * Laufzeit / Duration
  * Think Time (Wartezeit zwischen Requests)
  * HTTP-Methode, Pfad, Header, Query-Parameter, Body
  * Gewichtung (Weights) von Szenarien/Transaktionen
  * Scheduler Typ (RoundRobin, Serial, Random) ([Docs.rs][2])
  * TLS-Verifikation on/off
  * Rate Limiting / Durchsatzbegrenzung
  * History-Retention (maxHistory)
  * Logging-Level / Logausgabeort
  * Environment Variables Unterstützung ([block.github.io][3])
  * Unterstützen einer `.gooseignore`-Mechanik oder ähnlichem Filter
  * Extension-Allowlist, Extensions-Mechanismen ([block.github.io][3])

#### Lifecycle & Steuerung

* REST Endpoints (über Gateway):

  * `POST /api/goose/run` → Starten eines Tests
  * `POST /api/goose/stop` → Stoppen eines Tests
  * `GET /api/goose/status` → Aktueller Status abfragen
  * `GET /api/goose/history` → Historie früherer Läufe
* Single-Run-Semantik (nur ein aktiver Test zurzeit)
* Graceful Shutdown des Tests
* Integriertes Capability-Muster über Plug-in Bus:

  * `goose.run`, `goose.stop`, `goose.status`, `goose.history`
* Szenarien/Transaktionen starten, Gewichtung berücksichtigen
* Scheduler Steuerung (siehe oben)
* Unterstützung für verteiltes Testing („Gaggle“) – Manager/Worker Setup ([book.goose.rs][4])
* CLI/Flag-Unterstützung (optional) wie bei Goose standardmäßig

#### Monitoring & Telemetrie

* Echtzeit-Telemetrie via WebSocket oder Bus:

  * CPU-Auslastung, RAM-Verbrauch, Request/s, Fehlerquote, Durchsatz
* Live Log-Streaming (info, warn, error, debug)
* Statusmeldungen: z. B. Phase (Idle, Increase, Maintain, Decrease, Shutdown) ([Docs.rs][2])
* Ergebnis-Reports am Laufende oder nach Abschluss (JSON/CSV/HTML)
* Speicherung von Kennzahlen pro Run (RunID, Startzeit, Dauer, #Requests, Fehler)

#### UI / Admin Dashboard

* Menüpunkt `/plugins/goose` im Admin UI
* Komponenten:

  * PluginHeaderComponent (Name, Status, Buttons: Start/Stop/Restart)
  * PluginStatsComponent (Telemetrie)
  * PluginLogsComponent (Live Logs-Stream)
  * PluginConfigComponent (Formular mit Schema)
  * GooseConfigComponent (speziell für Goose Einstellungen)
* Formular zur Szenarien-Definition: HTTP Methode, Pfad, Header, Body, ThinkTime, Gewichtung
* Tabelle mit Historie der Läufe (RunID, Datum, Dauer, Requests, Fehler)
* Visuelle Anzeige: Status-Badge (z. B. „running“, „stopped“), Timer, Fortschritt
* Download/Export von Reports
* Rollen-basierte Steuerung (nur berechtigte Nutzer darf starten/stoppen)

#### API & Plug-in Bus Integration

* Registration des Plug-ins mit Metadata: ID, Name, Version, Capabilities, Schema
* REST-Endpoints im Gateway, die auf Plug-in Bus-Capabilities abbilden
* Bus-Integration: Logik für `goose.run`, `goose.stop`, etc.
* Nutzung von Bus für Logs und Telemetrie

#### Persistenz & Historie

* Datenbank-Tabelle (z. B.) `goose_runs`: RunID uuid, plugin_id, config_json, metrics_json, start_time, end_time, status
* Speicherung von Konfigurationsänderungen (`config.runtime.json`)
* Trim Mechanismus für Historie (maxHistory)

#### Sicherheit & Rollen

* JWT Authentication & API Key Management im Gateway
* Rollen wie `write:loadtests`, `read:loadtests`
* Audit Logging: Test Start, Stop, Config Change

#### Fehlerbehandlung & Robustheit

* Konfigurationsvalidierung (z. B. gültige URL, Wertebereiche)
* Timeout- und Netzwerkfehler-Handling (z. B. Host unreachable)
* Stop bzw. Killswitch Mechanismus ([Docs.rs][2])
* ulimit/Network-Tuning Hinweise für hohe Lastszenarien (optional)

#### Erweiterbarkeit & Skalierbarkeit

* Support für Distributed/Gaggle Modus
* Erweiterbare Szenarien/Tasks (.gooseignore Mechanik, Extensions)
* JSON-Schema-basiertes UI macht Erweiterung ohne Code-Change möglich
* CLI/Extensions Support

#### Dokumentation & Benutzerhilfe

* Tooltips und Hilfetexte in der UI
* Link zur offiziellen Goose Dokumentation
* Beispielkonfigurationen (z. B. einfaches Szenario, komplexes Szenario)

---

Wenn du möchtest, kann ich **diese Liste als JSON-Konfiguration** generieren, sodass dein Codegenerator/LLM-Agent sie automatisch als Anforderungen akzeptiert und implementiert.

[1]: https://book.goose.rs/?utm_source=chatgpt.com "The Goose Book: What Is Goose?"
[2]: https://docs.rs/goose/latest/src/goose/lib.rs.html?utm_source=chatgpt.com "lib.rs - source"
[3]: https://block.github.io/goose/docs/guides/using-goosehints/?utm_source=chatgpt.com "Providing Hints to Goose - GitHub Pages"
[4]: https://book.goose.rs/glossary.html?utm_source=chatgpt.com "Glossary - The Goose Book"
Gut gesehen — du hast völlig recht. Es gibt noch **wichtige Features aus den „Tutorials“ und „Getting Started“-Abschnitten** von Goose, die in der bisherigen Checkliste nicht enthalten waren.

Ich ergänze sie hier — sodass dein Re-Coding-Plan lückenlos alle Features abdeckt.

---

### ✅ Zusätzliche Features aus Tutorials / Getting Started

Basierend auf den Seiten:

* Quickstart / Installation ([block.github.io][1])
* Extensions & Hints (z. B. `.goosehints`) ([block.github.io][2])
* CLI Commands ([block.github.io][3])

**Neu zu berücksichtigen:**

* Installation/Setup-Flows: Unterstütze Installationen (Desktop/CLI) → Deine Plug-in sollte Deployment/Install-Dokumentation haben.
* Extensions-Mechanismus: Dein Plug-in sollte Extension-Hook-Mechanismus unterstützen, falls Goose Erweiterungen nutzt.
* Hint-Files (`.goosehints`): Umgebungskontext & Konfigurationsdateien, dein Plug-in muss in Konfigurations-Mechanismus Hinweise unterstützen.
* CLI-Unterstützung: Neben REST/Bus eventuell CLI-Integration (z. B. `goose-plugin run`) oder zumindest Design dafür.
* Scheduled Tasks / Recipes: Reusable Workflows, planen von Läufen.
* Environment Variables & Shared Configurations (z. B. LLM-Agent Mechanismus) – dein Plug-in-Config sollte Umgebungsvariable-Overrides unterstützen.

---

Wenn du willst, kann ich **die vollständige aktualisierte Liste inklusive dieser Punkte** in ein JSON-Schema ausgeben, damit dein Entwickler-Agent sie direkt übernimmt.

---

## 📚 Goose Guides (von block.github.io/goose/docs/category/guides)

### 🗃️ Managing Sessions
- Session Management
- Session Configuration
- Session Persistence

### 🗃️ Recipes
- Common Load Testing Patterns
- Performance Testing Recipes
- Scenario Templates
- Best Practices

### 📄️ Managing Projects
- Project Structure
- Working Directory Management
- Context Preservation
- Multi-codebase Support

### 🗃️ Managing Tools
- Tool Integration
- Extension Management
- Custom Tools

### 📄️ Updating Goose
- Version Management
- Update Procedures
- Compatibility Checks
- Release Notes

### 📄️ Goose Permissions
- Permission Modes
- File Modification Control
- Extension Permissions
- Automated Action Control

### 📄️ Quick Tips
- Best Practices
- Performance Optimization
- Common Patterns
- Troubleshooting

### 📄️ CLI Commands
- Command Reference
- Interactive Session Features
- Configuration Management
- Session Control

### 📄️ Using Goosehints
- `.goosehints` File Format
- Project Context
- Communication Improvement
- Task Execution Enhancement

### 🗃️ Security
- Security Best Practices
- API Key Management
- Access Control

### 📄️ CLI Providers
- Claude Code Integration
- Cursor Agent Support
- Gemini CLI Integration
- Provider Configuration

### 📄️ Subagents
- Independent Task Execution
- Process Isolation
- Context Preservation
- Temporary Assistants

### 🗃️ Rich Interactive Chat
- Chat Interface
- Interactive Features
- Real-time Feedback

### 📄️ LLM Rate Limits
- Rate Limit Handling
- Request Throttling
- Quota Management
- Error Recovery

### 📄️ Logging System
- Conversation Storage
- Local Storage Locations
- Log Management
- Data Retention

### 📄️ File Management
- File Discovery
- File References
- Safe File Operations
- Best Practices

### 📄️ Run Tasks
- Task Execution
- File Passing
- Instruction Handling
- Workflow Automation

### 📄️ Using Gooseignore
- `.gooseignore` File Format
- File Exclusion Patterns
- Directory Filtering
- Access Restrictions

### 📄️ Configuration File
- YAML Configuration
- Settings Management
- Extension Configuration
- Custom Settings

### 📄️ Extension Allowlist
- MCP Server Control
- Installation Restrictions
- Corporate Settings
- Security Policies

### 📄️ Environment Variables
- Variable Configuration
- Behavior Customization
- Provider Settings
- Custom Parameters

### 🗃️ Multi-Model Config
- Multiple Model Support
- Provider Configuration
- Model Switching Strategies
- Load Balancing

### 📄️ Goose in ACP Clients
- Agent Client Protocol
- Native Integration
- Client Applications
- Seamless Interaction

### 📄️ Enhanced Code Editing
- AI-powered Code Changes
- Intelligent Application
- Code Modification
- Automated Refactoring

---

[1]: https://block.github.io/goose/docs/quickstart/?utm_source=chatgpt.com "Quickstart | codename goose - GitHub Pages"
[2]: https://block.github.io/goose/docs/guides/using-goosehints/?utm_source=chatgpt.com "Providing Hints to Goose | codename goose - GitHub Pages"
[3]: https://block.github.io/goose/docs/guides/goose-cli-commands/?utm_source=chatgpt.com "CLI Commands | codename goose - GitHub Pages"
