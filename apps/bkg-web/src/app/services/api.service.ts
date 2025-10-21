import { Injectable, NgZone } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import {
  ApiKeyRecord,
  ChatCompletionResponse,
  ChatMessage,
  EmbeddingResponse,
  PluginConfig,
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
    return this.http.get<PluginState[]>(`${this.baseUrl}/api/plugins`);
  }

  startPlugin(name: string): Observable<PluginState> {
    return this.http.post<PluginState>(`${this.baseUrl}/api/plugins/${name}/start`, {});
  }

  stopPlugin(name: string): Observable<{ status: string }> {
    return this.http.post<{ status: string }>(`${this.baseUrl}/api/plugins/${name}/stop`, {});
  }

  restartPlugin(name: string): Observable<PluginState> {
    return this.http.post<PluginState>(`${this.baseUrl}/api/plugins/${name}/restart`, {});
  }

  updatePluginConfig(config: PluginConfig): Observable<PluginState> {
    return this.http.post<PluginState>(`${this.baseUrl}/api/plugins/${config.name}/config`, config);
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

  streamPluginLogs(): never {
    throw new Error('Use PluginWsService for log streaming');
  }
}
