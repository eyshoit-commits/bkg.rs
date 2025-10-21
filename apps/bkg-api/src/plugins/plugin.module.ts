import { Module } from '@nestjs/common';
import { PluginBusService } from './plugin-bus.service';
import { PluginService } from './plugin.service';
import { DatabaseModule } from '../storage/database.module';

@Module({
  imports: [DatabaseModule],
  providers: [PluginBusService, PluginService],
  exports: [PluginBusService, PluginService],
})
export class PluginModule {}
