import { Injectable, NgZone } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import {
  ApiKeyRecord,
  ChatCompletionResponse,
  ChatMessage,
  EmbeddingResponse,
  PluginConfig,
  PluginLogEvent,
  PluginState,
} from '../models/api.models';

@Injectable({ providedIn: 'root' })
export class ApiService {
  private readonly baseUrl = (document.querySelector('meta[name="bkg-api"]') as HTMLMetaElement)?.content ?? '';

  constructor(private readonly http: HttpClient, private readonly zone: NgZone) {}

  login(username: string, password: string): Observable<{ token: string; scopes: string[] }> {
    return this.http.post<{ token: string; scopes: string[] }>(`${this.baseUrl}/auth/login`, {
      username,
      password,
    });
  }

  chat(messages: ChatMessage[]): Observable<ChatCompletionResponse> {
    return this.http.post<ChatCompletionResponse>(`${this.baseUrl}/v1/chat/completions`, { messages });
  }

  embeddings(input: string[]): Observable<EmbeddingResponse> {
    return this.http.post<EmbeddingResponse>(`${this.baseUrl}/v1/embeddings`, { input });
  }

  listPlugins(): Observable<PluginState[]> {
    return this.http.get<PluginState[]>(`${this.baseUrl}/admin/plugins`);
  }

  startPlugin(name: string): Observable<PluginState> {
    return this.http.post<PluginState>(`${this.baseUrl}/admin/plugins/${name}/start`, {});
  }

  stopPlugin(name: string): Observable<{ status: string }> {
    return this.http.post<{ status: string }>(`${this.baseUrl}/admin/plugins/${name}/stop`, {});
  }

  updatePluginConfig(config: PluginConfig): Observable<PluginState> {
    return this.http.post<PluginState>(`${this.baseUrl}/admin/plugins/${config.name}/config`, config);
  }

  portTable(): Observable<{ service: string; port: string | number; status: string }[]> {
    return this.http.get<{ service: string; port: string | number; status: string }[]>(
      `${this.baseUrl}/admin/ports`,
    );
  }

  listApiKeys(): Observable<ApiKeyRecord[]> {
    return this.http.get<ApiKeyRecord[]>(`${this.baseUrl}/admin/keys`);
  }

  createApiKey(user: string, scopes: string[]): Observable<{ token: string; user: string; scopes: string[] }> {
    return this.http.post<{ token: string; user: string; scopes: string[] }>(
      `${this.baseUrl}/admin/keys`,
      { user, scopes },
    );
  }

  revokeApiKey(id: string): Observable<{ revoked: string }> {
    return this.http.request<{ revoked: string }>('delete', `${this.baseUrl}/admin/keys/${encodeURIComponent(id)}`);
  }

  streamPluginLogs(plugin: string): Observable<PluginLogEvent> {
    const url = `${this.baseUrl}/admin/plugins/${plugin}/logs`;
    return new Observable<PluginLogEvent>((observer) => {
      const source = new EventSource(url);
      source.onmessage = (event) => {
        this.zone.run(() => {
          observer.next(JSON.parse(event.data));
        });
      };
      source.onerror = () => {
        this.zone.run(() => {
          observer.error(new Error('Log stream disconnected'));
          source.close();
        });
      };
      return () => source.close();
    });
  }
}
