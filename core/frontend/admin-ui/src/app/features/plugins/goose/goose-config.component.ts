import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { firstValueFrom } from 'rxjs';
import {
  GooseRunHistory,
  GooseRunRequest,
  GooseScheduleEntry,
  GooseSettings,
  GooseStatus,
  PluginState,
} from '../../../models/api.models';
import { ApiService } from '../../../services/api.service';

interface KeyValueRow {
  key: string;
  value: string;
}

@Component({
  selector: 'app-goose-config',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './goose-config.component.html',
  styleUrls: ['./goose-config.component.css'],
})
export class GooseConfigComponent implements OnInit, OnChanges {
  @Input({ required: true }) settings!: GooseSettings;
  @Input() plugin?: PluginState;
  @Output() settingsChange = new EventEmitter<GooseSettings>();

  editing!: GooseSettings;
  status?: GooseStatus;
  history: GooseRunHistory['runs'] = [];
  globalHeaders: KeyValueRow[] = [];
  scheduleHeadersText: string[] = [];
  scheduleQueriesText: string[] = [];
  loading = false;
  actionMessage = '';
  actionError = '';

  constructor(private readonly api: ApiService) {}

  ngOnInit(): void {
    this.syncFromInput();
    void this.reload();
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['settings'] && changes['settings'].currentValue) {
      this.syncFromInput();
    }
  }

  get canRun(): boolean {
    return this.plugin?.status === 'running';
  }

  get hasActiveRun(): boolean {
    return this.status?.status === 'running' || this.status?.status === 'starting';
  }

  addGlobalHeader(): void {
    this.globalHeaders.push({ key: '', value: '' });
    this.updateGlobalHeaders();
  }

  removeGlobalHeader(index: number): void {
    this.globalHeaders.splice(index, 1);
    this.updateGlobalHeaders();
  }

  addScheduleEntry(): void {
    this.editing.schedule.push({
      name: 'Request',
      method: 'GET',
      path: '/',
      weight: 1,
      thinkTimeMs: 0,
      headers: {},
      query: {},
    });
    this.scheduleHeadersText.push('');
    this.scheduleQueriesText.push('');
    this.emitSettings();
  }

  removeScheduleEntry(index: number): void {
    this.editing.schedule.splice(index, 1);
    this.scheduleHeadersText.splice(index, 1);
    this.scheduleQueriesText.splice(index, 1);
    this.emitSettings();
  }

  updateScheduleHeaders(index: number): void {
    this.editing.schedule[index].headers = this.parseLines(this.scheduleHeadersText[index]);
    this.emitSettings();
  }

  updateScheduleQueries(index: number): void {
    this.editing.schedule[index].query = this.parseLines(this.scheduleQueriesText[index]);
    this.emitSettings();
  }

  updateGlobalHeaders(): void {
    this.editing.globalHeaders = this.toRecord(this.globalHeaders);
    this.emitSettings();
  }

  onThrottleChange(value: string | number | null): void {
    if (value === null || value === '') {
      this.editing.throttleRps = null;
    } else {
      const numeric = typeof value === 'number' ? value : Number(value);
      if (!Number.isFinite(numeric) || numeric <= 0) {
        this.editing.throttleRps = null;
      } else {
        this.editing.throttleRps = Math.floor(numeric);
      }
    }
    this.emitSettings();
  }

  async run(): Promise<void> {
    if (!this.canRun) {
      this.actionError = 'Plug-in muss laufen, bevor ein Test gestartet werden kann.';
      return;
    }
    this.loading = true;
    this.actionError = '';
    this.actionMessage = '';
    try {
      const payload: GooseRunRequest = this.buildRunRequest();
      const response = await firstValueFrom(this.api.gooseRun(payload));
      this.actionMessage = response.message;
      await this.reloadStatus();
    } catch (error) {
      this.actionError = (error as Error).message;
    } finally {
      this.loading = false;
    }
  }

  async stop(): Promise<void> {
    if (!this.hasActiveRun) {
      this.actionError = 'Es läuft aktuell kein Testlauf.';
      return;
    }
    this.loading = true;
    this.actionError = '';
    this.actionMessage = '';
    try {
      const response = await firstValueFrom(this.api.gooseStop());
      this.actionMessage = response.message;
      await this.reloadStatus();
    } catch (error) {
      this.actionError = (error as Error).message;
    } finally {
      this.loading = false;
    }
  }

  async reload(): Promise<void> {
    await Promise.all([this.reloadStatus(), this.reloadHistory()]);
  }

  trackHeader(_: number, item: KeyValueRow): string {
    return `${item.key}-${item.value}`;
  }

  trackSchedule(_: number, item: GooseScheduleEntry): string {
    return `${item.name}-${item.path}-${item.method}`;
  }

  private async reloadStatus(): Promise<void> {
    try {
      this.status = await firstValueFrom(this.api.gooseStatus());
    } catch (error) {
      this.actionError = (error as Error).message;
    }
  }

  private async reloadHistory(): Promise<void> {
    try {
      const history = await firstValueFrom(this.api.gooseHistory());
      this.history = history.runs;
    } catch (error) {
      this.actionError = (error as Error).message;
    }
  }

  private buildRunRequest(): GooseRunRequest {
    return {
      target: this.editing.defaultTarget,
      users: this.editing.users,
      hatchRate: this.editing.hatchRate,
      runTimeSeconds: this.editing.runTimeSeconds,
      timeoutSeconds: this.editing.timeoutSeconds,
      startupTimeSeconds: this.editing.startupTimeSeconds,
      gracefulStopSeconds: this.editing.gracefulStopSeconds,
      throttleRps: this.editing.throttleRps ?? undefined,
      globalHeaders: this.editing.globalHeaders,
      verifyTls: this.editing.verifyTls,
      stickyCookies: this.editing.stickyCookies,
      followRedirects: this.editing.followRedirects,
      schedule: this.editing.schedule.map((entry) => ({
        name: entry.name,
        method: entry.method,
        path: entry.path,
        weight: entry.weight,
        body: entry.body,
        headers: entry.headers,
        query: entry.query,
        thinkTimeMs: entry.thinkTimeMs,
      })),
    };
  }

  private syncFromInput(): void {
    this.editing = JSON.parse(JSON.stringify(this.settings)) as GooseSettings;
    if (this.editing.throttleRps === undefined || this.editing.throttleRps === 0) {
      this.editing.throttleRps = null;
    }
    this.globalHeaders = this.toRows(this.editing.globalHeaders);
    this.scheduleHeadersText = this.editing.schedule.map((entry) => this.mapToLines(entry.headers ?? {}));
    this.scheduleQueriesText = this.editing.schedule.map((entry) => this.mapToLines(entry.query ?? {}));
  }

  emitSettings(): void {
    this.settingsChange.emit(JSON.parse(JSON.stringify(this.editing)) as GooseSettings);
  }

  private toRows(record: Record<string, string> | undefined): KeyValueRow[] {
    if (!record) {
      return [];
    }
    return Object.entries(record).map(([key, value]) => ({ key, value }));
  }

  private toRecord(rows: KeyValueRow[]): Record<string, string> {
    const result: Record<string, string> = {};
    for (const row of rows) {
      if (row.key) {
        result[row.key] = row.value;
      }
    }
    return result;
  }

  private mapToLines(record: Record<string, string>): string {
    return Object.entries(record)
      .map(([key, value]) => `${key}=${value}`)
      .join('\n');
  }

  private parseLines(value: string): Record<string, string> {
    const result: Record<string, string> = {};
    value
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line.length > 0)
      .forEach((line) => {
        const [key, ...rest] = line.split('=');
        if (key) {
          result[key.trim()] = rest.join('=').trim();
        }
      });
    return result;
  }
}
