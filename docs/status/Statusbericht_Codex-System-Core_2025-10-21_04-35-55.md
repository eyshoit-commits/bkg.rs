# Statusbericht: Codex-System-Core

**Datum:** 2025-10-21  
**Uhrzeit:** 04:35 UTC  
**Autor:** lofmas

## Zusammenfassung
- NestJS-Gateway `apps/bkg-api` orchestriert das Plug-in-Ökosystem, verwaltet Subprozesse und exponiert OpenAI-kompatible Routen plus Admin- und Health-APIs.  
- Vier Plug-ins (Rust-LLM, RepoAgent, API-Keys, BrainML) registrieren ihre Fähigkeiten, streamen Logs und interagieren über den Plugin-Bus mit der SQLite-Konfiguration.  
- Die Angular-17/Tailwind-Oberfläche bietet Chat-, Plug-in- und Admin-Ansichten inklusive Live-Log-Streaming; Docker-Assets bündeln alle Komponenten zu einem Multiservice-Container.

## Fortschritt & Meilensteine
- Gesamtfortschritt geschätzt bei **62 %** nach Implementierung der BrainML-Portierung, Bus-RPC-Erweiterungen und Dokumentationsautomatisierung.  
- Kernel-Meilensteine (Gateway, Plug-ins, UI, Docker, BrainML) abgeschlossen; verbleibend: ausstehende Integrationstests, Persistenz-Validierungen und Pipeline-Härtung.

## Core-Sprachen & Toolchains
- **TypeScript/NestJS** für das Gateway und Plug-in-Host-Management.  
- **Rust** für BrainML und LLM-Server, **Python/FastAPI** für RepoAgent, **TypeScript/Angular** für das Frontend.  
- Build-/QA-Tooling: `cargo`, `npm`, `ng`, `jest`, `sqlite`, Docker Buildx, supervisord.

## WebUI-Technologien
- Angular 17, RxJS, Router, HttpClient, Signals-basierte State-Verwaltung.  
- Tailwind CSS für Layout und Komponenten, Markdown-Rendering im Chat, WebSocket-Streams für Plug-in-Logs.

## Aktive Module & Plugins
- `bkg-api` Host (Plugin-Bus, Auth-Guard, Admin/Chat/Health-Routen).  
- Plug-ins: `llmserver` (Chat+Embedding), `repoagent` (Repo-Analyse), `apikeys` (Auth/API-Key), `brainml` (Index/Query/Train/Admin/Stats).  
- Gemeinsame SQLite-Konfiguration (`/data/bkg.db`) plus zentrale `plugins/plugins.json` für Autostart und Capabilities.

## Fehlerhafte Komponenten
- Rust-Build-Pipeline: `cargo clippy -- -D warnings` und `cargo test --release` scheitern aufgrund eines 403-Proxy-Blocks auf crates.io; Code selbst kompiliert lokal.  
- Node-Abhängigkeiten lassen sich in der Sandbox nicht von der Registry beziehen (`npm install` 403), weshalb CI-Läufe ohne externen Cache blockiert bleiben.

## Tests & Integrationspipeline
- Format-Checks (`cargo fmt -- --check`) und Jest-Suites lokal grün; automatisierte E2E-/Integrationstests für BrainML hybrid query existieren, laufen jedoch aktuell nicht in CI wegen Proxy-Sperre.  
- GitHub-Actions/CI ist deaktiviert; Wiedereinführung nach Proxy-Fix geplant.

## Nächste Schritte
- Proxy-/Registry-Probleme für Rust- und Node-Toolchains beheben, damit Clippy/Test/Install wiederholbar laufen.  
- RAG/Hybrid-Suchintegrationstests erweitern (z. B. deterministische Score-Validierung) und Persistenz-Snapshot-Checks implementieren.  
- Staging-Umgebung mit TLS/CORS validieren, Admin-UI Telemetrie (OpenTelemetry/Prometheus) aktivieren.

## Risiken & Schulden
- Abhängigkeit von externen Registries ohne Mirror blockiert CI und Auslieferung.  
- Persistenz- und Snapshot-Pfade (ironstore/braindb) noch nicht automatisiert getestet.  
- Fehlende Telemetrie/Monitoring erschwert Incident-Response.

## Priorisierte Aufgaben
1. `fix_proxy_build`: Proxy-Konfiguration reparieren, damit Cargo/NPM-Registries erreichbar werden.  
2. `rag_integration_tests`: BrainML-RAG-Flows mit deterministischen Assertions absichern.  
3. `validate_persistence`: Snapshot-Hash-Vergleiche und Neustart-Resilienz prüfen.  
4. `reactivate_ci`: CI-Pipeline mit Format/Lint/Test (cargo fmt, clippy, test) reaktivieren.  
5. `staging_env_tls`: TLS/CORS-Staging überprüfen.  
6. `ui_and_telemetry`: Admin-UI mit Telemetrie-/Tracing-Backends verbinden.

_Automatisch generiert durch Codex AI Statussystem_
