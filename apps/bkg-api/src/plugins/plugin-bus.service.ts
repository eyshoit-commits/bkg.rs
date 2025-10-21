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
} from './plugin.types';

interface PendingRequest {
  resolve: (data: unknown) => void;
  reject: (error: Error) => void;
  timeout: NodeJS.Timeout;
}

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
  private readonly ready: Promise<void>;
  private resolveReady?: () => void;
  private rejectReady?: (error: Error) => void;

  constructor() {
    super();
    this.ready = new Promise<void>((resolve, reject) => {
      this.resolveReady = resolve;
      this.rejectReady = reject;
    });
  }

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

  async waitUntilReady(): Promise<void> {
    if (this.httpServer) {
      const address = this.httpServer.address();
      if (address && typeof address === 'object' && address.port) {
        return;
      }
    }
    return this.ready;
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
        this.resolveReady?.();
      }
    });
    httpServer.on('error', (error) => {
      this.logger.error('Plugin bus server error', error as Error);
      this.rejectReady?.(error as Error);
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
        this.emit('log', logMessage);
        return;
      }
      if (message.type === 'health') {
        this.updateState(plugin, (state) => ({
          ...state,
          lastHeartbeat: new Date(),
          status: message.status === 'up' ? 'running' : 'degraded',
        }));
        this.emit('health', message);
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
}
