import { Controller, Get } from '@nestjs/common';
import { PluginService } from '../plugins/plugin.service';

@Controller()
export class HealthController {
  constructor(private readonly plugins: PluginService) {}

  @Get('/health')
  health() {
    return {
      status: 'ok',
      timestamp: new Date().toISOString(),
      plugins: this.plugins.listPlugins().map((plugin) => ({
        name: plugin.name,
        status: plugin.status,
        port: plugin.port ?? 'n/a',
      })),
    };
  }
}
