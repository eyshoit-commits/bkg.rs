import { Injectable, Logger, OnModuleDestroy, OnModuleInit } from '@nestjs/common';
import { OnGatewayConnection, OnGatewayDisconnect, WebSocketGateway } from '@nestjs/websockets';
import WebSocket from 'ws';
import { PluginService } from './plugin.service';
import { PluginBusService } from './plugin-bus.service';
import { PluginLogMessage, PluginRuntimeState, PluginTelemetrySnapshot } from './plugin.types';

interface Subscription {
  topic: 'status' | 'logs' | 'telemetry';
  pluginId?: string;
}

interface ClientContext {
  socket: WebSocket;
  subscriptions: Subscription[];
}

interface ClientMessage {
  action: 'SUB' | 'UNSUB';
  topic: Subscription['topic'];
  pluginId?: string;
}

interface StatusEvent {
  type: 'status';
  payload: PluginRuntimeState;
}

interface LogEvent {
  type: 'logs';
  payload: PluginLogMessage;
}

interface TelemetryEvent {
  type: 'telemetry';
  payload: PluginTelemetrySnapshot;
}

type OutboundEvent = StatusEvent | LogEvent | TelemetryEvent;

@Injectable()
@WebSocketGateway({
  path: '/ws/plugins',
  cors: { origin: '*' },
})
export class PluginsGateway
  implements OnGatewayConnection<WebSocket>, OnGatewayDisconnect<WebSocket>, OnModuleInit, OnModuleDestroy
{
  private readonly logger = new Logger(PluginsGateway.name);

  private readonly clients = new Map<WebSocket, ClientContext>();
  private readonly listeners: Array<() => void> = [];

  constructor(
    private readonly plugins: PluginService,
    private readonly bus: PluginBusService,
  ) {}

  onModuleInit() {
    const handleStatus = (state: PluginRuntimeState) => this.broadcast({ type: 'status', payload: state });
    const handleLog = (log: PluginLogMessage) => this.broadcast({ type: 'logs', payload: log });
    const handleTelemetry = (snapshot: PluginTelemetrySnapshot) =>
      this.broadcast({ type: 'telemetry', payload: snapshot });

    const handleUnregistered = (name: string) => {
      const state = this.plugins.getPlugin(name);
      if (state) {
        handleStatus(state);
      }
    };

    this.bus.on('pluginRegistered', handleStatus);
    this.bus.on('pluginUpdated', handleStatus);
    this.bus.on('pluginUnregistered', handleUnregistered);
    this.bus.on('log', handleLog);
    this.bus.on('telemetry', handleTelemetry);

    this.listeners.push(
      () => this.bus.off('pluginRegistered', handleStatus),
      () => this.bus.off('pluginUpdated', handleStatus),
      () => this.bus.off('pluginUnregistered', handleUnregistered),
      () => this.bus.off('log', handleLog),
      () => this.bus.off('telemetry', handleTelemetry),
    );
  }

  onModuleDestroy() {
    for (const release of this.listeners) {
      try {
        release();
      } catch (error) {
        this.logger.error('Failed to remove websocket listener', error as Error);
      }
    }
    this.listeners.length = 0;
    this.clients.clear();
  }

  handleConnection(client: WebSocket) {
    this.logger.log('Client connected to /ws/plugins');
    const context: ClientContext = { socket: client, subscriptions: [] };
    this.clients.set(client, context);
    client.on('message', (raw) => this.handleMessage(context, raw));
    client.on('close', () => this.handleDisconnect(client));
    client.on('error', (error) => this.logger.error('WebSocket client error', error as Error));

    const initial = this.plugins.listPlugins();
    const telemetry = this.plugins.getTelemetry() ?? [];
    const envelope = {
      type: 'bootstrap',
      payload: {
        plugins: initial,
        telemetry,
      },
    };
    client.send(JSON.stringify(envelope));
  }

  handleDisconnect(client: WebSocket) {
    this.clients.delete(client);
    this.logger.log('Client disconnected from /ws/plugins');
  }

  private handleMessage(context: ClientContext, raw: WebSocket.RawData) {
    try {
      const message = JSON.parse(raw.toString()) as ClientMessage;
      if (message.action === 'SUB') {
        this.subscribe(context, message);
      } else if (message.action === 'UNSUB') {
        this.unsubscribe(context, message);
      }
    } catch (error) {
      this.logger.warn(`Invalid WS payload: ${(error as Error).message}`);
    }
  }

  private subscribe(context: ClientContext, message: ClientMessage) {
    const existing = context.subscriptions.find(
      (subscription) =>
        subscription.topic === message.topic && subscription.pluginId === message.pluginId,
    );
    if (existing) {
      return;
    }
    context.subscriptions.push({ topic: message.topic, pluginId: message.pluginId });
    if (message.topic === 'status') {
      const payload = message.pluginId
        ? this.plugins.getPlugin(message.pluginId)
        : undefined;
      if (payload) {
        this.send(context.socket, { type: 'status', payload });
      } else if (!message.pluginId) {
        for (const plugin of this.plugins.listPlugins()) {
          this.send(context.socket, { type: 'status', payload: plugin });
        }
      }
    }
    if (message.topic === 'telemetry') {
      if (message.pluginId) {
        const snapshot = this.plugins.getTelemetry(message.pluginId);
        if (snapshot) {
          this.send(context.socket, { type: 'telemetry', payload: snapshot });
        }
      } else {
        const snapshots = this.plugins.getTelemetry();
        if (Array.isArray(snapshots)) {
          for (const snapshot of snapshots) {
            this.send(context.socket, { type: 'telemetry', payload: snapshot });
          }
        }
      }
    }
    if (message.topic === 'logs' && message.pluginId) {
      const history = this.plugins.getLogs(message.pluginId, 100);
      for (const event of history) {
        this.send(context.socket, { type: 'logs', payload: event });
      }
    }
  }

  private unsubscribe(context: ClientContext, message: ClientMessage) {
    context.subscriptions = context.subscriptions.filter(
      (subscription) =>
        !(subscription.topic === message.topic && subscription.pluginId === message.pluginId),
    );
  }

  private broadcast(event: OutboundEvent) {
    for (const context of this.clients.values()) {
      if (this.shouldDeliver(context, event)) {
        this.send(context.socket, event);
      }
    }
  }

  private shouldDeliver(context: ClientContext, event: OutboundEvent) {
    if (context.subscriptions.length === 0) {
      return false;
    }
    return context.subscriptions.some((subscription) => {
      if (subscription.topic !== event.type) {
        return false;
      }
      if (!subscription.pluginId) {
        return true;
      }
      if (event.type === 'status') {
        return event.payload.name === subscription.pluginId;
      }
      if (event.type === 'logs') {
        return event.payload.plugin === subscription.pluginId;
      }
      if (event.type === 'telemetry') {
        return event.payload.plugin === subscription.pluginId;
      }
      return false;
    });
  }

  private send(socket: WebSocket, event: OutboundEvent | { type: string; payload: unknown }) {
    try {
      socket.send(JSON.stringify(event));
    } catch (error) {
      this.logger.error('Failed to send websocket payload', error as Error);
    }
  }
}
