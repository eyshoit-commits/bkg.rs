# RustyFace Plug-in

## Überblick
RustyFace stellt biometrische Funktionen für Gesichtserkennung bereit. Es liefert Encoding, Suche und Datensatzverwaltung für Sicherheits- und Compliance-Szenarien und nutzt BrainDB zur Persistenz.

## Fähigkeiten
- `rustyface.faces.encode` – Gesichter in Embeddings umwandeln.
- `rustyface.faces.search` – Embeddings vergleichen und Treffer priorisieren.
- `rustyface.dataset.manage` – Referenzdatensätze verwalten und versionieren.

## Admin-UI
- Route: `/plugins/rustyface`
- Lebenszyklus: Start, Stop, Restart, Refresh
- Featurekarten (4): Face-Encoding, Ähnlichkeitssuche, Datensatzpflege, Drift-Monitoring
- Konfigurationseditor: Quellenordner, Schwellenwerte und Bus-Timeouts konfigurieren
- Logstream: Streaming von Aufnahme-, Matching- und Audit-Ereignissen

## Abhängigkeiten
- RPC: BrainDB (`db.*`) für Persistenz
- Embeddings: `llm.embed` bei fallback-Szenarien für Texte
- Datenschutz: Audit-Trail über das API-Key-Plug-in

## Betriebshinweise
- Drift-Monitoring regelmäßig prüfen, um Modellverschlechterungen zu erkennen.
- Datensatzänderungen triggern automatische Re-Indexierung; diese kann in der Admin-UI überwacht werden.
- API-Scopes streng verwalten (`faces.*`), um Missbrauch zu verhindern.
