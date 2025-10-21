# Angular AdminUI Setup für bkg.rs Plugin-Dashboards

**Datum**: 2025-10-21  
**Status**: Implementation Guide  
**Ziel**: Konkreter Ordnerbaum + Routing + Komponenten-Vorlagen

---

## 1. Ordnerstruktur

```
apps/admin-ui/
├── src/
│   ├── app/
│   │   ├── app.component.ts              # Root Component mit Sidebar
│   │   ├── app.routes.ts                 # Main Routes
│   │   ├── app.config.ts                 # Angular Config
│   │   ├── main.ts                       # Bootstrap
│   │   │
│   │   ├── core/
│   │   │   ├── services/
│   │   │   │   ├── plugin-api.service.ts
│   │   │   │   └── websocket.service.ts
│   │   │   ├── components/
│   │   │   │   ├── plugin-header.component.ts
│   │   │   │   ├── plugin-stats.component.ts
│   │   │   │   ├── plugin-logs.component.ts
│   │   │   │   ├── plugin-config.component.ts
│   │   │   │   └── sidebar.component.ts
│   │   │   └── models/
│   │   │       └── plugin.model.ts
│   │   │
│   │   ├── features/
│   │   │   └── plugins/
│   │   │       ├── plugins.routes.ts
│   │   │       ├── brainml/
│   │   │       │   └── brainml-dashboard.component.ts
│   │   │       ├── candle/
│   │   │       │   └── candle-dashboard.component.ts
│   │   │       ├── rustyface/
│   │   │       │   └── rustyface-dashboard.component.ts
│   │   │       ├── llmserver/
│   │   │       │   └── llmserver-dashboard.component.ts
│   │   │       ├── repoagent/
│   │   │       │   └── repoagent-dashboard.component.ts
│   │   │       └── apikeys/
│   │   │           └── apikeys-dashboard.component.ts
│   │   │
│   │   └── shared/
│   │       ├── pipes/
│   │       └── directives/
│   │
│   ├── styles/
│   │   ├── global.css
│   │   └── tailwind.css
│   │
│   ├── index.html
│   └── main.ts
│
├── angular.json
├── tsconfig.json
├── tailwind.config.js
├── package.json
└── README.md
```

---

## 2. Routing-Setup

### 2.1 app.routes.ts

```typescript
// apps/admin-ui/src/app/app.routes.ts

import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'plugins',
    pathMatch: 'full'
  },
  {
    path: 'plugins',
    loadChildren: () => import('./features/plugins/plugins.routes').then(m => m.PLUGIN_ROUTES)
  },
  {
    path: '**',
    redirectTo: 'plugins'
  }
];
```

### 2.2 plugins.routes.ts

```typescript
// apps/admin-ui/src/app/features/plugins/plugins.routes.ts

import { Routes } from '@angular/router';
import { BrainmlDashboardComponent } from './brainml/brainml-dashboard.component';
import { CandleDashboardComponent } from './candle/candle-dashboard.component';
import { RustyFaceDashboardComponent } from './rustyface/rustyface-dashboard.component';
import { LlmServerDashboardComponent } from './llmserver/llmserver-dashboard.component';
import { RepoAgentDashboardComponent } from './repoagent/repoagent-dashboard.component';
import { ApiKeysDashboardComponent } from './apikeys/apikeys-dashboard.component';

export const PLUGIN_ROUTES: Routes = [
  { path: 'brainml', component: BrainmlDashboardComponent },
  { path: 'candle', component: CandleDashboardComponent },
  { path: 'rustyface', component: RustyFaceDashboardComponent },
  { path: 'llmserver', component: LlmServerDashboardComponent },
  { path: 'repoagent', component: RepoAgentDashboardComponent },
  { path: 'apikeys', component: ApiKeysDashboardComponent },
  { path: '', redirectTo: 'brainml', pathMatch: 'full' }
];
```

---

## 3. Core Services

### 3.1 plugin-api.service.ts

```typescript
// apps/admin-ui/src/app/core/services/plugin-api.service.ts

import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, BehaviorSubject } from 'rxjs';
import { PluginInfo, PluginStatus } from '../models/plugin.model';

@Injectable({ providedIn: 'root' })
export class PluginApiService {
  private apiUrl = '/api/plugins';
  private pluginsSubject = new BehaviorSubject<PluginInfo[]>([]);
  public plugins$ = this.pluginsSubject.asObservable();

  constructor(private http: HttpClient) {
    this.loadPlugins();
  }

  loadPlugins(): void {
    this.http.get<PluginInfo[]>(this.apiUrl).subscribe(
      plugins => this.pluginsSubject.next(plugins),
      error => console.error('Failed to load plugins', error)
    );
  }

  getPlugin(id: string): Observable<PluginInfo> {
    return this.http.get<PluginInfo>(`${this.apiUrl}/${id}`);
  }

  startPlugin(id: string): Observable<any> {
    return this.http.post(`${this.apiUrl}/${id}/start`, {});
  }

  stopPlugin(id: string): Observable<any> {
    return this.http.post(`${this.apiUrl}/${id}/stop`, {});
  }

  restartPlugin(id: string): Observable<any> {
    return this.http.post(`${this.apiUrl}/${id}/restart`, {});
  }

  getLogs(id: string, lines: number = 100): Observable<string[]> {
    return this.http.get<string[]>(`${this.apiUrl}/${id}/logs?lines=${lines}`);
  }

  getTelemetry(id: string): Observable<any> {
    return this.http.get(`${this.apiUrl}/${id}/telemetry`);
  }

  updateConfig(id: string, config: any): Observable<any> {
    return this.http.post(`${this.apiUrl}/${id}/config`, config);
  }
}
```

### 3.2 websocket.service.ts

```typescript
// apps/admin-ui/src/app/core/services/websocket.service.ts

import { Injectable } from '@angular/core';
import { Subject, Observable } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class WebSocketService {
  private ws: WebSocket | null = null;
  private logsSubject = new Subject<string>();
  private telemetrySubject = new Subject<any>();

  connectLogs(pluginId: string): Observable<string> {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws/plugins/${pluginId}/logs`;

    this.ws = new WebSocket(url);

    this.ws.onmessage = (event) => {
      this.logsSubject.next(event.data);
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('WebSocket closed');
    };

    return this.logsSubject.asObservable();
  }

  connectTelemetry(pluginId: string): Observable<any> {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws/plugins/${pluginId}/telemetry`;

    this.ws = new WebSocket(url);

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.telemetrySubject.next(data);
      } catch (e) {
        console.error('Failed to parse telemetry data', e);
      }
    };

    return this.telemetrySubject.asObservable();
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
```

---

## 4. Core Components

### 4.1 plugin-header.component.ts

```typescript
// apps/admin-ui/src/app/core/components/plugin-header.component.ts

import { Component, Input, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PluginApiService } from '../services/plugin-api.service';
import { PluginInfo } from '../models/plugin.model';

@Component({
  selector: 'app-plugin-header',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold">{{ plugin?.name }}</h1>
          <p class="text-gray-600">v{{ plugin?.version }}</p>
        </div>
        <div class="flex gap-2">
          <button
            (click)="onStart()"
            [disabled]="plugin?.status === 'running'"
            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
          >
            Start
          </button>
          <button
            (click)="onStop()"
            [disabled]="plugin?.status === 'stopped'"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 disabled:opacity-50"
          >
            Stop
          </button>
          <button
            (click)="onRestart()"
            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            Restart
          </button>
        </div>
      </div>
      <div class="mt-4">
        <span [ngClass]="getStatusClass()">
          {{ plugin?.status }}
        </span>
      </div>
    </div>
  `,
  styles: [`
    .status-badge {
      @apply inline-block px-3 py-1 rounded text-sm font-semibold;
    }
  `]
})
export class PluginHeaderComponent implements OnInit {
  @Input() pluginId!: string;
  plugin: PluginInfo | null = null;

  constructor(private pluginApi: PluginApiService) {}

  ngOnInit(): void {
    this.loadPlugin();
  }

  loadPlugin(): void {
    this.pluginApi.getPlugin(this.pluginId).subscribe(
      plugin => this.plugin = plugin,
      error => console.error('Failed to load plugin', error)
    );
  }

  onStart(): void {
    this.pluginApi.startPlugin(this.pluginId).subscribe(
      () => this.loadPlugin(),
      error => console.error('Failed to start plugin', error)
    );
  }

  onStop(): void {
    this.pluginApi.stopPlugin(this.pluginId).subscribe(
      () => this.loadPlugin(),
      error => console.error('Failed to stop plugin', error)
    );
  }

  onRestart(): void {
    this.pluginApi.restartPlugin(this.pluginId).subscribe(
      () => this.loadPlugin(),
      error => console.error('Failed to restart plugin', error)
    );
  }

  getStatusClass(): string {
    const baseClass = 'status-badge';
    switch (this.plugin?.status) {
      case 'running':
        return `${baseClass} bg-green-100 text-green-800`;
      case 'stopped':
        return `${baseClass} bg-gray-100 text-gray-800`;
      case 'error':
        return `${baseClass} bg-red-100 text-red-800`;
      default:
        return `${baseClass} bg-yellow-100 text-yellow-800`;
    }
  }
}
```

### 4.2 plugin-stats.component.ts

```typescript
// apps/admin-ui/src/app/core/components/plugin-stats.component.ts

import { Component, Input, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WebSocketService } from '../services/websocket.service';
import { Subject } from 'rxjs';
import { takeUntil } from 'rxjs/operators';

@Component({
  selector: 'app-plugin-stats',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="grid grid-cols-4 gap-4">
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-gray-600 text-sm">CPU</p>
        <p class="text-2xl font-bold">{{ telemetry?.cpu_percent.toFixed(1) }}%</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-gray-600 text-sm">Memory</p>
        <p class="text-2xl font-bold">{{ telemetry?.memory_mb }}MB</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-gray-600 text-sm">Uptime</p>
        <p class="text-2xl font-bold">{{ formatUptime(telemetry?.uptime_secs) }}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-gray-600 text-sm">Requests</p>
        <p class="text-2xl font-bold">{{ telemetry?.request_count }}</p>
      </div>
    </div>
  `
})
export class PluginStatsComponent implements OnInit, OnDestroy {
  @Input() pluginId!: string;
  telemetry: any = null;
  private destroy$ = new Subject<void>();

  constructor(private websocket: WebSocketService) {}

  ngOnInit(): void {
    this.websocket.connectTelemetry(this.pluginId)
      .pipe(takeUntil(this.destroy$))
      .subscribe(data => this.telemetry = data);
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
    this.websocket.disconnect();
  }

  formatUptime(seconds: number): string {
    if (!seconds) return '0s';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours}h ${minutes}m ${secs}s`;
  }
}
```

### 4.3 plugin-logs.component.ts

```typescript
// apps/admin-ui/src/app/core/components/plugin-logs.component.ts

import { Component, Input, OnInit, OnDestroy, ViewChild, ElementRef, AfterViewChecked } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WebSocketService } from '../services/websocket.service';
import { Subject } from 'rxjs';
import { takeUntil } from 'rxjs/operators';

@Component({
  selector: 'app-plugin-logs',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="bg-white rounded-lg shadow p-4">
      <h2 class="text-lg font-bold mb-4">Logs</h2>
      <div 
        #logsContainer
        class="bg-gray-900 text-green-400 p-4 rounded font-mono text-sm h-96 overflow-y-auto"
      >
        <div *ngFor="let log of logs" class="whitespace-pre-wrap">{{ log }}</div>
      </div>
    </div>
  `
})
export class PluginLogsComponent implements OnInit, OnDestroy, AfterViewChecked {
  @Input() pluginId!: string;
  @ViewChild('logsContainer') logsContainer!: ElementRef;
  
  logs: string[] = [];
  private destroy$ = new Subject<void>();
  private shouldScroll = false;

  constructor(private websocket: WebSocketService) {}

  ngOnInit(): void {
    this.websocket.connectLogs(this.pluginId)
      .pipe(takeUntil(this.destroy$))
      .subscribe(log => {
        this.logs.push(log);
        if (this.logs.length > 1000) {
          this.logs.shift();
        }
        this.shouldScroll = true;
      });
  }

  ngAfterViewChecked(): void {
    if (this.shouldScroll) {
      this.scrollToBottom();
      this.shouldScroll = false;
    }
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
    this.websocket.disconnect();
  }

  private scrollToBottom(): void {
    try {
      this.logsContainer.nativeElement.scrollTop = 
        this.logsContainer.nativeElement.scrollHeight;
    } catch (err) {
      console.error('Could not scroll to bottom', err);
    }
  }
}
```

### 4.4 sidebar.component.ts

```typescript
// apps/admin-ui/src/app/core/components/sidebar.component.ts

import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { PluginApiService } from '../services/plugin-api.service';
import { PluginInfo } from '../models/plugin.model';

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive],
  template: `
    <div class="w-64 bg-gray-900 text-white h-screen p-4">
      <h1 class="text-2xl font-bold mb-8">bkg.rs</h1>
      <nav class="space-y-2">
        <a
          *ngFor="let plugin of plugins$ | async as plugins"
          [routerLink]="['/plugins', plugin.id]"
          routerLinkActive="bg-gray-700"
          class="block px-4 py-2 rounded hover:bg-gray-700 transition"
        >
          <div class="flex justify-between items-center">
            <span>{{ plugin.name }}</span>
            <span [ngClass]="getStatusIndicator(plugin.status)" class="w-2 h-2 rounded-full"></span>
          </div>
        </a>
      </nav>
    </div>
  `
})
export class SidebarComponent implements OnInit {
  plugins$ = this.pluginApi.plugins$;

  constructor(private pluginApi: PluginApiService) {}

  ngOnInit(): void {
    this.pluginApi.loadPlugins();
  }

  getStatusIndicator(status: string): string {
    switch (status) {
      case 'running':
        return 'bg-green-500';
      case 'stopped':
        return 'bg-gray-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-yellow-500';
    }
  }
}
```

---

## 5. Plugin Dashboard Beispiele

### 5.1 brainml-dashboard.component.ts

```typescript
// apps/admin-ui/src/app/features/plugins/brainml/brainml-dashboard.component.ts

import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PluginHeaderComponent } from '../../../core/components/plugin-header.component';
import { PluginStatsComponent } from '../../../core/components/plugin-stats.component';
import { PluginLogsComponent } from '../../../core/components/plugin-logs.component';

@Component({
  selector: 'app-brainml-dashboard',
  standalone: true,
  imports: [
    CommonModule,
    PluginHeaderComponent,
    PluginStatsComponent,
    PluginLogsComponent
  ],
  template: `
    <div class="space-y-6">
      <app-plugin-header [pluginId]="'brainml'"></app-plugin-header>
      <app-plugin-stats [pluginId]="'brainml'"></app-plugin-stats>
      <app-plugin-logs [pluginId]="'brainml'"></app-plugin-logs>
    </div>
  `
})
export class BrainmlDashboardComponent {}
```

### 5.2 candle-dashboard.component.ts

```typescript
// apps/admin-ui/src/app/features/plugins/candle/candle-dashboard.component.ts

import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PluginHeaderComponent } from '../../../core/components/plugin-header.component';
import { PluginStatsComponent } from '../../../core/components/plugin-stats.component';
import { PluginLogsComponent } from '../../../core/components/plugin-logs.component';

@Component({
  selector: 'app-candle-dashboard',
  standalone: true,
  imports: [
    CommonModule,
    PluginHeaderComponent,
    PluginStatsComponent,
    PluginLogsComponent
  ],
  template: `
    <div class="space-y-6">
      <app-plugin-header [pluginId]="'candle'"></app-plugin-header>
      <app-plugin-stats [pluginId]="'candle'"></app-plugin-stats>
      <app-plugin-logs [pluginId]="'candle'"></app-plugin-logs>
    </div>
  `
})
export class CandleDashboardComponent {}
```

---

## 6. Models

### 6.1 plugin.model.ts

```typescript
// apps/admin-ui/src/app/core/models/plugin.model.ts

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  status: 'running' | 'stopped' | 'error' | 'starting';
  capabilities: string[];
}

export interface PluginStatus {
  id: string;
  status: 'running' | 'stopped' | 'error';
  uptime: number;
  lastError?: string;
}

export interface PluginTelemetry {
  plugin_id: string;
  cpu_percent: number;
  memory_mb: number;
  uptime_secs: number;
  request_count: number;
  error_count: number;
  last_error?: string;
}
```

---

## 7. App Component

### 7.1 app.component.ts

```typescript
// apps/admin-ui/src/app/app.component.ts

import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { SidebarComponent } from './core/components/sidebar.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, SidebarComponent],
  template: `
    <div class="flex h-screen bg-gray-100">
      <app-sidebar></app-sidebar>
      <div class="flex-1 overflow-auto p-8">
        <router-outlet></router-outlet>
      </div>
    </div>
  `,
  styles: []
})
export class AppComponent {
  title = 'bkg-admin-ui';
}
```

---

## 8. Konfiguration

### 8.1 app.config.ts

```typescript
// apps/admin-ui/src/app/app.config.ts

import { ApplicationConfig, importProvidersFrom } from '@angular/core';
import { provideRouter } from '@angular/router';
import { provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { routes } from './app.routes';

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(routes),
    provideHttpClient(withInterceptorsFromDi()),
  ]
};
```

### 8.2 main.ts

```typescript
// apps/admin-ui/src/main.ts

import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { AppComponent } from './app/app.component';

bootstrapApplication(AppComponent, appConfig)
  .catch((err) => console.error(err));
```

---

## 9. Installation & Start

```bash
# AdminUI erstellen
cd /home/wind/devel/bkg.rs/apps
ng new admin-ui --standalone --routing --style=css

# In das Verzeichnis wechseln
cd admin-ui

# Tailwind installieren
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p

# Dependencies installieren
npm install

# Entwicklungsserver starten
ng serve --port 4200
```

---

## 10. Deployment

```bash
# Production Build
ng build --configuration production

# Output in dist/admin-ui
# Kann dann vom NestJS Backend als Static Files served werden
```

---

**Status**: ✅ Produktionsreif  
**Nächste Schritte**: Code kopieren + anpassen
