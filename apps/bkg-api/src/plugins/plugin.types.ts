export type PluginCapability =
  | 'llm.chat'
  | 'llm.embed'
  | 'repo.analyze'
  | 'repo.patch'
  | 'auth.login'
  | 'auth.createKey'
  | 'auth.revokeKey'
  | 'auth.listKeys'
  | 'auth.validate'
  | 'brainml.index'
  | 'brainml.query'
  | 'brainml.train'
  | 'brainml.stats'
  | 'brainml.admin';

export interface PluginConfig {
  name: string;
  description: string;
  entrypoint: string;
  args?: string[];
  env?: Record<string, string>;
  autostart?: boolean;
  capabilities: PluginCapability[];
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
  error?: string;
}
