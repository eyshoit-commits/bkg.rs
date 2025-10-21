import { Injectable, NgZone } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { PluginLogEvent, PluginState, PluginTelemetry } from '../models/api.models';

type Topic = 'status' | 'logs' | 'telemetry';

interface ClientMessage {
  action: 'SUB' | 'UNSUB';
  topic: Topic;
  pluginId?: string;
}

interface BootstrapPayload {
  plugins: PluginState[];
  telemetry: PluginTelemetry[];
}

@Injectable({ providedIn: 'root' })
export class PluginWsService {
  private socket?: WebSocket;
  private readonly baseUrl = (document.querySelector('meta[name="bkg-api"]') as HTMLMetaElement)?.content ?? '';
  private readonly subscriptions = new Map<
    string,
    { count: number; subject: Subject<PluginState | PluginLogEvent | PluginTelemetry> }
  >();
  private readonly pending: ClientMessage[] = [];
  private connected = false;
  private readonly bootstrapSubject = new Subject<BootstrapPayload>();

  constructor(private readonly zone: NgZone) {}

  bootstrap(): Observable<BootstrapPayload> {
    this.ensureSocket();
    return this.bootstrapSubject.asObservable();
  }

  watchStatus(pluginId?: string): Observable<PluginState> {
    return this.createStream<PluginState>('status', pluginId);
  }

  watchLogs(pluginId: string): Observable<PluginLogEvent> {
    return this.createStream<PluginLogEvent>('logs', pluginId);
  }

  watchTelemetry(pluginId?: string): Observable<PluginTelemetry> {
    return this.createStream<PluginTelemetry>('telemetry', pluginId);
  }

  private createStream<T extends PluginState | PluginLogEvent | PluginTelemetry>(topic: Topic, pluginId?: string): Observable<T> {
    this.ensureSocket();
    const key = this.key(topic, pluginId);
    let entry = this.subscriptions.get(key);
    if (!entry) {
      entry = { count: 0, subject: new Subject<PluginState | PluginLogEvent | PluginTelemetry>() };
      this.subscriptions.set(key, entry);
      this.enqueue({ action: 'SUB', topic, pluginId });
    }
    entry.count += 1;
    return new Observable<T>((observer) => {
      const subscription = (entry as { subject: Subject<T> }).subject.subscribe(observer);
      return () => {
        subscription.unsubscribe();
        this.zone.run(() => this.teardown(topic, pluginId));
      };
    });
  }

  private teardown(topic: Topic, pluginId?: string) {
    const key = this.key(topic, pluginId);
    const entry = this.subscriptions.get(key);
    if (!entry) {
      return;
    }
    entry.count -= 1;
    if (entry.count <= 0) {
      this.subscriptions.delete(key);
      entry.subject.complete();
      this.enqueue({ action: 'UNSUB', topic, pluginId });
    }
  }

  private ensureSocket() {
    if (this.socket) {
      return;
    }
    const url = new URL(this.baseUrl || window.location.origin);
    url.protocol = url.protocol.startsWith('https') ? 'wss:' : 'ws:';
    url.pathname = '/ws/plugins';
    const socket = new WebSocket(url.toString());
    socket.onopen = () => {
      this.zone.run(() => {
        this.connected = true;
        while (this.pending.length > 0) {
          const message = this.pending.shift();
          if (message) {
            socket.send(JSON.stringify(message));
          }
        }
      });
    };
    socket.onclose = () => {
      this.zone.run(() => {
        this.connected = false;
        this.socket = undefined;
        setTimeout(() => this.ensureSocket(), 1_000);
      });
    };
    socket.onerror = (error) => {
      console.error('Plugin websocket error', error);
    };
    socket.onmessage = (event) => {
      this.zone.run(() => this.dispatch(event.data));
    };
    this.socket = socket;
  }

  private dispatch(data: string) {
    try {
      const message = JSON.parse(data) as { type: string; payload: unknown };
      if (message.type === 'bootstrap') {
        this.bootstrapSubject.next(message.payload as BootstrapPayload);
        const payload = message.payload as BootstrapPayload;
        for (const plugin of payload.plugins) {
          this.forward('status', plugin.name, plugin);
        }
        for (const snapshot of payload.telemetry ?? []) {
          this.forward('telemetry', snapshot.plugin, snapshot);
        }
        return;
      }
      if (message.type === 'status') {
        const payload = message.payload as PluginState;
        this.forward('status', payload.name, payload);
        this.forward('status', undefined, payload);
        return;
      }
      if (message.type === 'logs') {
        const payload = message.payload as PluginLogEvent;
        this.forward('logs', payload.plugin, payload);
        return;
      }
      if (message.type === 'telemetry') {
        const payload = message.payload as PluginTelemetry;
        this.forward('telemetry', payload.plugin, payload);
        this.forward('telemetry', undefined, payload);
        return;
      }
    } catch (error) {
      console.warn('Failed to parse websocket payload', error);
    }
  }

  private forward(topic: Topic, pluginId: string | undefined, payload: PluginState | PluginLogEvent | PluginTelemetry) {
    const specific = this.subscriptions.get(this.key(topic, pluginId));
    specific?.subject.next(payload);
    if (pluginId) {
      const wildcard = this.subscriptions.get(this.key(topic, undefined));
      wildcard?.subject.next(payload);
    }
  }

  private enqueue(message: ClientMessage) {
    if (this.connected && this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(message));
    } else {
      this.pending.push(message);
    }
  }

  private key(topic: Topic, pluginId?: string) {
    return `${topic}:${pluginId ?? '*'}`;
  }
}
