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
}

export interface PluginLogEvent {
  plugin: string;
  level: string;
  message: string;
  timestamp: string;
}

export interface ApiKeyRecord {
  id: string;
  user: string;
  scopes: string[];
  created_at: number;
  preview: string;
}
