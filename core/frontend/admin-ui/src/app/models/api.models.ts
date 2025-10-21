export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface ChatCompletionResponse {
  id: string;
  model: string;
  choices: { index: number; message: ChatMessage; finish_reason: string }[];
}

export interface EmbeddingResponse {
  model: string;
  data: { index: number; embedding: number[] }[];
}

export interface PluginState {
  name: string;
  status: 'stopped' | 'starting' | 'running' | 'error';
  port?: number | 'internal';
  capabilities: string[];
  config: PluginConfig;
  configSchema?: unknown;
  error?: string;
}

export interface PluginConfig {
  name: string;
  description: string;
  entrypoint: string;
  args?: string[];
  env?: Record<string, string>;
  autostart?: boolean;
  capabilities: string[];
  settings?: Record<string, unknown>;
}

export interface RepoAgentCommandSpec {
  name: string;
  executable: string[];
  timeoutSeconds: number;
  allowArgs: boolean;
  workingDir?: string;
}

export interface RepoAgentSettings {
  defaultRoot: string;
  workspaceRoots: string[];
  maxFiles: number;
  ignoreGlobs: string[];
  commandAllowlist: RepoAgentCommandSpec[];
  environment: Record<string, string>;
  enableGit: boolean;
  telemetry: {
    sampleIntervalSeconds: number;
  };
}

export type GooseRunState = 'idle' | 'starting' | 'running' | 'stopping' | 'completed' | 'failed';

export interface GooseScheduleEntry {
  name: string;
  method: string;
  path: string;
  weight: number;
  body?: string;
  headers?: Record<string, string>;
  query?: Record<string, string>;
  thinkTimeMs?: number;
}

export interface GooseSettings {
  defaultTarget: string;
  users: number;
  hatchRate: number;
  runTimeSeconds: number;
  timeoutSeconds: number;
  globalHeaders: Record<string, string>;
  verifyTls: boolean;
  maxHistory: number;
  schedule: GooseScheduleEntry[];
}

export interface GooseRunRequest {
  target?: string;
  users?: number;
  hatchRate?: number;
  runTimeSeconds?: number;
  timeoutSeconds?: number;
  globalHeaders?: Record<string, string>;
  verifyTls?: boolean;
  schedule?: GooseScheduleEntry[];
}

export interface GooseMetricsSnapshot {
  totalRequests: number;
  successRequests: number;
  failedRequests: number;
  requestsPerSecond: number;
  averageLatencyMs: number;
  p95LatencyMs: number;
  p99LatencyMs: number;
  bytesSent: number;
  bytesReceived: number;
}

export interface GooseEffectiveSettings {
  target: string;
  users: number;
  hatchRate: number;
  runTimeSeconds: number;
  timeoutSeconds: number;
  verifyTls: boolean;
}

export interface GooseStatus {
  status: GooseRunState;
  runId?: string;
  startedAt?: string;
  finishedAt?: string;
  durationSeconds: number;
  settings: GooseEffectiveSettings;
  metrics: GooseMetricsSnapshot;
}

export interface GooseRunResponse {
  status: GooseRunState;
  runId?: string;
  message: string;
  metrics: GooseMetricsSnapshot;
}

export interface GooseRunHistoryEntry {
  runId: string;
  status: GooseRunState;
  startedAt: string;
  finishedAt: string;
  durationSeconds: number;
  settings: GooseEffectiveSettings;
  metrics: GooseMetricsSnapshot;
}

export interface GooseRunHistory {
  runs: GooseRunHistoryEntry[];
}

export interface PluginLogEvent {
  plugin: string;
  level: string;
  message: string;
  timestamp: string;
}

export interface PluginTelemetry {
  plugin: string;
  cpu: number;
  memBytes: number;
  modelsLoaded?: number;
  datasets?: number;
  entries?: number;
  timestamp: string;
}

export interface ApiKeyRecord {
  id: string;
  user: string;
  scopes: string[];
  created_at: number;
  preview: string;
}
