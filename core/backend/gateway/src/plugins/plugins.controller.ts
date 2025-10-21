import {
  BadRequestException,
  Body,
  Controller,
  Get,
  Param,
  Post,
  Query,
  UseGuards,
  NotFoundException,
} from '@nestjs/common';
import { PluginService } from './plugin.service';
import { PluginConfig } from './plugin.types';
import { ApiKeyGuard } from '../common/guards/api-key.guard';

@Controller('/api/plugins')
@UseGuards(ApiKeyGuard)
export class PluginsController {
  constructor(private readonly plugins: PluginService) {}

  @Get()
  list() {
    return this.plugins.listPlugins();
  }

  @Get('/:name')
  get(@Param('name') name: string) {
    const plugin = this.plugins.getPlugin(name);
    if (!plugin) {
      throw new NotFoundException(`Plugin ${name} not found`);
    }
    return plugin;
  }

  @Post('/:name/start')
  start(@Param('name') name: string) {
    return this.plugins.startPlugin(name);
  }

  @Post('/:name/stop')
  async stop(@Param('name') name: string) {
    await this.plugins.stopPlugin(name);
    return { status: 'stopped' };
  }

  @Post('/:name/restart')
  restart(@Param('name') name: string) {
    return this.plugins.restartPlugin(name);
  }

  @Get('/:name/status')
  status(@Param('name') name: string) {
    const plugin = this.plugins.getPlugin(name);
    if (!plugin) {
      throw new NotFoundException(`Plugin ${name} not found`);
    }
    return { name: plugin.name, status: plugin.status, pid: plugin.pid, port: plugin.port };
  }

  @Get('/:name/capabilities')
  capabilities(@Param('name') name: string) {
    const plugin = this.plugins.getPlugin(name);
    if (!plugin) {
      throw new NotFoundException(`Plugin ${name} not found`);
    }
    return plugin.capabilities;
  }

  @Post('/:name/config')
  updateConfig(@Param('name') name: string, @Body() config: PluginConfig) {
    if (config.name !== name) {
      throw new BadRequestException('Plugin name mismatch');
    }
    this.plugins.saveConfig(config);
    return this.plugins.getPlugin(name);
  }

  @Get('/:name/logs')
  logs(@Param('name') name: string, @Query('limit') limit?: string) {
    const parsed = limit ? Number.parseInt(limit, 10) : 200;
    const safeLimit = Number.isFinite(parsed) && parsed > 0 ? Math.min(parsed, 500) : 200;
    return this.plugins.getLogs(name, safeLimit);
  }

  @Get('/:name/telemetry')
  telemetry(@Param('name') name: string) {
    const telemetry = this.plugins.getTelemetry(name);
    if (!telemetry) {
      throw new NotFoundException(`No telemetry for ${name}`);
    }
    return telemetry;
  }

  @Get('/telemetry/summary')
  telemetrySummary() {
    return this.plugins.getTelemetry();
  }
}
