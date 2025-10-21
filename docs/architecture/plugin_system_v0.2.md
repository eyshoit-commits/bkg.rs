# bkg.rs Plugin-System Architektur (v0.2)

**Version**: 0.2  
**Datum**: 2025-10-21  
**Status**: Design Document

---

## 1. Überblick

Das bkg.rs Plugin-System v0.2 ist eine vollständig modulare, Hot-Swap-fähige Architektur, bei der:

- **Backend (Rust)**: Ein zentraler Plugin-Bus orchestriert alle Plug-ins via RPC
- **Frontend (Angular 17)**: Ein dynamisches Admin-Dashboard steuert Plug-ins in Echtzeit
- **Isolation**: Jedes Plug-in läuft in eigenem Prozess oder Thread
- **Skalierbarkeit**: Neue Plug-ins ohne Core-Änderungen hinzufügbar

---

## 2. Backend-Architektur

### 2.1 Plugin-Traits (Rust)

```rust
// core/src/plugin_traits.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub status: PluginStatus,
    pub capabilities: Vec<PluginCapability>,
    pub config: PluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub id: String,
    pub enabled: bool,
    pub autostart: bool,
    pub env: HashMap<String, String>,
    pub resources: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_cpu_percent: u32,
    pub timeout_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTelemetry {
    pub plugin_id: String,
    pub cpu_percent: f32,
    pub memory_mb: u32,
    pub uptime_secs: u64,
    pub request_count: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
}

// Haupttrait für alle Plug-ins
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn info(&self) -> PluginInfo;
    fn capabilities(&self) -> Vec<PluginCapability>;
    
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn restart(&mut self) -> Result<(), String>;
    fn status(&self) -> PluginStatus;
    
    fn invoke(&self, capability: &str, payload: serde_json::Value) 
        -> Result<serde_json::Value, String>;
    
    fn config(&self) -> &PluginConfig;
    fn set_config(&mut self, config: PluginConfig) -> Result<(), String>;
    
    fn telemetry(&self) -> PluginTelemetry;
    fn logs(&self, lines: usize) -> Vec<String>;
}
```

### 2.2 Plugin-Registry (Rust)

```rust
// core/src/plugin_registry.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::plugin_traits::*;

pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register(&self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        let id = plugin.id().to_string();
        
        if plugins.contains_key(&id) {
            return Err(format!("Plugin {} already registered", id));
        }
        
        plugins.insert(id, plugin);
        Ok(())
    }
    
    pub fn unregister(&self, id: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        plugins.remove(id)
            .ok_or_else(|| format!("Plugin {} not found", id))?;
        Ok(())
    }
    
    pub fn get(&self, id: &str) -> Result<Arc<Box<dyn Plugin>>, String> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(id)
            .ok_or_else(|| format!("Plugin {} not found", id))
            .map(|p| Arc::new(p.clone()))
    }
    
    pub fn list(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().map(|p| p.info()).collect()
    }
    
    pub fn start(&self, id: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        let plugin = plugins.get_mut(id)
            .ok_or_else(|| format!("Plugin {} not found", id))?;
        plugin.start()
    }
    
    pub fn stop(&self, id: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        let plugin = plugins.get_mut(id)
            .ok_or_else(|| format!("Plugin {} not found", id))?;
        plugin.stop()
    }
    
    pub fn invoke(&self, id: &str, capability: &str, payload: serde_json::Value) 
        -> Result<serde_json::Value, String> {
        let plugins = self.plugins.read().unwrap();
        let plugin = plugins.get(id)
            .ok_or_else(|| format!("Plugin {} not found", id))?;
        plugin.invoke(capability, payload)
    }
}
```

### 2.3 Plugin-Bus (Rust)

```rust
// core/src/plugin_bus.rs

use tokio::sync::mpsc;
use serde_json::{json, Value};

pub struct PluginBus {
    registry: Arc<PluginRegistry>,
    tx: mpsc::UnboundedSender<BusMessage>,
}

pub enum BusMessage {
    Invoke {
        plugin_id: String,
        capability: String,
        payload: Value,
        response_tx: mpsc::UnboundedSender<Result<Value, String>>,
    },
    Status {
        plugin_id: String,
        response_tx: mpsc::UnboundedSender<PluginStatus>,
    },
    Telemetry {
        plugin_id: String,
        response_tx: mpsc::UnboundedSender<PluginTelemetry>,
    },
}

impl PluginBus {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let registry_clone = registry.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    BusMessage::Invoke { plugin_id, capability, payload, response_tx } => {
                        let result = registry_clone.invoke(&plugin_id, &capability, payload);
                        let _ = response_tx.send(result);
                    }
                    BusMessage::Status { plugin_id, response_tx } => {
                        if let Ok(plugin) = registry_clone.get(&plugin_id) {
                            let _ = response_tx.send(plugin.status());
                        }
                    }
                    BusMessage::Telemetry { plugin_id, response_tx } => {
                        if let Ok(plugin) = registry_clone.get(&plugin_id) {
                            let _ = response_tx.send(plugin.telemetry());
                        }
                    }
                }
            }
        });
        
        Self { registry, tx }
    }
    
    pub async fn invoke(&self, plugin_id: &str, capability: &str, payload: Value) 
        -> Result<Value, String> {
        let (response_tx, mut response_rx) = mpsc::unbounded_channel();
        
        self.tx.send(BusMessage::Invoke {
            plugin_id: plugin_id.to_string(),
            capability: capability.to_string(),
            payload,
            response_tx,
        }).map_err(|e| e.to_string())?;
        
        response_rx.recv().await
            .ok_or_else(|| "No response from plugin bus".to_string())?
    }
}
```

### 2.4 NestJS API Integration

```typescript
// apps/bkg-api/src/plugins/plugins.controller.ts

import { Controller, Get, Post, Param, Body } from '@nestjs/common';
import { PluginService } from './plugins.service';

@Controller('api/plugins')
export class PluginsController {
  constructor(private readonly pluginService: PluginService) {}

  @Get()
  async listPlugins() {
    return this.pluginService.listPlugins();
  }

  @Get(':id')
  async getPlugin(@Param('id') id: string) {
    return this.pluginService.getPlugin(id);
  }

  @Post(':id/start')
  async startPlugin(@Param('id') id: string) {
    return this.pluginService.startPlugin(id);
  }

  @Post(':id/stop')
  async stopPlugin(@Param('id') id: string) {
    return this.pluginService.stopPlugin(id);
  }

  @Post(':id/restart')
  async restartPlugin(@Param('id') id: string) {
    return this.pluginService.restartPlugin(id);
  }

  @Get(':id/telemetry')
  async getTelemetry(@Param('id') id: string) {
    return this.pluginService.getTelemetry(id);
  }

  @Get(':id/logs')
  async getLogs(@Param('id') id: string) {
    return this.pluginService.getLogs(id);
  }

  @Post(':id/invoke')
  async invokeCapability(
    @Param('id') id: string,
    @Body() { capability, payload }: { capability: string; payload: any },
  ) {
    return this.pluginService.invokeCapability(id, capability, payload);
  }

  @Post(':id/config')
  async updateConfig(@Param('id') id: string, @Body() config: any) {
    return this.pluginService.updateConfig(id, config);
  }
}
```

---

## 3. Frontend-Architektur (Angular 17)

### 3.1 Plugin-State Management

```typescript
// apps/admin-ui/src/app/shared/services/plugin.store.ts

import { Injectable } from '@angular/core';
import { signalStore, withState, withMethods, withComputed } from '@ngrx/signals';
import { computed } from '@angular/core';

export interface PluginState {
  id: string;
  name: string;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'error';
  version: string;
  capabilities: string[];
  telemetry: {
    cpu: number;
    memory: number;
    uptime: number;
  };
}

@Injectable({ providedIn: 'root' })
export class PluginStore {
  private store = signalStore(
    withState(() => ({
      plugins: [] as PluginState[],
      selectedPlugin: null as PluginState | null,
      loading: false,
      error: null as string | null,
    })),
    withMethods((store) => ({
      loadPlugins: async () => {
        store.setLoading(true);
        try {
          const response = await fetch('/api/plugins');
          const plugins = await response.json();
          store.setPlugins(plugins);
        } catch (error) {
          store.setError((error as Error).message);
        } finally {
          store.setLoading(false);
        }
      },
      
      startPlugin: async (id: string) => {
        try {
          await fetch(`/api/plugins/${id}/start`, { method: 'POST' });
          await this.store.loadPlugins();
        } catch (error) {
          store.setError((error as Error).message);
        }
      },
      
      stopPlugin: async (id: string) => {
        try {
          await fetch(`/api/plugins/${id}/stop`, { method: 'POST' });
          await this.store.loadPlugins();
        } catch (error) {
          store.setError((error as Error).message);
        }
      },
      
      selectPlugin: (plugin: PluginState | null) => {
        store.setSelectedPlugin(plugin);
      },
      
      setPlugins: (plugins: PluginState[]) => {
        store.patchState({ plugins });
      },
      
      setSelectedPlugin: (plugin: PluginState | null) => {
        store.patchState({ selectedPlugin: plugin });
      },
      
      setLoading: (loading: boolean) => {
        store.patchState({ loading });
      },
      
      setError: (error: string | null) => {
        store.patchState({ error });
      },
    })),
    withComputed((store) => ({
      runningPlugins: computed(() => 
        store.plugins().filter(p => p.status === 'running')
      ),
      stoppedPlugins: computed(() => 
        store.plugins().filter(p => p.status === 'stopped')
      ),
    }))
  );

  plugins = this.store.plugins;
  selectedPlugin = this.store.selectedPlugin;
  loading = this.store.loading;
  error = this.store.error;
  runningPlugins = this.store.runningPlugins;
  stoppedPlugins = this.store.stoppedPlugins;

  loadPlugins = this.store.loadPlugins;
  startPlugin = this.store.startPlugin;
  stopPlugin = this.store.stopPlugin;
  selectPlugin = this.store.selectPlugin;
}
```

### 3.2 Shared Components

```typescript
// apps/admin-ui/src/app/shared/components/plugin-card.component.ts

import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PluginState } from '../services/plugin.store';

@Component({
  selector: 'app-plugin-card',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="bg-white rounded-lg shadow p-6 hover:shadow-lg transition">
      <div class="flex justify-between items-start mb-4">
        <div>
          <h3 class="text-lg font-semibold">{{ plugin.name }}</h3>
          <p class="text-sm text-gray-500">v{{ plugin.version }}</p>
        </div>
        <span [ngClass]="getStatusClass()">
          {{ plugin.status }}
        </span>
      </div>
      
      <div class="flex gap-2 mb-4">
        <button 
          (click)="onStart()"
          [disabled]="plugin.status !== 'stopped'"
          class="px-3 py-1 bg-green-500 text-white rounded text-sm disabled:opacity-50"
        >
          Start
        </button>
        <button 
          (click)="onStop()"
          [disabled]="plugin.status !== 'running'"
          class="px-3 py-1 bg-red-500 text-white rounded text-sm disabled:opacity-50"
        >
          Stop
        </button>
        <button 
          (click)="onSelect()"
          class="px-3 py-1 bg-blue-500 text-white rounded text-sm"
        >
          Details
        </button>
      </div>
      
      <div class="text-xs text-gray-600">
        <p>CPU: {{ plugin.telemetry.cpu.toFixed(1) }}%</p>
        <p>Memory: {{ plugin.telemetry.memory }}MB</p>
        <p>Uptime: {{ plugin.telemetry.uptime }}s</p>
      </div>
    </div>
  `,
})
export class PluginCardComponent {
  @Input() plugin!: PluginState;
  @Output() start = new EventEmitter<void>();
  @Output() stop = new EventEmitter<void>();
  @Output() select = new EventEmitter<void>();

  getStatusClass(): string {
    const baseClass = 'px-2 py-1 rounded text-xs font-semibold';
    switch (this.plugin.status) {
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

  onStart() {
    this.start.emit();
  }

  onStop() {
    this.stop.emit();
  }

  onSelect() {
    this.select.emit();
  }
}
```

### 3.3 Plugin-Dashboard

```typescript
// apps/admin-ui/src/app/admin/plugin-dashboard.component.ts

import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PluginStore } from '../shared/services/plugin.store';
import { PluginCardComponent } from '../shared/components/plugin-card.component';

@Component({
  selector: 'app-plugin-dashboard',
  standalone: true,
  imports: [CommonModule, PluginCardComponent],
  template: `
    <div class="p-8">
      <h1 class="text-3xl font-bold mb-8">Plugin Dashboard</h1>
      
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div *ngFor="let plugin of pluginStore.plugins()">
          <app-plugin-card 
            [plugin]="plugin"
            (start)="pluginStore.startPlugin(plugin.id)"
            (stop)="pluginStore.stopPlugin(plugin.id)"
            (select)="pluginStore.selectPlugin(plugin)"
          />
        </div>
      </div>
      
      <div *ngIf="pluginStore.selectedPlugin() as selected" class="mt-8 p-6 bg-white rounded-lg shadow">
        <h2 class="text-2xl font-bold mb-4">{{ selected.name }} Details</h2>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <p class="text-sm text-gray-600">Status</p>
            <p class="font-semibold">{{ selected.status }}</p>
          </div>
          <div>
            <p class="text-sm text-gray-600">Version</p>
            <p class="font-semibold">{{ selected.version }}</p>
          </div>
          <div>
            <p class="text-sm text-gray-600">Capabilities</p>
            <p class="font-semibold">{{ selected.capabilities.join(', ') }}</p>
          </div>
        </div>
      </div>
    </div>
  `,
})
export class PluginDashboardComponent implements OnInit {
  constructor(public pluginStore: PluginStore) {}

  ngOnInit() {
    this.pluginStore.loadPlugins();
  }
}
```

---

## 4. Plug-in-Beispiel: Candle

```rust
// plugins/candle/src/lib.rs

use bkg_core::plugin_traits::*;
use serde_json::{json, Value};

pub struct CandlePlugin {
    id: String,
    status: PluginStatus,
    config: PluginConfig,
}

impl CandlePlugin {
    pub fn new() -> Self {
        Self {
            id: "candle".to_string(),
            status: PluginStatus::Stopped,
            config: PluginConfig {
                id: "candle".to_string(),
                enabled: true,
                autostart: false,
                env: Default::default(),
                resources: ResourceLimits {
                    max_memory_mb: 4096,
                    max_cpu_percent: 80,
                    timeout_secs: 300,
                },
            },
        }
    }
}

impl Plugin for CandlePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: self.id.clone(),
            name: "Candle".to_string(),
            version: "0.1.0".to_string(),
            description: "Hugging Face Candle ML Integration".to_string(),
            author: "lofmas".to_string(),
            status: self.status.clone(),
            capabilities: self.capabilities(),
            config: self.config.clone(),
        }
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![
            PluginCapability {
                id: "model.load".to_string(),
                name: "Load Model".to_string(),
                description: "Load a model from Hugging Face".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "model_id": { "type": "string" },
                        "revision": { "type": "string" }
                    }
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "model_id": { "type": "string" },
                        "loaded": { "type": "boolean" }
                    }
                }),
            },
            PluginCapability {
                id: "model.infer".to_string(),
                name: "Inference".to_string(),
                description: "Run inference on loaded model".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "output": { "type": "string" }
                    }
                }),
            },
        ]
    }

    fn start(&mut self) -> Result<(), String> {
        self.status = PluginStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        self.status = PluginStatus::Stopped;
        Ok(())
    }

    fn restart(&mut self) -> Result<(), String> {
        self.stop()?;
        self.start()
    }

    fn status(&self) -> PluginStatus {
        self.status.clone()
    }

    fn invoke(&self, capability: &str, payload: Value) -> Result<Value, String> {
        match capability {
            "model.load" => {
                // Candle model loading logic
                Ok(json!({
                    "model_id": payload["model_id"],
                    "loaded": true
                }))
            }
            "model.infer" => {
                // Candle inference logic
                Ok(json!({
                    "output": "inference result"
                }))
            }
            _ => Err(format!("Unknown capability: {}", capability)),
        }
    }

    fn config(&self) -> &PluginConfig {
        &self.config
    }

    fn set_config(&mut self, config: PluginConfig) -> Result<(), String> {
        self.config = config;
        Ok(())
    }

    fn telemetry(&self) -> PluginTelemetry {
        PluginTelemetry {
            plugin_id: self.id.clone(),
            cpu_percent: 0.0,
            memory_mb: 0,
            uptime_secs: 0,
            request_count: 0,
            error_count: 0,
            last_error: None,
        }
    }

    fn logs(&self, _lines: usize) -> Vec<String> {
        vec![]
    }
}
```

---

## 5. WebSocket Integration

```typescript
// apps/admin-ui/src/app/shared/services/websocket.service.ts

import { Injectable } from '@angular/core';
import { Subject } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class WebSocketService {
  private ws: WebSocket | null = null;
  private logs$ = new Subject<string>();
  private telemetry$ = new Subject<any>();

  connect(pluginId: string) {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws/plugins/${pluginId}`;
    
    this.ws = new WebSocket(url);
    
    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'log') {
        this.logs$.next(data.message);
      } else if (data.type === 'telemetry') {
        this.telemetry$.next(data);
      }
    };
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  getLogs() {
    return this.logs$.asObservable();
  }

  getTelemetry() {
    return this.telemetry$.asObservable();
  }
}
```

---

## 6. Deployment & Skalierung

### 6.1 Docker Multi-Stage Build

```dockerfile
# Dockerfile für v0.2

# Stage 1: Rust Core & Plugins
FROM rust:latest as rust-builder
WORKDIR /build
COPY core ./core
COPY plugins ./plugins
RUN cargo build --release -p candle -p rustyface -p brainml

# Stage 2: Node.js Backend & Frontend
FROM node:20-bullseye as node-builder
WORKDIR /build
COPY apps/bkg-api ./bkg-api
COPY apps/admin-ui ./admin-ui
RUN cd bkg-api && npm install && npm run build
RUN cd admin-ui && npm install && npm run build

# Stage 3: Runtime
FROM debian:bookworm-slim
# ... copy artifacts from stages 1 & 2
```

### 6.2 Kubernetes Deployment (optional)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bkg-plugin-host
spec:
  replicas: 1
  selector:
    matchLabels:
      app: bkg-plugin-host
  template:
    metadata:
      labels:
        app: bkg-plugin-host
    spec:
      containers:
      - name: bkg-api
        image: bkg:v0.2
        ports:
        - containerPort: 43119
        env:
        - name: BKG_PLUGIN_BUS_PORT
          value: "43121"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
```

---

## 7. Sicherheit & Best Practices

### 7.1 Plugin-Isolation

- **Process Isolation**: Jedes Plug-in in eigenem Prozess
- **Resource Limits**: CPU/Memory-Limits pro Plug-in
- **Timeout**: Automatischer Timeout bei Hang
- **Error Handling**: Graceful Degradation bei Fehler

### 7.2 API-Sicherheit

- **Authentication**: Bearer Token via apikeys-Plug-in
- **Authorization**: Scope-basierte Zugriffskontrolle
- **Rate Limiting**: Pro Plug-in & Capability
- **Input Validation**: JSON Schema Validation

### 7.3 Logging & Monitoring

- **Structured Logging**: JSON-Logs mit Kontext
- **Telemetry**: CPU/Memory/Request-Metriken
- **Tracing**: Distributed Tracing via OpenTelemetry
- **Alerting**: Prometheus + Grafana

---

## 8. Roadmap

| Phase | Ziel | Aufwand | Zeitrahmen |
|-------|------|---------|-----------|
| **Phase 1** | Core Registry, Candle Plugin | 12h | Woche 1 |
| **Phase 2** | RustyFace, Admin-UI | 16h | Woche 2 |
| **Phase 3** | WebSocket, Telemetry | 12h | Woche 3 |
| **Phase 4** | Testing, Docs, Release | 8h | Woche 4 |

---

**Autor**: lofmas  
**Version**: 0.2 (Draft)  
**Status**: 📋 Design Document  
**Nächste Aktion**: Proxy-Fix + Core-Implementierung
