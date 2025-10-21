import { Component, OnDestroy, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Subscription, firstValueFrom } from 'rxjs';
import { ApiService } from '../../services/api.service';
import { PluginLogEvent, PluginState } from '../../models/api.models';
import { PluginWsService } from '../../services/plugin-ws.service';

@Component({
  selector: 'app-plugins',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './plugins.component.html',
  styleUrls: ['./plugins.component.css']
})
export class PluginsComponent implements OnInit, OnDestroy {
  plugins: PluginState[] = [];
  selected?: PluginState;
  configEditor = '';
  logs: PluginLogEvent[] = [];
  loading = false;
  error: string | null = null;
  private logSubscription?: Subscription;
  private statusSubscription?: Subscription;
  private bootstrapSubscription?: Subscription;

  constructor(private readonly api: ApiService, private readonly ws: PluginWsService) {}

  ngOnInit(): void {
    void this.refresh();
    this.bootstrapSubscription = this.ws.bootstrap().subscribe(({ plugins }) => {
      this.plugins = plugins;
      if (this.selected) {
        this.selected = this.plugins.find((plugin) => plugin.name === this.selected?.name);
      }
    });
    this.statusSubscription = this.ws.watchStatus().subscribe((state) => {
      const index = this.plugins.findIndex((plugin) => plugin.name === state.name);
      if (index >= 0) {
        this.plugins = [
          ...this.plugins.slice(0, index),
          state,
          ...this.plugins.slice(index + 1),
        ];
        if (this.selected?.name === state.name) {
          this.selected = state;
          this.configEditor = JSON.stringify(state.config, null, 2);
        }
      }
    });
  }

  ngOnDestroy(): void {
    this.logSubscription?.unsubscribe();
    this.statusSubscription?.unsubscribe();
    this.bootstrapSubscription?.unsubscribe();
  }

  async refresh(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.plugins = await firstValueFrom(this.api.listPlugins());
      if (this.selected) {
        this.selected = this.plugins.find((plugin) => plugin.name === this.selected?.name);
      }
    } catch (error) {
      this.error = (error as Error).message;
    } finally {
      this.loading = false;
    }
  }

  async start(plugin: PluginState): Promise<void> {
    this.loading = true;
    try {
      await firstValueFrom(this.api.startPlugin(plugin.name));
      await this.refresh();
    } finally {
      this.loading = false;
    }
  }

  async stop(plugin: PluginState): Promise<void> {
    this.loading = true;
    try {
      await firstValueFrom(this.api.stopPlugin(plugin.name));
      await this.refresh();
    } finally {
      this.loading = false;
    }
  }

  select(plugin: PluginState): void {
    this.selected = plugin;
    this.configEditor = JSON.stringify(plugin.config, null, 2);
    this.logs = [];
    this.logSubscription?.unsubscribe();
    this.logSubscription = this.ws.watchLogs(plugin.name).subscribe({
      next: (event) => {
        this.logs = [...this.logs.slice(-199), event];
      },
      error: (err) => {
        this.error = err.message;
      }
    });
  }

  async saveConfig(): Promise<void> {
    if (!this.selected) {
      return;
    }
    try {
      const config = JSON.parse(this.configEditor);
      await firstValueFrom(this.api.updatePluginConfig(config));
      await this.refresh();
    } catch (error) {
      this.error = (error as Error).message;
    }
  }
}
