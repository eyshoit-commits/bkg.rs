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
