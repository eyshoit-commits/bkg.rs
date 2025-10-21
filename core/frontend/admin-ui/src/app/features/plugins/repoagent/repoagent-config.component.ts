import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, Output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RepoAgentCommandSpec, RepoAgentSettings } from '../../../models/api.models';

@Component({
  selector: 'app-repoagent-config',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './repoagent-config.component.html',
  styleUrls: ['./repoagent-config.component.css']
})
export class RepoagentConfigComponent {
  @Input({ required: true }) settings!: RepoAgentSettings;
  @Output() settingsChange = new EventEmitter<RepoAgentSettings>();

  addWorkspaceRoot(): void {
    this.emitMutation((draft) => {
      draft.workspaceRoots.push('');
    });
  }

  removeWorkspaceRoot(index: number): void {
    this.emitMutation((draft) => {
      draft.workspaceRoots.splice(index, 1);
    });
  }

  onWorkspaceRootChange(index: number, value: string): void {
    this.emitMutation((draft) => {
      draft.workspaceRoots[index] = value.trim();
    });
  }

  addIgnoreGlob(): void {
    this.emitMutation((draft) => {
      draft.ignoreGlobs.push('');
    });
  }

  removeIgnoreGlob(index: number): void {
    this.emitMutation((draft) => {
      draft.ignoreGlobs.splice(index, 1);
    });
  }

  onIgnoreGlobChange(index: number, value: string): void {
    this.emitMutation((draft) => {
      draft.ignoreGlobs[index] = value.trim();
    });
  }

  onDefaultRootChange(value: string): void {
    this.emitMutation((draft) => {
      draft.defaultRoot = value.trim();
    });
  }

  onMaxFilesChange(value: number | string): void {
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed <= 0) {
      return;
    }
    this.emitMutation((draft) => {
      draft.maxFiles = Math.floor(parsed);
    });
  }

  onTelemetryIntervalChange(value: number | string): void {
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed < 5) {
      return;
    }
    this.emitMutation((draft) => {
      draft.telemetry.sampleIntervalSeconds = Math.floor(parsed);
    });
  }

  toggleGit(enabled: boolean): void {
    this.emitMutation((draft) => {
      draft.enableGit = enabled;
    });
  }

  addCommand(): void {
    this.emitMutation((draft) => {
      const nextName = this.nextCommandName(draft.commandAllowlist);
      draft.commandAllowlist.push({
        name: nextName,
        executable: ['echo', 'hello-world'],
        timeoutSeconds: 300,
        allowArgs: false,
      });
    });
  }

  removeCommand(index: number): void {
    this.emitMutation((draft) => {
      if (!draft.commandAllowlist[index]) {
        return;
      }
      draft.commandAllowlist.splice(index, 1);
    });
  }

  onCommandNameChange(index: number, value: string): void {
    this.emitMutation((draft) => {
      if (!draft.commandAllowlist[index]) {
        return;
      }
      draft.commandAllowlist[index].name = value.trim();
    });
  }

  onCommandExecutableChange(index: number, value: string): void {
    const tokens = value
      .split(/\s+/)
      .map((token) => token.trim())
      .filter((token) => token.length > 0);
    if (tokens.length === 0) {
      return;
    }
    this.emitMutation((draft) => {
      if (!draft.commandAllowlist[index]) {
        return;
      }
      draft.commandAllowlist[index].executable = tokens;
    });
  }

  onCommandTimeoutChange(index: number, value: number | string): void {
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed <= 0) {
      return;
    }
    this.emitMutation((draft) => {
      if (!draft.commandAllowlist[index]) {
        return;
      }
      draft.commandAllowlist[index].timeoutSeconds = Math.floor(parsed);
    });
  }

  onCommandAllowArgsChange(index: number, allow: boolean): void {
    this.emitMutation((draft) => {
      if (!draft.commandAllowlist[index]) {
        return;
      }
      draft.commandAllowlist[index].allowArgs = allow;
    });
  }

  onCommandWorkingDirChange(index: number, value: string): void {
    this.emitMutation((draft) => {
      if (!draft.commandAllowlist[index]) {
        return;
      }
      draft.commandAllowlist[index].workingDir = value.trim() || undefined;
    });
  }

  environmentEntries(): { key: string; value: string }[] {
    return Object.entries(this.settings.environment || {}).map(([key, value]) => ({ key, value }));
  }

  addEnvironmentEntry(): void {
    this.emitMutation((draft) => {
      const base = 'NEW_VAR';
      let key = base;
      let counter = 1;
      while (Object.prototype.hasOwnProperty.call(draft.environment, key)) {
        key = `${base}_${counter++}`;
      }
      draft.environment[key] = '';
    });
  }

  removeEnvironmentEntry(index: number): void {
    this.emitMutation((draft) => {
      const entries = this.environmentEntriesFromDraft(draft);
      entries.splice(index, 1);
      draft.environment = this.entriesToEnvironment(entries);
    });
  }

  onEnvironmentKeyChange(index: number, value: string): void {
    this.emitMutation((draft) => {
      const entries = this.environmentEntriesFromDraft(draft);
      const [, currentValue] = entries[index];
      entries[index] = [value.trim(), currentValue];
      draft.environment = this.entriesToEnvironment(entries.filter(([key]) => key.length > 0));
    });
  }

  onEnvironmentValueChange(index: number, value: string): void {
    this.emitMutation((draft) => {
      const entries = this.environmentEntriesFromDraft(draft);
      const [key] = entries[index];
      entries[index] = [key, value];
      draft.environment = this.entriesToEnvironment(entries);
    });
  }

  trackByIndex(index: number): number {
    return index;
  }

  trackCommand(_: number, item: RepoAgentCommandSpec): string {
    return item.name;
  }

  private emitMutation(mutator: (draft: RepoAgentSettings) => void): void {
    const draft = this.cloneSettings(this.settings);
    mutator(draft);
    this.settingsChange.emit(draft);
  }

  private cloneSettings(settings: RepoAgentSettings): RepoAgentSettings {
    return {
      ...settings,
      workspaceRoots: [...settings.workspaceRoots],
      ignoreGlobs: [...settings.ignoreGlobs],
      commandAllowlist: settings.commandAllowlist.map((spec) => ({
        ...spec,
        executable: [...spec.executable],
      })),
      environment: { ...settings.environment },
      telemetry: { ...settings.telemetry },
    };
  }

  private nextCommandName(existing: RepoAgentCommandSpec[]): string {
    const base = 'command';
    let index = existing.length + 1;
    let candidate = `${base}-${index}`;
    const names = new Set(existing.map((spec) => spec.name));
    while (names.has(candidate)) {
      index += 1;
      candidate = `${base}-${index}`;
    }
    return candidate;
  }

  private environmentEntriesFromDraft(draft: RepoAgentSettings): [string, string][] {
    return Object.entries(draft.environment || {});
  }

  private entriesToEnvironment(entries: [string, string][]): Record<string, string> {
    const env: Record<string, string> = {};
    for (const [key, value] of entries) {
      if (key.length === 0) {
        continue;
      }
      env[key] = value;
    }
    return env;
  }
}
