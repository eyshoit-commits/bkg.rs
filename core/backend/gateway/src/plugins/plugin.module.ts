import { Module } from '@nestjs/common';
import { PluginBusService } from './plugin-bus.service';
import { PluginService } from './plugin.service';
import { DatabaseModule } from '../storage/database.module';
import { PluginsGateway } from './plugins.gateway';

@Module({
  imports: [DatabaseModule],
  providers: [PluginBusService, PluginService, PluginsGateway],
  exports: [PluginBusService, PluginService],
})
export class PluginModule {}
