# Statusbericht: Codex-System-Core

**Datum:** 2025-10-21  
**Uhrzeit:** 05:50 UTC  
**Autor:** lofmas

## Zusammenfassung
- BrainML-Plug-in vollständig in das bkg.rs-Plug-in-Ökosystem integriert; Capabilities `brainml.index`, `brainml.query`, `brainml.train`, `brainml.stats` und `brainml.admin` werden über den Plugin-Bus exponiert.
- Adapter-Layer zu braindb- und llm-Plug-ins fertiggestellt, sodass Persistenz- und Embedding-Aufrufe konsistent über RPC laufen und keine lokale Datenbank benötigt wird.
- Dokumentationspipeline erweitert: Statusberichte, Changelog-Verlinkungen und Dashboard-Metadaten werden regelmäßig aktualisiert, um Fortschritt und Risiken transparent zu halten.

## Fortschritt & Meilensteine
- Projektfortschritt aktuell auf **64 %**, nachdem BrainML stabil läuft und die Dokumentations-/Reporting-Kette vervollständigt wurde.
- Offene Meilensteine: Proxy-/Registry-Fix für Rust & Node, Persistenz-/Snapshot-Validierung, TLS-Staging-Tests und Telemetrie-Rollout.

## Core-Sprachen & Toolchains
- Backend: TypeScript (NestJS) + SQLite-Integration.
- Plug-ins: Rust (brainml, llmserver), Python (repoagent), Node.js (apikeys).
- Build-/Ops-Stack: Cargo, npm, Angular CLI, Docker Buildx, supervisord, utoipa für OpenAPI, tracing + thiserror für Logging und Fehlerbehandlung.

## WebUI-Technologien
- Angular 17 mit Standalone-Komponenten, Signals-gestützter State-Verwaltung und RxJS-Streams.
- Tailwind CSS und HeadlessUI für Layout, Markdown-Rendering im Chat, WebSocket-basierte Log-Streams.

## Aktive Module & Plugins
<<<<<<< ours
<<<<<<< ours
- `core/backend/gateway` als Plug-in-Host mit Auth, Admin, Chat/Embeddings und Health-Routen.
- Plug-ins: `llmserver` (Chat/Embedding), `repoagent` (Codeanalyse), `apikeys` (Auth & Keyverwaltung), `brainml` (Index/Query/Train/Admin/Stats).
- Gemeinsame Ressourcen: SQLite-Datenbank `/data/bkg.db`, zentrale Plug-in-Konfiguration `core/plugins/plugins.json`, busbasierte RPC-Aufrufe.
=======
- `apps/bkg-api` als Plug-in-Host mit Auth, Admin, Chat/Embeddings und Health-Routen.
- Plug-ins: `llmserver` (Chat/Embedding), `repoagent` (Codeanalyse), `apikeys` (Auth & Keyverwaltung), `brainml` (Index/Query/Train/Admin/Stats).
- Gemeinsame Ressourcen: SQLite-Datenbank `/data/bkg.db`, zentrale Plug-in-Konfiguration `plugins/plugins.json`, busbasierte RPC-Aufrufe.
>>>>>>> theirs
=======
- `apps/bkg-api` als Plug-in-Host mit Auth, Admin, Chat/Embeddings und Health-Routen.
- Plug-ins: `llmserver` (Chat/Embedding), `repoagent` (Codeanalyse), `apikeys` (Auth & Keyverwaltung), `brainml` (Index/Query/Train/Admin/Stats).
- Gemeinsame Ressourcen: SQLite-Datenbank `/data/bkg.db`, zentrale Plug-in-Konfiguration `plugins/plugins.json`, busbasierte RPC-Aufrufe.
>>>>>>> theirs

## Fehlerhafte Komponenten
- `cargo clippy -- -D warnings` und `cargo test --release` scheitern weiterhin an 403-Proxy-Fehlern Richtung crates.io.
- `npm install` schlägt wegen Registry-403 fehl, wodurch Frontend-Builds ohne Mirror nicht automatisierbar sind.

## Tests & Integrationspipeline
- Formatprüfungen (`cargo fmt -- --check`) laufen erfolgreich; BrainML-Hybrid-Query-Tests vorhanden, aber aufgrund Proxy-Blocker aktuell nicht in CI.
- CI-Pipeline weiterhin deaktiviert; Reaktivierung an Proxy-Fix gekoppelt.

## Nächste Schritte
- Proxy-/Registry-Konfiguration für Cargo und npm reparieren, um vollständige Builds und Tests zu ermöglichen.
- RAG-/Hybrid-Integrationstests ausweiten, Persistenz-Snapshots (braindb) automatisieren und deterministische Ranking-Checks ergänzen.
- TLS-/CORS-Staging validieren und Telemetrie (OpenTelemetry/Prometheus) in Admin-UI einbinden.

## Risiken & Schulden
- Abhängigkeit von externen Registries ohne funktionierende Proxy-Konfiguration blockiert CI und Release.
- Persistenzmechanismen noch nicht durch Snapshot-/Restart-Tests abgesichert.
- Fehlende Telemetrie verhindert proaktives Monitoring und Alerting.

## Priorisierte Aufgaben
1. `fix_proxy_build`: Proxy-Konfiguration für Cargo/npm reparieren und Build-Blocker beseitigen.
2. `rag_integration_tests`: End-to-End-RAG-Tests für BrainML verfeinern und deterministische Scoring-Prüfungen ergänzen.
3. `validate_persistence`: Snapshot-Hash-Vergleiche und Restart-Konsistenz abbilden.
4. `reactivate_ci`: Format/Lint/Test-Stufen in GitHub Actions oder alternativer CI erneut aktivieren.
5. `staging_env_tls`: TLS/CORS-Konfiguration in Staging verifizieren.
6. `ui_and_telemetry`: Admin-UI um Telemetrie-/Tracing-Integrationen erweitern.

_Automatisch generiert durch Codex AI Statussystem_
