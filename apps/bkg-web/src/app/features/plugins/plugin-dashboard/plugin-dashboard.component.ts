import { Component, OnDestroy, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { Subscription, firstValueFrom } from 'rxjs';
import { ApiService } from '../../../services/api.service';
import { PluginLogEvent, PluginState } from '../../../models/api.models';

interface PluginFeatureDescription {
  title: string;
  description: string;
}

@Component({
  selector: 'app-plugin-dashboard',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  templateUrl: './plugin-dashboard.component.html',
  styleUrls: ['./plugin-dashboard.component.css']
})
export class PluginDashboardComponent implements OnInit, OnDestroy {
  plugins: PluginState[] = [];
  plugin?: PluginState;
  logs: PluginLogEvent[] = [];
  configEditor = '';
  loading = false;
  error: string | null = null;

  private logSubscription?: Subscription;
  private routeSubscription?: Subscription;

  private readonly featureCatalog: Record<string, PluginFeatureDescription[]> = {
    brainml: [
      { title: 'Indexierung', description: 'Dokumente ingestieren und Metadaten an BrainDB weiterleiten.' },
      { title: 'Hybrid-Suche', description: 'Vektor- und Volltextsuche kombinieren, um präzise Antworten zu liefern.' },
      { title: 'Trainingspipelines', description: 'Pipelines verwalten und Trainingsjobs orchestrieren.' },
      { title: 'Statistiken', description: 'Kollektionen, Embedding-Größen und Durchsatz überwachen.' }
    ],
    candle: [
      { title: 'Modelle laden', description: 'Hugging Face Artefakte abrufen und im Speichercache bereithalten.' },
      { title: 'Inference', description: 'Batch- und Streaming-Inferenz für Text- und Multimodalmodelle.' },
      { title: 'Tensor Ops', description: 'Optimierte Tensoroperationen für Downstream-Plug-ins anbieten.' }
    ],
    rustyface: [
      { title: 'Encoding', description: 'Face-Vektoren für Uploads generieren.' },
      { title: 'Suche', description: 'Embedding-Vergleiche und Ähnlichkeitssuche.' },
      { title: 'Datensatzverwaltung', description: 'Referenzdatensätze verwalten und versionieren.' }
    ],
    llmserver: [
      { title: 'Chat-Proxys', description: 'OpenAI-kompatible Chat-Vervollständigungen bereitstellen.' },
      { title: 'Embeddings', description: 'Deterministische Embeddings für BrainML & RepoAgent.' },
      { title: 'Templating', description: 'Konversationsvorlagen verwalten und anwenden.' }
    ],
    repoagent: [
      { title: 'Code-Analyse', description: 'Repository-Checks, Impact-Analysen und Abhängigkeitsgraphen.' },
      { title: 'Suche', description: 'Semantische Code-Suche und Diff-Erzeugung.' },
      { title: 'Patchen', description: 'Gefundene Fixes direkt als Patch bereitstellen.' }
    ],
    apikeys: [
      { title: 'Benutzerverwaltung', description: 'Rollen, Passwörter und Multi-Faktor-Policies pflegen.' },
      { title: 'Schlüssel-Lifecycle', description: 'API-Keys ausstellen, rotieren und auditieren.' },
      { title: 'Session-Gatekeeper', description: 'JWT-Sessions validieren und Scope-Prüfungen durchführen.' }
    ]
  };

  constructor(
    private readonly route: ActivatedRoute,
    private readonly router: Router,
    private readonly api: ApiService
  ) {}

  ngOnInit(): void {
    this.routeSubscription = this.route.paramMap.subscribe((params) => {
      const pluginId = params.get('pluginId');
      if (!pluginId) {
        this.plugin = undefined;
        return;
      }
      void this.load(pluginId);
    });
  }

  ngOnDestroy(): void {
    this.logSubscription?.unsubscribe();
    this.routeSubscription?.unsubscribe();
  }

  get features(): PluginFeatureDescription[] {
    if (!this.plugin) {
      return [];
    }
    return this.featureCatalog[this.plugin.name] ?? [];
  }

  async refresh(): Promise<void> {
    if (!this.plugin) {
      return;
    }
    await this.load(this.plugin.name);
  }

  async start(): Promise<void> {
    if (!this.plugin) {
      return;
    }
    this.error = null;
    try {
      await firstValueFrom(this.api.startPlugin(this.plugin.name));
      await this.load(this.plugin.name);
    } catch (error) {
      this.error = (error as Error).message;
    }
  }

  async stop(): Promise<void> {
    if (!this.plugin) {
      return;
    }
    this.error = null;
    try {
      await firstValueFrom(this.api.stopPlugin(this.plugin.name));
      await this.load(this.plugin.name);
    } catch (error) {
      this.error = (error as Error).message;
    }
  }

  async restart(): Promise<void> {
    if (!this.plugin) {
      return;
    }
    this.error = null;
    try {
      await firstValueFrom(this.api.restartPlugin(this.plugin.name));
      await this.load(this.plugin.name);
    } catch (error) {
      this.error = (error as Error).message;
    }
  }

  async saveConfig(): Promise<void> {
    if (!this.plugin) {
      return;
    }
    this.error = null;
    try {
      const config = JSON.parse(this.configEditor);
      await firstValueFrom(this.api.updatePluginConfig(config));
      await this.load(this.plugin.name);
    } catch (error) {
      this.error = (error as Error).message;
    }
  }

  trackCapability(index: number, capability: string): string {
    return `${index}-${capability}`;
  }

  private async load(pluginId: string): Promise<void> {
    this.loading = true;
    this.error = null;
    this.logSubscription?.unsubscribe();
    this.logs = [];
    try {
      this.plugins = await firstValueFrom(this.api.listPlugins());
      const plugin = this.plugins.find((item) => item.name === pluginId);
      if (!plugin) {
        this.error = `Plug-in \\"${pluginId}\\" wurde nicht gefunden.`;
        this.plugin = undefined;
        return;
      }
      this.plugin = plugin;
      this.configEditor = JSON.stringify(plugin.config, null, 2);
      this.logSubscription = this.api.streamPluginLogs(plugin.name).subscribe({
        next: (event) => {
          this.logs = [...this.logs.slice(-199), event];
        },
        error: (err) => {
          this.error = err.message;
        }
      });
    } catch (error) {
      this.error = (error as Error).message;
      if (this.plugins.length === 0) {
        this.router.navigate(['/plugins']);
      }
    } finally {
      this.loading = false;
    }
  }
}
