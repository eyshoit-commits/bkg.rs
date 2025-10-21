import { PluginFeatureDescription } from './plugin-feature.model';

export const PLUGIN_FEATURES: Record<string, PluginFeatureDescription[]> = {
  brainml: [
    {
      title: 'Indexieren',
      description: 'Dokumente ingestieren, normalisieren und an BrainDB persistieren.',
    },
    {
      title: 'Hybrid-Suche',
      description: 'FTS- und Vektor-Retrieval kombinieren, um konsistente Rankings zu liefern.',
    },
    {
      title: 'Trainingspipelines',
      description: 'Pipelines planen, Trainingsjobs überwachen und Modellversionen verwalten.',
    },
    {
      title: 'Statistiken',
      description: 'Kollektionen, Embedding-Dimensionen und Latenzen beobachten.',
    },
    {
      title: 'Verwaltung',
      description: 'Sammlungen, Indizes und Schema-Konfigurationen mit dem Admin-API pflegen.',
    },
  ],
  candle: [
    {
      title: 'Modelle laden',
      description: 'Hugging Face Artefakte abrufen und im persistenten Cache bereitstellen.',
    },
    {
      title: 'Quantisierung',
      description: 'Gewichte live quantisieren, um Speicher- und Latenzanforderungen zu reduzieren.',
    },
    {
      title: 'Inference',
      description: 'Batch- und Streaming-Inferenz für Text-, Bild- und Multimodalmodelle anbieten.',
    },
    {
      title: 'Tensor-Ops',
      description: 'Optimierte Tensor-Primitive für Downstream-Plug-ins exportieren.',
    },
    {
      title: 'Monitoring',
      description: 'Durchsatz, Latenzen und Ressourcenverbrauch in Echtzeit verfolgen.',
    },
  ],
  rustyface: [
    {
      title: 'Face-Encoding',
      description: 'Gesichtsdaten extrahieren und als Embeddings an BrainDB übermitteln.',
    },
    {
      title: 'Ähnlichkeitssuche',
      description: 'Bestände auf ähnliche Personen prüfen und Treffer priorisieren.',
    },
    {
      title: 'Datensatzpflege',
      description: 'Enrollments verwalten, Versionen pflegen und Datenbereinigung durchführen.',
    },
    {
      title: 'Drift-Monitoring',
      description: 'Erkennungsqualität beobachten und Schwellenwerte dynamisch anpassen.',
    },
  ],
  llmserver: [
    {
      title: 'Chat-Proxys',
      description: 'OpenAI-kompatible Chat-Vervollständigungen für Clients und Operatoren bereitstellen.',
    },
    {
      title: 'Embeddings',
      description: 'Deterministische Embeddings für BrainML, RepoAgent und Dritt-Workloads erzeugen.',
    },
    {
      title: 'Prompt-Templates',
      description: 'System-, Benutzer- und Tool-Prompts versionieren und verwalten.',
    },
    {
      title: 'Werkzeugbrücke',
      description: 'Function-Calling und Tool-Routing an verbundene Services orchestrieren.',
    },
    {
      title: 'Auslastung',
      description: 'GPU/CPU-Metriken, Warteschlangenlängen und Konversationsdurchsatz verfolgen.',
    },
  ],
  repoagent: [
    {
      title: 'Code-Analyse',
      description: 'Statische und semantische Analysen über Repositories durchführen und Findings sammeln.',
    },
    {
      title: 'Semantische Suche',
      description: 'Quelltext, Tests und Dokumentation mit Vektor- und Pattern-Matching durchsuchen.',
    },
    {
      title: 'Patch-Vorschläge',
      description: 'Automatisierte Fixes als überprüfbare Patches und Diff-Bundles bereitstellen.',
    },
    {
      title: 'Dependency-Graph',
      description: 'Modul- und Paketabhängigkeiten visualisieren und zyklische Risiken markieren.',
    },
    {
      title: 'Knowledge-Sync',
      description: 'Analyseergebnisse mit BrainML synchronisieren und im Control-Plane speichern.',
    },
  ],
  apikeys: [
    {
      title: 'Benutzerverwaltung',
      description: 'Rollen, MFA-Policies und Passwort-Rotationen orchestrieren.',
    },
    {
      title: 'Schlüssel-Lifecycle',
      description: 'API-Keys ausstellen, rotieren, sperren und archivieren.',
    },
    {
      title: 'Scope-Prüfung',
      description: 'Bereichs- und Rollenprüfungen für sämtliche Requests durchführen.',
    },
    {
      title: 'Audit-Events',
      description: 'Zugriffe, Login-Versuche und Konfigurationsänderungen protokollieren.',
    },
  ],
};

export const TOTAL_FEATURE_COUNT = Object.values(PLUGIN_FEATURES).reduce(
  (sum, entries) => sum + entries.length,
  0,
);
