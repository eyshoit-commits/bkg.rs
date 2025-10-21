import { Component, OnDestroy, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { Subscription, firstValueFrom } from 'rxjs';
import { ApiService } from '../../../services/api.service';
import {
  PluginConfig,
  PluginLogEvent,
  PluginState,
  PluginTelemetry,
  RepoAgentSettings,
} from '../../../models/api.models';
import { PluginWsService } from '../../../services/plugin-ws.service';
import { RepoagentConfigComponent } from '../repoagent/repoagent-config.component';

interface PluginFeatureDescription {
  title: string;
  description: string;
}

@Component({
  selector: 'app-plugin-dashboard',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, RepoagentConfigComponent],
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
  telemetry?: PluginTelemetry;
  currentConfig?: PluginConfig;
  repoAgentSettings?: RepoAgentSettings;

  private logSubscription?: Subscription;
  private routeSubscription?: Subscription;
  private statusSubscription?: Subscription;
  private telemetrySubscription?: Subscription;
  private bootstrapSubscription?: Subscription;

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
    private readonly api: ApiService,
    private readonly ws: PluginWsService,
  ) {}

  ngOnInit(): void {
    this.bootstrapSubscription = this.ws.bootstrap().subscribe(({ plugins, telemetry }) => {
      this.plugins = plugins;
      if (this.plugin) {
        const updated = plugins.find((item) => item.name === this.plugin?.name);
        if (updated) {
          this.plugin = updated;
          this.updateConfigState(updated.config);
        }
      }
      if (Array.isArray(telemetry)) {
        const latest = telemetry.find((snapshot) => snapshot.plugin === this.plugin?.name);
        if (latest) {
          this.telemetry = latest;
        }
      }
    });
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
    this.statusSubscription?.unsubscribe();
    this.telemetrySubscription?.unsubscribe();
    this.bootstrapSubscription?.unsubscribe();
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
      const config = JSON.parse(this.configEditor) as PluginConfig;
      this.currentConfig = config;
      if (this.plugin?.name === 'repoagent') {
        this.repoAgentSettings = this.extractRepoAgentSettings(config);
      }
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
    this.statusSubscription?.unsubscribe();
    this.telemetrySubscription?.unsubscribe();
    this.logs = [];
    this.telemetry = undefined;
    try {
      this.plugins = await firstValueFrom(this.api.listPlugins());
      const plugin = this.plugins.find((item) => item.name === pluginId);
      if (!plugin) {
        this.error = `Plug-in "${pluginId}" wurde nicht gefunden.`;
        this.plugin = undefined;
        this.currentConfig = undefined;
        this.repoAgentSettings = undefined;
        this.configEditor = '';
        return;
      }
      this.plugin = plugin;
      this.updateConfigState(plugin.config);
      this.statusSubscription = this.ws.watchStatus(plugin.name).subscribe((state) => {
        this.plugin = state;
        this.updateConfigState(state.config);
      });
      this.telemetrySubscription = this.ws.watchTelemetry(plugin.name).subscribe((snapshot) => {
        this.telemetry = snapshot;
      });
      this.logSubscription = this.ws.watchLogs(plugin.name).subscribe({
        next: (event) => {
          this.logs = [...this.logs.slice(-199), event];
        },
        error: (err) => {
          this.error = err.message;
        },
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

  onRepoAgentSettingsChange(settings: RepoAgentSettings): void {
    if (!this.plugin || this.plugin.name !== 'repoagent' || !this.currentConfig) {
      return;
    }
    this.repoAgentSettings = settings;
    const updated: PluginConfig = {
      ...this.currentConfig,
      settings: { ...settings },
    };
    this.currentConfig = updated;
    this.configEditor = JSON.stringify(updated, null, 2);
  }

  private updateConfigState(config: PluginConfig): void {
    const cloned = JSON.parse(JSON.stringify(config)) as PluginConfig;
    this.currentConfig = cloned;
    this.configEditor = JSON.stringify(cloned, null, 2);
    if (this.plugin?.name === 'repoagent') {
      this.repoAgentSettings = this.extractRepoAgentSettings(cloned);
    } else {
      this.repoAgentSettings = undefined;
    }
  }

  private extractRepoAgentSettings(config: PluginConfig | undefined): RepoAgentSettings | undefined {
    if (!config?.settings) {
      return undefined;
    }
    const settings = config.settings as Partial<RepoAgentSettings>;
    return {
      defaultRoot: settings.defaultRoot ?? '',
      workspaceRoots: Array.isArray(settings.workspaceRoots)
        ? [...settings.workspaceRoots]
        : typeof settings.workspaceRoots === 'string'
          ? [settings.workspaceRoots]
          : [],
      maxFiles: settings.maxFiles ?? 500,
      ignoreGlobs: Array.isArray(settings.ignoreGlobs) ? [...settings.ignoreGlobs] : [],
      commandAllowlist: (settings.commandAllowlist ?? []).map((raw) => {
        const spec = raw as Partial<RepoAgentSettings['commandAllowlist'][number]> & {
          executable?: unknown;
        };
        let executable: string[] = [];
        if (Array.isArray(spec.executable)) {
          executable = spec.executable.map((value) => String(value));
        } else if (typeof spec.executable === 'string') {
          const execStr = spec.executable as string;
          executable = execStr.split(/\s+/).filter((token: string) => token.length > 0);
        }
        return {
          name: spec.name ?? 'command',
          executable,
          timeoutSeconds: spec.timeoutSeconds ?? 300,
          allowArgs: spec.allowArgs ?? false,
          workingDir: spec.workingDir ?? undefined,
        };
      }),
      environment: { ...(settings.environment ?? {}) },
      enableGit: settings.enableGit ?? true,
      telemetry: {
        sampleIntervalSeconds: settings.telemetry?.sampleIntervalSeconds ?? 15,
      },
    };
  }
}
