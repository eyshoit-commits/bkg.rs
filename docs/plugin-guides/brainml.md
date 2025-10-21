# BrainML Plug-in

## Überblick
BrainML ist das Retrieval- und Trainings-Backend von bkg.rs. Das Plug-in stellt Hybrid-Suche, Indexierung sowie Pipeline-Verwaltung bereit und nutzt ausschließlich die Plattform-Dienste (BrainDB und LLM-Fähigkeiten) für Persistenz und Embeddings.

## Fähigkeiten
- `brainml.index` – Dokumente ingestieren, normalisieren und an BrainDB weiterreichen.
- `brainml.query` – Volltext-, Vektor- und Hybrid-Abfragen orchestrieren.
- `brainml.train` – Trainingspipelines planen, versionieren und überwachen.
- `brainml.stats` – Betriebs- und Indexmetriken melden.
- `brainml.admin` – Sammlungen, Indizes und Konfigurationen verwalten.

## Admin-UI
- Route: `/plugins/brainml`
- Lebenszyklus: Start, Stop, Restart, Refresh
- Featurekarten (5): Indexieren, Hybrid-Suche, Trainingspipelines, Statistiken, Verwaltung
- Konfigurationseditor: Bearbeitet `plugins.json`-Einträge und triggert Neustarts
- Logstream: Echtzeit SSE-Feed aus dem Plug-in-Bus

## Abhängigkeiten
- RPC: `db.*`-Capabilities des BrainDB-Plug-ins
- Embeddings: `llm.embed`
- Telemetrie: Plug-in-Bus Health & Log Frames

## Betriebshinweise
- Vor Indexläufen sicherstellen, dass Ziel-Kollektionen existieren (`brainml.admin`).
- Hybrid-Querys setzen voraus, dass sowohl FTS- als auch Vektorindizes konfiguriert sind.
- Trainingsjobs schreiben Snapshots in das zentrale Artefakt-Directory (`/srv/artifacts`).
