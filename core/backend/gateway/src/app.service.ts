import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { PluginService } from './plugins/plugin.service';
import { ConfigService } from '@nestjs/config';

@Injectable()
export class AppService implements OnModuleInit {
  private readonly logger = new Logger(AppService.name);

  constructor(
    private readonly plugins: PluginService,
    private readonly config: ConfigService,
  ) {}

  onModuleInit() {
    this.logPortTable();
    this.plugins.listPlugins();
  }

  logPortTable() {
    const table = this.plugins.listPlugins();
    const rows = [
      ['Service', 'Port', 'Status'],
      ['API', this.config.get('BKG_API_PORT') ?? 'dynamic', 'running'],
      ...table.map((plugin) => [
        plugin.name,
        plugin.port?.toString() ?? 'pending',
        plugin.status,
      ]),
    ];
    const widths = rows[0].map((_, index) => Math.max(...rows.map((row) => row[index].length)));
    this.logger.log('Service       Port     Status');
    for (const row of rows) {
      const line = row
        .map((value, index) => value.padEnd(widths[index]))
        .join('    ');
      this.logger.log(line);
    }
  }
}
