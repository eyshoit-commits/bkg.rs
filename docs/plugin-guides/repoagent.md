# RepoAgent Plug-in

## Überblick
RepoAgent ist ein Python/FastAPI-Service für semantische Codeanalyse. Er liefert Suche, Patch-Vorschläge und Wissenssynchronisation für Entwickler-Workflows.

## Fähigkeiten
- `repoagent.code.analyze` – Statische, semantische und heuristische Analysen fahren.
- `repoagent.code.search` – Code-, Test- und Dokumentationssuche mit Embedding-Unterstützung.

## Admin-UI
- Route: `/plugins/repoagent`
- Lebenszyklus: Start, Stop, Restart, Refresh
- Featurekarten (5): Code-Analyse, Semantische Suche, Patch-Vorschläge, Dependency-Graph, Knowledge-Sync
- Konfigurationseditor: Repository-Roots, Include-/Exclude-Filter, Cache-Einstellungen
- Logstream: Streaming von Analyseergebnissen, Patch-Generierungen und Fehlern

## Abhängigkeiten
- Filesystem: Schreib-/Leserechte auf den analysierten Repos
- RPC: BrainML (`brainml.index`/`brainml.query`) für Wissensspeicherung
- Auth: API-Key-Scopes `repoagent.*`

## Betriebshinweise
- Große Repositories sollten über dedizierte Worker-Profile mit erhöhter Timeout-Konfiguration laufen.
- Knowledge-Sync kann automatisiert werden, indem Pipelines im BrainML-Dashboard geplant werden.
- Bei Patch-Vorschlägen stets Review-Workflow aktivieren, um Fehleinspielungen zu vermeiden.
