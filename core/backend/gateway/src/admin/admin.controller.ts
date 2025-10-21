import {
  Body,
  Controller,
  Delete,
  Get,
  MessageEvent,
  Param,
  Post,
  Sse,
  UseGuards,
} from '@nestjs/common';
import { Observable } from 'rxjs';
import { PluginService } from '../plugins/plugin.service';
import { ApiKeyGuard } from '../common/guards/api-key.guard';
import { PluginBusService } from '../plugins/plugin-bus.service';
import { PluginConfig } from '../plugins/plugin.types';

@Controller('/admin')
@UseGuards(ApiKeyGuard)
export class AdminController {
  constructor(
    private readonly plugins: PluginService,
    private readonly bus: PluginBusService,
  ) {}

  @Get('/plugins')
  listPlugins() {
    return this.plugins.listPlugins();
  }

  @Post('/plugins/:name/start')
  async startPlugin(@Param('name') name: string) {
    return this.plugins.startPlugin(name);
  }

  @Post('/plugins/:name/stop')
  async stopPlugin(@Param('name') name: string) {
    await this.plugins.stopPlugin(name);
    return { status: 'stopped' };
  }

  @Post('/plugins/:name/restart')
  restartPlugin(@Param('name') name: string) {
    return this.plugins.restartPlugin(name);
  }

  @Post('/plugins/:name/config')
  async updatePluginConfig(@Param('name') name: string, @Body() config: PluginConfig) {
    if (config.name !== name) {
      throw new Error('Name mismatch');
    }
    this.plugins.saveConfig(config);
    return this.plugins.getPlugin(name);
  }

  @Get('/ports')
  portTable() {
    return this.plugins.listPlugins().map((plugin) => ({
      service: plugin.name,
      port: plugin.port ?? 'n/a',
      status: plugin.status,
    }));
  }

  @Get('/keys')
  listKeys() {
    return this.plugins.invokeCapability('apikeys', 'auth.listKeys', {});
  }

  @Post('/keys')
  createKey(@Body() body: { user: string; scopes: string[] }) {
    return this.plugins.invokeCapability('apikeys', 'auth.createKey', body);
  }

  @Delete('/keys/:id')
  revokeKey(@Param('id') id: string) {
    return this.plugins.invokeCapability('apikeys', 'auth.revokeKey', { id });
  }

  @Sse('/plugins/:name/logs')
  streamLogs(@Param('name') name: string): Observable<MessageEvent> {
    return new Observable<MessageEvent>((subscriber) => {
      const handler = (event: { plugin: string; level: string; message: string; timestamp?: string }) => {
        if (event.plugin === name) {
          subscriber.next({ data: event } as MessageEvent);
        }
      };
      this.bus.on('log', handler);
      return () => {
        this.bus.off('log', handler);
      };
    });
  }
}
