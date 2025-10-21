# Candle Plug-in

## Überblick
Das Candle-Plug-in integriert die Rust-basierte Candle-Laufzeit von Hugging Face. Es liefert Modell-Serving, Tensor-Operationen und Monitoring für rechenintensive Pipelines innerhalb des bkg.rs Ökosystems.

## Fähigkeiten
- `candle.model.load` – Modelle aus Remote- oder lokalen Repositories laden.
- `candle.model.run` – Inferenzjobs für Text-, Bild- und Multimodalmodelle ausführen.
- `candle.tensor.ops` – Optimierte Tensor-Primitive für abhängige Plug-ins bereitstellen.
- `candle.stats` – Laufzeitmetriken und Ressourcenauslastung veröffentlichen.

## Admin-UI
- Route: `/plugins/candle`
- Lebenszyklus: Start, Stop, Restart, Refresh
- Featurekarten (5): Modelle laden, Quantisierung, Inference, Tensor-Ops, Monitoring
- Konfigurationseditor: Model-Cache, Build-Targets und Startargumente verwalten
- Logstream: Build-, Lade- und Inferenzlogs in Echtzeit

## Abhängigkeiten
- Artefakte: Zugriff auf `/srv/models` gemäß Docker-Build-Args
- RPC: Optionaler Austausch mit BrainML (`brainml.index`) und RepoAgent (`repoagent.*`)
- Telemetrie: GPU/CPU Sensoren über den Plug-in-Bus

## Betriebshinweise
- Modelle vor dem Start via Admin-UI herunterladen oder in `config.json` vorgeben.
- Quantisierung sollte außerhalb von Peak-Lastfenstern erfolgen, da sie CPU-intensiv ist.
- Monitoring-Alerts lassen sich über das Admin-Dashboard an Prometheus weiterreichen.
