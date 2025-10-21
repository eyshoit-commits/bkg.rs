# CORE Strukturübersicht (v0.2)

## Zielsetzung

Die bisherige Projektstruktur unter `apps/` und `plugins/` wurde in eine modulare Kernaufteilung überführt. Damit ist klar erkennbar, welche Komponenten zum produktiven Kern gehören und welche Artefakte dem DevOps-Bereich zugeordnet sind. Die neue Struktur unterstützt zukünftige Service-Trennungen (Gateway, Auth, ML-Engine, Vector Store) und erleichtert den Hot-Swap-Betrieb der Plug-ins.

## Verzeichnislayout

```
core/
├── backend/
│   └── gateway/              # NestJS API & Plug-in-Host
├── frontend/
│   └── admin-ui/             # Angular Admin-UI
├── plugins/                  # Alle Plug-ins + plugins.json
│   ├── apikeys/
│   ├── brainml/
│   ├── candle/
│   ├── llmserver/
│   ├── repoagent/
│   └── rustyface/
├── database/                 # Platzhalter für Migrationen (SQLite/PostgreSQL)
└── config/                   # Zentralisierte Konfigurations-Templates

devops/
├── docker/                   # Dockerfile, supervisord, Compose
└── scripts/                  # Start- und Hilfsskripte

models/                       # Modellartefakte (per Download oder Build-Arg)
```

## Wichtige Änderungen

- **Backend-Pfad**: Alle NestJS-Quellen liegen nun unter `core/backend/gateway`. Der Docker-Build, die Tests und Dokumentation wurden auf den neuen Pfad umgestellt.
- **Frontend-Pfad**: Die Angular-Anwendung liegt unter `core/frontend/admin-ui` und behält ihre bisherigen Build-/Serve-Konfigurationen.
- **Plug-ins**: Sämtliche Plug-ins sowie `plugins.json` befinden sich unter `core/plugins`. Der Plug-in-Host lädt Konfigurationen ausschließlich aus diesem Verzeichnis.
- **DevOps**: Docker-Artefakte und Hilfsskripte wurden nach `devops/` verschoben. Skripte ermitteln das Repository-Root dynamisch und funktionieren unabhängig vom lokalen Pfad.
- **Dokumentation**: Verweise in Specs, Statusberichten und Deployment-Notizen nutzen die neuen Pfade.

## Build- & Laufzeitbefehle

```bash
# Backend
cd core/backend/gateway
npm install
npm run build

# Frontend
cd core/frontend/admin-ui
npm install
npm run build -- --configuration production

# Docker Compose
./devops/scripts/docker-start.sh

# Modelle herunterladen
./devops/scripts/download-models.sh
```

## Migration bestehender Umgebungen

1. Lokale Skripte oder CI-Pipelines, die auf `apps/bkg-api` oder `apps/bkg-web` verweisen, müssen auf die neuen Pfade (`core/backend/gateway`, `core/frontend/admin-ui`) aktualisiert werden.
2. Docker-basierte Deployments verwenden nun `devops/docker/Dockerfile` und `devops/docker/docker-compose.yml`.
3. Plug-in-bezogene Pfade in Konfigurationen oder Secrets müssen `core/plugins/<name>` adressieren.
4. Für Hot-Swap-Szenarien lädt der Plug-in-Host Konfigurationen aus `core/plugins/plugins.json`; individuelle Plug-in-Dateien befinden sich darunter im gleichnamigen Ordner.

## Nächste Schritte

- `core/backend/` um zusätzliche Services (`auth`, `ml-engine`, `vector`, `shared`) erweitern.
- Migrationen (`core/database`) konsolidieren und CI-Läufe auf die neue Struktur ausrichten.
- Devcontainer- und Workflow-Definitionen nach `devops/` überführen.
