# LLMServer Plug-in

## Überblick
LLMServer kapselt den Rust-basierten llmserver-rs und fungiert als OpenAI-kompatible Proxy-Schicht. Es bedient Chat- und Embedding-Anfragen für BrainML, RepoAgent und externe Clients.

## Fähigkeiten
- `llm.chat` – Chat-Completions via `/v1/chat/completions`.
- `llm.embed` – Embedding-Generierung via `/v1/embeddings`.

## Admin-UI
- Route: `/plugins/llmserver`
- Lebenszyklus: Start, Stop, Restart, Refresh
- Featurekarten (5): Chat-Proxys, Embeddings, Prompt-Templates, Werkzeugbrücke, Auslastung
- Konfigurationseditor: Modellpfade, Prompt-Templates und Workeranzahl verwalten
- Logstream: Tokenstatistiken, Pipeline-Latenzen und Tool-Calls beobachten

## Abhängigkeiten
- Modelle: Build-Args `CHAT_MODEL_FILE`, `EMBEDDING_MODEL_FILE`
- RPC: optionaler Zugriff auf RepoAgent (`repoagent.*`) für Tool-Einsätze
- Auth: `apikeys`-Plug-in für Tokens und Scope-Gates

## Betriebshinweise
- Prompt-Templates versionieren, bevor neue Modelle deployed werden.
- Auslastungsmetriken beobachten; bei hoher Nachfrage horizontale Skalierung via Supervisord-Profile nutzen.
- Tool-Aufrufe (Function Calling) benötigen gültige Capability-Verknüpfungen.
