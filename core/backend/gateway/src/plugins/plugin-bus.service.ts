import { Injectable, Logger, OnModuleDestroy, OnModuleInit } from '@nestjs/common';
import { createServer, Server as HttpServer } from 'http';
import WebSocket, { WebSocketServer } from 'ws';
import { EventEmitter } from 'events';
import { randomUUID } from 'crypto';
import {
  PluginBusMessage,
  PluginCapability,
  PluginLogMessage,
  PluginRegistrationMessage,
  PluginRequestMessage,
  PluginResponseMessage,
  PluginRuntimeState,
  PluginTelemetryMessage,
  PluginTelemetrySnapshot,
} from './plugin.types';

interface PendingRequest {
  resolve: (data: unknown) => void;
  reject: (error: Error) => void;
  timeout: NodeJS.Timeout;
}

type PluginStatus = 'error' | 'stopped' | 'starting' | 'running' | 'degraded';

@Injectable()
export class PluginBusService
  extends EventEmitter
  implements OnModuleInit, OnModuleDestroy
{
  private readonly logger = new Logger(PluginBusService.name);
  private wss?: WebSocketServer;
  private httpServer?: HttpServer;
  private readonly connections = new Map<string, WebSocket>();
  private readonly states = new Map<string, PluginRuntimeState>();
  private readonly pending = new Map<string, PendingRequest>();
  private readonly logs = new Map<string, PluginLogMessage[]>();
  private readonly telemetry = new Map<string, PluginTelemetrySnapshot>();

  get port(): number {
    if (!this.httpServer) {
      throw new Error('Plugin bus has not been initialized');
    }
    const address = this.httpServer.address();
    if (address && typeof address === 'object') {
      return address.port;
    }
    throw new Error('Unable to determine plugin bus port');
  }

  getPlugins(): PluginRuntimeState[] {
    return Array.from(this.states.values()).map((state) => ({ ...state }));
  }

  getPlugin(name: string): PluginRuntimeState | undefined {
    return this.states.get(name);
  }

  getLogs(plugin: string, limit: number): PluginLogMessage[] {
    const buffer = this.logs.get(plugin) ?? [];
    return buffer.slice(Math.max(0, buffer.length - limit));
  }

  getTelemetrySnapshot(plugin: string): PluginTelemetrySnapshot | undefined {
    return this.telemetry.get(plugin);
  }

  getAllTelemetry(): PluginTelemetrySnapshot[] {
    return Array.from(this.telemetry.values());
  }

  async request<T = unknown>(
    plugin: string,
    capability: PluginCapability,
    payload: unknown,
    token?: string,
  ): Promise<T> {
    const connection = this.connections.get(plugin);
    if (!connection || connection.readyState !== WebSocket.OPEN) {
      throw new Error(`Plugin ${plugin} is not connected`);
    }
    const requestId = randomUUID();
    const message: PluginRequestMessage = {
      type: 'request',
      requestId,
      capability,
      payload,
      token,
    };
    const promise = new Promise<T>((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(requestId);
        reject(new Error(`Request to ${plugin} timed out`));
      }, 30_000);
      this.pending.set(requestId, {
        resolve: (data) => {
          clearTimeout(timeout);
          resolve(data as T);
        },
        reject: (error) => {
          clearTimeout(timeout);
          reject(error);
        },
        timeout,
      });
    });
    connection.send(JSON.stringify(message));
    return promise;
  }

  onModuleInit(): void {
    const envPort = process.env.BKG_PLUGIN_BUS_PORT
      ? Number.parseInt(process.env.BKG_PLUGIN_BUS_PORT, 10)
      : undefined;
    const httpServer = createServer();
    const wss = new WebSocketServer({ server: httpServer });
    wss.on('connection', (socket) => this.handleConnection(socket));
    const chosenPort = envPort ?? 0;
    httpServer.listen(chosenPort, '0.0.0.0', () => {
      const address = httpServer.address();
      if (address && typeof address === 'object') {
        this.logger.log(`Plugin bus listening on port ${address.port}`);
      }
    });
    this.httpServer = httpServer;
    this.wss = wss;
  }

  onModuleDestroy(): void {
    for (const [, connection] of this.connections) {
      try {
        connection.close();
      } catch (err) {
        this.logger.error('Error closing plugin connection', err as Error);
      }
    }
    this.wss?.close();
    this.httpServer?.close();
    for (const [, pending] of this.pending) {
      clearTimeout(pending.timeout);
      pending.reject(new Error('Server shutting down'));
    }
    this.pending.clear();
  }

  private handleConnection(socket: WebSocket) {
    socket.once('message', (data) => {
      try {
        const message = JSON.parse(data.toString()) as PluginBusMessage;
        if (message.type !== 'register') {
          throw new Error('First message must be register');
        }
        this.registerPlugin(socket, message);
        socket.on('message', (payload) => this.handleMessage(message.plugin, payload));
        socket.on('close', () => this.unregisterPlugin(message.plugin));
        socket.on('error', (error) => {
          this.logger.error(`Plugin ${message.plugin} connection error`, error as Error);
          this.unregisterPlugin(message.plugin);
        });
      } catch (error) {
        this.logger.error('Failed to process plugin registration', error as Error);
        socket.close();
      }
    });
  }

  private registerPlugin(socket: WebSocket, message: PluginRegistrationMessage) {
    const state: PluginRuntimeState = {
      name: message.plugin,
      status: 'running',
      pid: undefined,
      port: message.port,
      lastHeartbeat: new Date(),
      capabilities: message.capabilities,
      config: this.states.get(message.plugin)?.config ?? {
        name: message.plugin,
        description: message.plugin,
        entrypoint: '',
        capabilities: message.capabilities,
      },
      configSchema: message.configSchema ?? this.states.get(message.plugin)?.configSchema,
    };
    this.connections.set(message.plugin, socket);
    this.states.set(message.plugin, state);
    this.emit('pluginRegistered', state);
    this.logger.log(`Plugin ${message.plugin} registered on port ${message.port}`);
  }

  private unregisterPlugin(plugin: string) {
    this.connections.delete(plugin);
    const state = this.states.get(plugin);
    if (state) {
      this.states.set(plugin, { ...state, status: 'stopped' });
    }
    this.emit('pluginUnregistered', plugin);
    this.logger.warn(`Plugin ${plugin} disconnected`);
  }

  updateState(plugin: string, updater: (state: PluginRuntimeState) => PluginRuntimeState) {
    const current = this.states.get(plugin);
    if (!current) {
      return;
    }
    const updated = updater(current);
    this.states.set(plugin, updated);
    this.emit('pluginUpdated', updated);
  }

  setConfig(plugin: string, config: PluginRuntimeState['config']): void {
    const state = this.states.get(plugin);
    if (state) {
      this.states.set(plugin, { ...state, config });
    }
  }

  ensureState(plugin: string, config: PluginRuntimeState['config']): void {
    if (!this.states.has(plugin)) {
      this.states.set(plugin, {
        name: plugin,
        status: 'stopped',
        capabilities: config.capabilities,
        config,
        configSchema: undefined,
        pid: undefined,
        port: undefined,
        lastHeartbeat: undefined,
      });
    }
  }

  private handleMessage(plugin: string, payload: WebSocket.RawData) {
    try {
      const message = JSON.parse(payload.toString()) as PluginBusMessage;
      if (message.type === 'log') {
        const logMessage = message as PluginLogMessage;
        this.storeLog(plugin, logMessage);
        this.emit('log', logMessage);
        return;
      }
      if (message.type === 'health') {
        this.updateState(plugin, (state) => ({
          ...state,
          lastHeartbeat: new Date(),
          status: (message.status === 'up' ? 'running' : 'degraded') as PluginRuntimeState['status'],
        }));
        this.emit('health', message);
        return;
      }
      if (message.type === 'telemetry') {
        const telemetry = message as PluginTelemetryMessage;
        const snapshot: PluginTelemetrySnapshot = {
          plugin,
          cpu: telemetry.cpu,
          memBytes: telemetry.mem_bytes,
          modelsLoaded: telemetry.models_loaded,
          datasets: telemetry.datasets,
          entries: telemetry.entries,
          timestamp: new Date(),
        };
        this.telemetry.set(plugin, snapshot);
        this.emit('telemetry', snapshot);
        return;
      }
      if (message.type === 'response') {
        const response = message as PluginResponseMessage;
        const pending = this.pending.get(response.requestId);
        if (!pending) {
          this.logger.warn(`No pending request for ${response.requestId}`);
          return;
        }
        this.pending.delete(response.requestId);
        if (response.success) {
          pending.resolve(response.data ?? null);
        } else {
          pending.reject(new Error(response.error ?? 'Unknown error'));
        }
      }
    } catch (error) {
      this.logger.error(`Failed to process message from ${plugin}`, error as Error);
    }
  }

  private storeLog(plugin: string, log: PluginLogMessage) {
    const buffer = this.logs.get(plugin);
    if (!buffer) {
      this.logs.set(plugin, [log]);
      return;
    }
    buffer.push(log);
    if (buffer.length > 500) {
      buffer.splice(0, buffer.length - 500);
    }
  }
}
