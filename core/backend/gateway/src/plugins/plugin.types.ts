export type PluginCapability =
  | 'llm.chat'
  | 'llm.embed'
  | 'repo.analyze'
  | 'repo.patch'
  | 'repo.tree'
  | 'repo.file.read'
  | 'repo.file.write'
  | 'repo.search'
  | 'repo.command'
  | 'repo.commit'
  | 'goose.run'
  | 'goose.status'
  | 'goose.stop'
  | 'goose.history'
  | 'auth.login'
  | 'auth.createKey'
  | 'auth.revokeKey'
  | 'auth.listKeys'
  | 'auth.validate'
  | 'brainml.index'
  | 'brainml.query'
  | 'brainml.train'
  | 'brainml.stats'
  | 'brainml.admin'
  | 'candle.model.load'
  | 'candle.model.run'
  | 'candle.stats'
  | 'faces.encode'
  | 'faces.search'
  | 'dataset.manage';

export interface PluginConfig {
  name: string;
  description: string;
  entrypoint: string;
  args?: string[];
  env?: Record<string, string>;
  autostart?: boolean;
  capabilities: PluginCapability[];
  settings?: Record<string, unknown>;
  healthcheck?: {
    path: string;
    intervalSeconds?: number;
  };
}

export interface PluginRegistrationMessage {
  type: 'register';
  plugin: string;
  port: number | 'internal';
  capabilities: PluginCapability[];
  configSchema?: unknown;
  meta?: Record<string, unknown>;
}

export interface PluginLogMessage {
  type: 'log';
  plugin: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  timestamp?: string;
}

export interface PluginTelemetryMessage {
  type: 'telemetry';
  plugin: string;
  cpu: number;
  mem_bytes: number;
  models_loaded?: number;
  datasets?: number;
  entries?: number;
}

export interface PluginTelemetrySnapshot {
  plugin: string;
  cpu: number;
  memBytes: number;
  modelsLoaded?: number;
  datasets?: number;
  entries?: number;
  timestamp: Date;
}

export interface PluginHealthMessage {
  type: 'health';
  plugin: string;
  status: 'up' | 'down' | 'degraded';
  detail?: string;
}

export interface PluginRequestMessage {
  type: 'request';
  requestId: string;
  capability: PluginCapability;
  payload: unknown;
  token?: string;
}

export interface PluginResponseMessage {
  type: 'response';
  requestId: string;
  success: boolean;
  data?: unknown;
  error?: string;
}

export type PluginBusMessage =
  | PluginRegistrationMessage
  | PluginLogMessage
  | PluginHealthMessage
  | PluginTelemetryMessage
  | PluginRequestMessage
  | PluginResponseMessage;

export interface PluginRuntimeState {
  name: string;
  status: 'stopped' | 'starting' | 'running' | 'error';
  pid?: number;
  port?: number | 'internal';
  lastHeartbeat?: Date;
  capabilities: PluginCapability[];
  config: PluginConfig;
  configSchema?: unknown;
  error?: string;
}
