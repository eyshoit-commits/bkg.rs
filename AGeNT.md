# bkg.rs

## 📘 **Projektauftrag: Recode von „goose“ zu „bkg“**

Baue das Open-Source-Projekt **[goose](https://github.com/block/goose)** komplett neu auf.
Der neue Name lautet **bkg**.

Das System soll in **einem einzigen Docker-Container** laufen, ohne Standard-Ports (keine 8080, 3000 usw.), und auf einem **modularen Plug-in-System** basieren.
Jede Hauptfunktion – also LLM-Server, RepoAgent-Integration und das API-Key-System – ist als **eigenständiges Plug-in** realisiert, das dynamisch geladen und gesteuert werden kann.

---

### 🧱 Architektur und Aufbau

* **Projektname:** `bkg`
* **Struktur (Monorepo):**

  * `/core/backend/gateway` → Backend als NestJS-Server (Node 20), zugleich Plug-in-Host
  * `/core/frontend/admin-ui` → Angular 17 + Tailwind Admin-UI
  * `/core/plugins` → Alle Plug-ins (llmserver, repoagent, apikeys, brainml, candle, rustyface)
  * `/core/database` → Migrationen & Schema (SQLite/PostgreSQL vorbereitet)
  * `/core/config` → Zentrale Konfigurations-Templates
  * `/devops/docker` → Dockerfile, supervisord, Compose
  * `/devops/scripts` → Hilfsskripte (Start, Modelldownload)
  * `/models` → GGUF-Modelldateien (werden beim Build über `--build-arg` eingebunden)

---

### ⚙️ Allgemeine Anforderungen

* **Ein Container, mehrere Prozesse:**

  * NestJS-Backend (Hauptprozess / Plug-in-Host)
  * Plug-ins werden als Subprozesse vom Host gestartet und verwaltet (nicht über separate Services).
  * Keine festen Ports – jede Komponente wählt automatisch einen freien Port, wenn keine Umgebungsvariable gesetzt ist.
  * Beispiel-Defaults (seltene Werte):

    * `BKG_WEB_PORT=43117`
    * `BKG_API_PORT=43119`
    * `BKG_PLUGIN_BUS_PORT=43121`

* Beim Start wird eine **Port-Tabelle** geloggt, z. B.:

  ```
  Service       Port     Health
  WebApp        43117    /
  API           43119    /health
  llmserver     43123    plugin: llmserver
  repoagent     43125    plugin: repoagent
  apikeys       intern   plugin: apikeys
  ```

---

### 🧩 Plug-in-System

Alle Plug-ins folgen einer gemeinsamen Schnittstelle mit Funktionen wie:

* `start()`, `stop()`, `status()`, `config()`
* Rückgabe der verfügbaren **Fähigkeiten** (Capabilities), z. B. `llm.chat`, `repo.analyze`, `auth.issueKey`
* Kommunikation über eine interne **Plugin-Bus-API** (z. B. WebSocket oder internes HTTP auf `BKG_PLUGIN_BUS_PORT`)

Jedes Plug-in besitzt:

* eine eigene Konfigurationsdatei (`config.json`)
* eine Registrierung in der SQLite-Datenbank
* eine Health-Route, um Status und Port an den Host zu melden

---

### 🤖 Plug-in 1: LLM-Server

* Verwendet **llmserver-rs** von eyshoit-commits (Rust-Projekt).
* Wird im Docker-Build mitkompiliert (`cargo build --release`).
* Modelle werden per Build-Args übergeben:

  * `CHAT_MODEL_FILE=Qwen2-0.5B-Instruct-Q5_K_M.gguf`
  * `EMBEDDING_MODEL_FILE=all-MiniLM-L6-v2-ggml-model-f16.gguf`
* Startbefehl:

  ```
  llmserver \
    --model /srv/models/${CHAT_MODEL_FILE} \
    --embedding-model /srv/models/${EMBEDDING_MODEL_FILE} \
    --prompt-template ${PROMPT_TEMPLATE} \
    --port <auto-frei>
  ```
* Das Plug-in meldet dem Host seinen Port und stellt die Funktionen
  `llm.chat` und `llm.embed` bereit.
* Das API-Gateway des Hauptservers (NestJS) bietet dann:

  * `POST /v1/chat/completions`
  * `POST /v1/embeddings`
    gemäß OpenAI-API-Schema.

---

### 🧰 Plug-in 2: RepoAgent

* Basierend auf **OpenBMB RepoAgent** (Python/FastAPI).
* Läuft als Subprozess im Container.
* Stellt Analyse- und Code-Management-Funktionen bereit (`repo.analyze`, `repo.patch`).
* Kann über die Admin-UI gestartet, gestoppt und konfiguriert werden.
* Unterstützt API-Key-Zuweisung (Keys kommen aus dem apikeys-Plug-in).

---

### 🔑 Plug-in 3: API-Keys (Auth-System)

* Verantwortlich für Benutzer, Rollen, Schlüsselverwaltung und Scopes.
* Speichert Daten in SQLite (`/data/bkg.db`).
* Tabellen:

  * `api_keys (key TEXT, user TEXT, scopes TEXT, created_at INT)`
  * `users (id TEXT, name TEXT, password_hash TEXT)`
* Funktionen (exponiert als Plug-in-Fähigkeiten):

  * `auth.login` → gibt Token zurück
  * `auth.createKey`, `auth.revokeKey`, `auth.listKeys`
  * `auth.validate` → prüft Berechtigung und Scopes
* Alle REST-Routen der Haupt-API sind über `Authorization: Bearer <key>` geschützt.
* Das Plug-in liefert auch UI-Komponenten für den **Admin-Tab**:

  * Key-Verwaltung (Anlegen, Löschen, Anzeigen)
  * Benutzer- und Rollenverwaltung
  * Zuweisung von Keys an Plug-ins

---

### 🖥️ Frontend (Angular-App)

Tabs in der Navigationsleiste:

1. **Chat** – Verbindung zur Backend-API, Streaming-Antworten, Markdown-Render.
2. **Plugins** – Liste aller Plug-ins, Status (aktiv/inaktiv), Start/Stop, Konfiguration (JSON-Form), Logs.
3. **Admin** – Verwaltung der API-Keys, Benutzer, Berechtigungen, Systemstatus (Ports, Modelle, Laufzeiten).

Weitere Anforderungen:

* Sauberes, modernes Design mit Tailwind CSS.
* Responsive Oberfläche (Desktop/Mobil).
* Echtzeit-Log-Viewer für jedes Plug-in (WebSocket).
* Alle UI-Daten kommen über die NestJS-API (keine direkten LLM-Aufrufe).

---

### 🐳 Docker-Container

* Multi-Arch Build (`linux/amd64`, `linux/arm64`)
* Enthält:

  * Rust-Build von llmserver-rs
  * Node/NestJS-Backend
  * Angular-Frontend (fertig kompiliert, statisch bedient)
  * Python mit RepoAgent
  * SQLite als lokaler Speicher
* Start über **supervisord**, das nur den Hauptprozess (Backend) und einen optionalen Sidecar startet.
  LLM- und RepoAgent-Prozesse werden vom Plug-in-Host kontrolliert.
* Umgebungsvariablen:

  * `ADMIN_PASSWORD`
  * `PROMPT_TEMPLATE`
  * `CHAT_MODEL_FILE`
  * `EMBEDDING_MODEL_FILE`
  * `BKG_WEB_PORT`, `BKG_API_PORT`, `BKG_PLUGIN_BUS_PORT`

Beispiel-Build:

```
docker buildx build . \
  --platform linux/amd64,linux/arm64 \
  -t bkg:latest \
  --build-arg CHAT_MODEL_FILE=Qwen2-0.5B-Instruct-Q5_K_M.gguf \
  --build-arg EMBEDDING_MODEL_FILE=all-MiniLM-L6-v2-ggml-model-f16.gguf \
  --build-arg PROMPT_TEMPLATE=chatml
```

Beispiel-Start (mit eigenen Ports):

```
docker run --rm \
  -e ADMIN_PASSWORD="change-me" \
  -e BKG_WEB_PORT=43117 \
  -e BKG_API_PORT=43119 \
  bkg:latest
```

Ohne Port-Angaben wählt das System automatisch freie Ports und zeigt sie beim Start an.

---

### 🧠 Qualitätsanforderungen

* Strenges TypeScript-Typing, Linting und Formatierung.
* Kommentierter, modularer Code.
* Gemeinsames Interface für alle Plug-ins (TypeScript-Definition).
* Tests für Authentifizierung, Plug-in-Lifecycle und LLM-Proxy.
* Sichere Speicherung (Keys gehasht mit bcrypt, keine Klartext-Passwörter).
* Kein Root-User im Container, minimale Systemrechte.
* Sauberer Shutdown: alle Subprozesse stoppen und Status speichern.

---

### 🎯 Ziel

Ein **voll funktionsfähiges, erweiterbares KI-System** mit:

* eingebautem LLM-Server (llmserver-rs),
* integriertem RepoAgent-Plug-in,
* vollwertigem Authentifizierungs- und API-Key-Plug-in,
* moderner Web-UI,
* keinerlei Standard-Ports,
* einfacher Erweiterbarkeit für neue Plug-ins (z. B. Embeddings, Tools, Memory, DB).
