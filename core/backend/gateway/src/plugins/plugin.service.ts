import { Injectable, Logger, OnModuleDestroy, OnModuleInit } from '@nestjs/common';
import { join } from 'path';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';
import { spawn, ChildProcess } from 'child_process';
import { PluginBusService } from './plugin-bus.service';
import {
  PluginCapability,
  PluginConfig,
  PluginLogMessage,
  PluginRuntimeState,
  PluginTelemetrySnapshot,
} from './plugin.types';
import { DatabaseService } from '../storage/database.service';

interface PluginProcess {
  process: ChildProcess;
  config: PluginConfig;
}

@Injectable()
export class PluginService implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(PluginService.name);
  private readonly processes = new Map<string, PluginProcess>();
  private readonly configs = new Map<string, PluginConfig>();
  private readonly configRoot = join(process.cwd(), 'core', 'plugins');
  private readonly configPath = join(this.configRoot, 'plugins.json');

  constructor(
    private readonly bus: PluginBusService,
    private readonly database: DatabaseService,
  ) {}

  onModuleInit() {
    this.ensureConfigStorage();
    const configs = this.loadConfigs();
    configs.forEach((config) => this.configs.set(config.name, config));
    for (const config of configs) {
      if (config.autostart) {
        this.startPlugin(config.name).catch((error) => {
          this.logger.error(`Failed to auto-start plugin ${config.name}`, error);
        });
      } else {
        this.bus.ensureState(config.name, config);
        this.bus.setConfig(config.name, {
          ...config,
        });
      }
      this.persistConfig(config);
    }
    this.bus.on('pluginUnregistered', (name: string) => {
      const state = this.bus.getPlugin(name);
      if (state) {
        this.bus.updateState(name, (current) => ({ ...current, status: 'stopped' }));
      }
    });
  }

  onModuleDestroy() {
    for (const [name] of this.processes) {
      this.stopPlugin(name).catch((error) => {
        this.logger.error(`Failed to stop plugin ${name}`, error);
      });
    }
  }

  listPlugins(): PluginRuntimeState[] {
    const states = this.bus.getPlugins();
    for (const config of this.configs.values()) {
      const existing = states.find((state) => state.name === config.name);
      if (!existing) {
        states.push({
          name: config.name,
          status: 'stopped',
          pid: undefined,
          port: undefined,
          lastHeartbeat: undefined,
          capabilities: config.capabilities,
          config,
        });
      }
    }
    return states;
  }

  getPlugin(name: string): PluginRuntimeState | undefined {
    const state = this.bus.getPlugin(name);
    if (state) {
      return state;
    }
    const config = this.configs.get(name);
    if (!config) {
      return undefined;
    }
    return {
      name,
      status: 'stopped',
      pid: undefined,
      port: undefined,
      lastHeartbeat: undefined,
      capabilities: config.capabilities,
      config,
    };
  }

  getLogs(name: string, limit = 200): PluginLogMessage[] {
    return this.bus.getLogs(name, limit);
  }

  getTelemetry(name?: string): PluginTelemetrySnapshot | PluginTelemetrySnapshot[] | undefined {
    if (name) {
      return this.bus.getTelemetrySnapshot(name);
    }
    return this.bus.getAllTelemetry();
  }

  async startPlugin(name: string): Promise<PluginRuntimeState> {
    const existing = this.processes.get(name);
    if (existing) {
      throw new Error(`Plugin ${name} already running`);
    }
    const config = this.configs.get(name);
    if (!config) {
      throw new Error(`No configuration for plugin ${name}`);
    }
    const entry = join(this.configRoot, name, config.entrypoint);
    if (!existsSync(entry)) {
      throw new Error(`Entrypoint ${entry} does not exist`);
    }
    const env = {
      ...process.env,
      ...(config.env ?? {}),
      BKG_PLUGIN_BUS_PORT: this.bus.port.toString(),
      BKG_PLUGIN_NAME: name,
      BKG_DATABASE_PATH: this.database.path,
    };
    this.bus.ensureState(name, config);
    this.bus.setConfig(name, config);
    const child = spawn(entry, config.args ?? [], {
      cwd: join(this.configRoot, name),
      env,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    child.stdout?.on('data', (data) => {
      this.logger.log(`[${name}] ${data.toString().trim()}`);
    });
    child.stderr?.on('data', (data) => {
      this.logger.error(`[${name}] ${data.toString().trim()}`);
    });
    child.on('error', (error) => {
      this.logger.error(`Plugin ${name} failed to start`, error);
      void this.forceStopOnFailure(name, child);
    });
    child.on('exit', (code, signal) => {
      this.logger.warn(`Plugin ${name} exited with code ${code} signal ${signal}`);
      this.processes.delete(name);
      this.bus.updateState(name, (state) => ({
        ...state,
        status: 'stopped',
        pid: undefined,
      }));
    });
    this.processes.set(name, { process: child, config });
    this.bus.updateState(name, (state) => ({
      ...state,
      status: 'starting',
      pid: child.pid,
    }));
    this.logger.log(`Started plugin ${name} with pid ${child.pid}`);
    try {
      return await this.waitForPluginReady(name, 30_000);
    } catch (error) {
      this.logger.error(`Plugin ${name} failed to register in time`, error as Error);
      this.forceStopOnFailure(name, child);
      throw error;
    }
  }

  async stopPlugin(name: string): Promise<void> {
    const processInfo = this.processes.get(name);
    if (!processInfo) {
      return;
    }
    processInfo.process.kill();
    this.processes.delete(name);
    this.bus.updateState(name, (state) => ({
      ...state,
      status: 'stopped',
      pid: undefined,
    }));
    this.logger.log(`Stopped plugin ${name}`);
  }

  async restartPlugin(name: string): Promise<PluginRuntimeState> {
    await this.stopPlugin(name);
    return this.startPlugin(name);
  }

  async invokeCapability<T = unknown>(
    plugin: string,
    capability: PluginCapability,
    payload: unknown,
    token?: string,
  ): Promise<T> {
    return this.bus.request<T>(plugin, capability, payload, token);
  }

  saveConfig(config: PluginConfig): void {
    this.configs.set(config.name, config);
    const configs = Array.from(this.configs.values());
    writeFileSync(this.configPath, JSON.stringify(configs, null, 2));
    this.bus.ensureState(config.name, config);
    this.bus.setConfig(config.name, config);
    this.persistConfig(config);
  }

  private forceStopOnFailure(name: string, child: ChildProcess) {
    if (!child.killed) {
      child.kill('SIGKILL');
    }
    this.processes.delete(name);
    this.bus.updateState(name, (state) => ({
      ...state,
      status: 'stopped',
      pid: undefined,
    }));
  }

  private ensureConfigStorage() {
    const dir = this.configRoot;
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }
    if (!existsSync(this.configPath)) {
      writeFileSync(this.configPath, JSON.stringify([], null, 2));
    }
  }

  private loadConfigs(): PluginConfig[] {
    const rows = this.database.connection.prepare('SELECT config FROM plugins').all();
    if (rows.length > 0) {
      return rows.map((row: { config: string }) => JSON.parse(row.config) as PluginConfig);
    }
    const raw = readFileSync(this.configPath, 'utf8');
    try {
      const configs = JSON.parse(raw) as PluginConfig[];
      return configs;
    } catch (error) {
      this.logger.error('Failed to parse plugin configuration file', error as Error);
      return [];
    }
  }

  private persistConfig(config: PluginConfig) {
    this.database.connection
      .prepare(
        `INSERT INTO plugins (name, description, capabilities, autostart, config)
         VALUES (@name, @description, @capabilities, @autostart, @config)
         ON CONFLICT(name) DO UPDATE SET
           description = excluded.description,
           capabilities = excluded.capabilities,
           autostart = excluded.autostart,
           config = excluded.config`,
      )
      .run({
        name: config.name,
        description: config.description,
        capabilities: JSON.stringify(config.capabilities),
        autostart: config.autostart ? 1 : 0,
        config: JSON.stringify(config),
      });
  }

  private waitForPluginReady(name: string, timeoutMs: number): Promise<PluginRuntimeState> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        cleanup();
        reject(new Error(`Plugin ${name} did not register before timeout`));
      }, timeoutMs);
      const onRegistered = (state: PluginRuntimeState) => {
        if (state.name === name) {
          cleanup();
          resolve(state);
        }
      };
      const cleanup = () => {
        clearTimeout(timeout);
        this.bus.off('pluginRegistered', onRegistered);
      };
      this.bus.on('pluginRegistered', onRegistered);
    });
  }
}
