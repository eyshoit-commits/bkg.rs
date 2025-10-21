import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { firstValueFrom } from 'rxjs';
import { ApiService } from '../../../../services/api.service';
import { PluginState } from '../../../../models/api.models';
import { PLUGIN_FEATURES, TOTAL_FEATURE_COUNT } from '../plugin-features';

@Component({
  selector: 'app-plugin-list',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './plugin-list.component.html',
  styleUrls: ['./plugin-list.component.css']
})
export class PluginListComponent implements OnInit {
  plugins: PluginState[] = [];
  loading = false;
  error: string | null = null;

  readonly featureCatalog = PLUGIN_FEATURES;
  readonly totalFeatureCount = TOTAL_FEATURE_COUNT;

  constructor(private readonly api: ApiService) {}

  ngOnInit(): void {
    void this.refresh();
  }

  async refresh(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.plugins = await firstValueFrom(this.api.listPlugins());
    } catch (error) {
      this.error = (error as Error).message;
    } finally {
      this.loading = false;
    }
  }

  async start(plugin: PluginState, event: Event): Promise<void> {
    event.stopPropagation();
    event.preventDefault();
    this.error = null;
    try {
      await firstValueFrom(this.api.startPlugin(plugin.name));
      await this.refresh();
    } catch (error) {
      this.error = (error as Error).message;
    }
  }

  async stop(plugin: PluginState, event: Event): Promise<void> {
    event.stopPropagation();
    event.preventDefault();
    this.error = null;
    try {
      await firstValueFrom(this.api.stopPlugin(plugin.name));
      await this.refresh();
    } catch (error) {
      this.error = (error as Error).message;
    }
  }

  featureCount(plugin: PluginState): number {
    return this.featureCatalog[plugin.name]?.length ?? plugin.capabilities.length;
  }
}
