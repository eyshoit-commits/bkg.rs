# Statusbericht: Codex System Core
**Datum:** 2025-10-21  
**Uhrzeit:** 04:25 UTC  
**Autor:** lofmas

## Zusammenfassung
- Der NestJS-Host lädt Plug-in-Konfigurationen aus `plugins.json`, startet Autostart-Dienste, synchronisiert Laufzeitstatus mit dem WebSocket-Bus und verwaltet Subprozesse inkl. Fehlersicherung.【F:core/backend/gateway/src/plugins/plugin.service.ts†L1-L200】【F:core/backend/gateway/src/plugins/plugin-bus.service.ts†L1-L176】
- Die Angular-17-Konsole bietet Chat-, Plug-in- und Admin-Workflows; sie ruft die OpenAI-kompatiblen API-Routen auf, verwaltet Plug-in-Logstreams per Server-Sent Events und erlaubt Konfigurationsupdates.【F:core/frontend/admin-ui/src/app/components/chat/chat.component.ts†L1-L45】【F:core/frontend/admin-ui/src/app/components/plugins/plugins.component.ts†L1-L96】【F:core/frontend/admin-ui/src/app/services/api.service.ts†L1-L90】
- Das brainml-Plug-in registriert Capabilities für Indexierung, Suche, Training, Statistiken und Administration, stellt HTTP-Endpunkte bereit und orchestriert RPCs zum Datenbank- und LLM-Busadapter.【F:core/plugins/brainml/src/main.rs†L18-L200】【F:core/plugins/brainml/src/api/mod.rs†L1-L115】【F:core/plugins/brainml/src/adapters/braindb.rs†L1-L200】

## Fortschritt & Meilensteine
- Vier Plug-ins (llmserver, repoagent, apikeys, brainml) sind konfiguriert, inklusive Health-Checks und Capability-Deklarationen.【F:core/plugins/plugins.json†L1-L53】
- Brainml deckt Indexierung, Hybrid-Retrieval und Pipelines ab; Integrationstests validieren Index- und Query-Flows mit deterministischem Ranking.【F:core/plugins/brainml/src/api/mod.rs†L42-L115】【F:core/plugins/brainml/tests/integration.rs†L1-L73】
- Das Control-Plane-Dashboard unterstützt Start/Stop und Konfigurationspflege, womit der Plug-in-Lifecycle Ende-zu-Ende abgedeckt ist.【F:core/frontend/admin-ui/src/app/components/plugins/plugins.component.ts†L34-L96】

## Toolchains & Programmiersprachen
- Backend und Frontend setzen auf TypeScript/NestJS bzw. Angular mit Tailwind-Build-Tooling.【F:core/backend/gateway/package.json†L1-L43】【F:core/frontend/admin-ui/package.json†L1-L41】
- brainml und llmserver liefern Rust-basierte Dienste mit Tokio/Axum; brainml bindet utoipa/OpenAPI ein.【F:core/plugins/brainml/Cargo.toml†L1-L42】
- Repoagent ergänzt eine FastAPI-Python-Implementierung, die über websockets den Bus bedient.【F:core/plugins/repoagent/pyproject.toml†L1-L15】

## Aktive Module & Plugins
- Der PluginService pflegt Laufzeitstatus, Ports und Capabilities aller registrierten Plug-ins, wodurch die Admin-UI stets aktuelle Zustände erhält.【F:core/backend/gateway/src/plugins/plugin.service.ts†L59-L188】
- brainml veröffentlicht `brainml.*`-Capabilities auf dem Bus und konsumiert `db.*`- und `llm.embed`-RPCs über dedizierte Adapter.【F:core/plugins/brainml/src/main.rs†L90-L200】【F:core/plugins/brainml/src/adapters/braindb.rs†L159-L200】
- Die übrigen Plug-ins bleiben konfiguriert und steuerbar; Logs und Health-Signale laufen über den zentralen Bus.【F:core/backend/gateway/src/plugins/plugin-bus.service.ts†L53-L176】【F:core/plugins/plugins.json†L1-L53】

## Fehlerhafte oder blockierende Komponenten
- `cargo clippy` scheiterte beim Laden der crates.io-Indexdateien (HTTP 403 über den Proxy), wodurch statische Analysen derzeit blockiert sind.【d2475c†L1-L10】
- `cargo test --release` schlägt aus demselben Grund fehl, sodass keine vollständige Testausführung möglich ist.【e1487e†L1-L8】

## Tests & Integrationspipeline
- Formatprüfungen (`cargo fmt -- --check`) laufen erfolgreich im brainml-Projekt.【7c19fc†L1-L3】
- Asynchrone Integrationstests für brainml belegen Hybrid-Query-Funktionalität, erfordern jedoch funktionierende Builds, um im CI zu laufen.【F:core/plugins/brainml/tests/integration.rs†L29-L73】
- Weitere Checks (clippy, release-tests) bleiben durch die Proxy-Sperre deaktiviert.【d2475c†L1-L10】【e1487e†L1-L8】

## Geplante nächste Schritte
- Proxy-/Netzwerkkonfiguration anpassen oder Dependencies vendoren, damit `cargo clippy` und `cargo test` ohne 403-Blockaden laufen können.【d2475c†L1-L10】【e1487e†L1-L8】
- brainml mit einem realen `db.*`-Backend verknüpfen, indem der PluginBus-Braindb-Client an das produktive Speichersystem gebunden wird.【F:core/plugins/brainml/src/adapters/braindb.rs†L159-L200】
- Admin-Workflows erweitern, um Brainml-Pipeline- und Trainingsstatus visuell darzustellen (derzeit nur via RPC abrufbar).【F:core/plugins/brainml/src/api/mod.rs†L98-L115】【F:core/frontend/admin-ui/src/app/components/plugins/plugins.component.ts†L69-L96】

## Risiken & Technische Schulden
- Die Nutzung des NullBraindbClient als Fallback maskiert fehlende Persistenzintegration und sollte durch produktive RPCs ersetzt werden.【F:core/plugins/brainml/src/adapters/braindb.rs†L66-L157】
- Ohne gelöste Proxy-Einstellungen bleibt die CI-Pipeline für Rust-Komponenten unvollständig.【d2475c†L1-L10】【e1487e†L1-L8】
- Mehrere Capabilities (z. B. Training) bieten noch keine UI-Unterstützung, wodurch Bedienfehler drohen.【F:core/plugins/brainml/src/api/mod.rs†L98-L115】【F:core/frontend/admin-ui/src/app/components/plugins/plugins.component.ts†L69-L96】

## Priorisierte Aufgaben
1. Proxy/Registry-Problem beheben, um clippy/tests zuverlässig auszuführen und CI wieder zu aktivieren.【d2475c†L1-L10】【e1487e†L1-L8】
2. Braindb- und LLM-Busadapter gegen echte Dienste testen, inklusive Erfolg-/Fehlerpfade und OpenAPI-Validierung.【F:core/plugins/brainml/src/adapters/braindb.rs†L159-L200】【F:core/plugins/brainml/src/api/mod.rs†L42-L115】
3. Admin-UI um Brainml-spezifische Statusanzeigen und Pipeline-Steuerung ergänzen, damit Operatoren Trainings- und Statistikdaten ohne API-Aufrufe sehen.【F:core/frontend/admin-ui/src/app/components/plugins/plugins.component.ts†L69-L96】【F:core/plugins/brainml/src/api/mod.rs†L98-L115】

_Automatisch generiert durch Codex AI Statussystem_
