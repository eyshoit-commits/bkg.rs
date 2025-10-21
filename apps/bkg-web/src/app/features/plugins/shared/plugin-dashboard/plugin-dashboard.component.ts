import { Component, Input, OnChanges, OnDestroy, OnInit, SimpleChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { Subscription, firstValueFrom } from 'rxjs';
import { ApiService } from '../../../../services/api.service';
import { PluginLogEvent, PluginState } from '../../../../models/api.models';
import { PluginFeatureDescription } from '../plugin-feature.model';
import { PLUGIN_FEATURES } from '../plugin-features';

@Component({
  selector: 'app-plugin-dashboard',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  templateUrl: './plugin-dashboard.component.html',
  styleUrls: ['./plugin-dashboard.component.css']
})
export class PluginDashboardComponent implements OnInit, OnDestroy, OnChanges {
  plugins: PluginState[] = [];
  plugin?: PluginState;
  logs: PluginLogEvent[] = [];
  configEditor = '';
  loading = false;
  error: string | null = null;

  @Input() pluginId: string | null | undefined;
  @Input() featureCards: PluginFeatureDescription[] = [];

  private logSubscription?: Subscription;
  private routeSubscription?: Subscription;

  private explicitPluginId: string | null = null;

  private readonly defaultFeatureCatalog = PLUGIN_FEATURES;

  constructor(
    private readonly route: ActivatedRoute,
    private readonly router: Router,
    private readonly api: ApiService
  ) {}

  ngOnInit(): void {
    this.explicitPluginId = this.pluginId ?? null;
    if (this.explicitPluginId) {
      void this.load(this.explicitPluginId);
      return;
    }
    this.subscribeToRoute();
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['pluginId']) {
      this.explicitPluginId = (changes['pluginId'].currentValue as string | null | undefined) ?? null;
      if (this.explicitPluginId) {
        this.routeSubscription?.unsubscribe();
        this.routeSubscription = undefined;
        void this.load(this.explicitPluginId);
      } else if (!this.routeSubscription) {
        this.subscribeToRoute();
      }
    }
  }

  ngOnDestroy(): void {
    this.logSubscription?.unsubscribe();
    this.routeSubscription?.unsubscribe();
  }

  get features(): PluginFeatureDescription[] {
    if (!this.plugin) {
      return [];
    }
    if (this.featureCards && this.featureCards.length > 0) {
      return this.featureCards;
    }
    if (this.plugin) {
      return this.defaultFeatureCatalog[this.plugin.name] ?? [];
    }
    return [];
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

  private subscribeToRoute(): void {
    this.routeSubscription?.unsubscribe();
    this.routeSubscription = this.route.paramMap.subscribe((params) => {
      const pluginId = params.get('pluginId');
      if (!pluginId) {
        this.plugin = undefined;
        return;
      }
      void this.load(pluginId);
    });
  }
}
