import { Body, Controller, Get, Post, UseGuards } from '@nestjs/common';
import { Type } from 'class-transformer';
import {
  IsBoolean,
  IsInt,
  IsObject,
  IsOptional,
  IsString,
  IsUrl,
  Max,
  Min,
  ValidateNested,
} from 'class-validator';
import { PluginService } from '../plugins/plugin.service';
import { ApiKeyGuard } from '../common/guards/api-key.guard';

class GooseScheduleEntryDto {
  @IsString()
  name!: string;

  @IsString()
  method!: string;

  @IsString()
  path!: string;

  @IsOptional()
  @IsInt()
  @Min(1)
  weight?: number;

  @IsOptional()
  @IsString()
  body?: string;

  @IsOptional()
  @IsObject()
  headers?: Record<string, string>;

  @IsOptional()
  @IsObject()
  query?: Record<string, string>;

  @IsOptional()
  @IsInt()
  @Min(0)
  thinkTimeMs?: number;
}

class GooseRunDto {
  @IsOptional()
  @IsUrl({ require_tld: false })
  target?: string;

  @IsOptional()
  @IsInt()
  @Min(1)
  @Max(100000)
  users?: number;

  @IsOptional()
  @IsInt()
  @Min(1)
  @Max(100000)
  hatchRate?: number;

  @IsOptional()
  @IsInt()
  @Min(1)
  @Max(86400)
  runTimeSeconds?: number;

  @IsOptional()
  @IsInt()
  @Min(1)
  @Max(600)
  timeoutSeconds?: number;

  @IsOptional()
  @IsObject()
  globalHeaders?: Record<string, string>;

  @IsOptional()
  @IsBoolean()
  verifyTls?: boolean;

  @IsOptional()
  @ValidateNested({ each: true })
  @Type(() => GooseScheduleEntryDto)
  schedule?: GooseScheduleEntryDto[];
}

@Controller('/api/goose')
@UseGuards(ApiKeyGuard)
export class GooseController {
  constructor(private readonly plugins: PluginService) {}

  @Post('/run')
  run(@Body() body: GooseRunDto) {
    return this.plugins.invokeCapability('goose', 'goose.run', body ?? {});
  }

  @Post('/stop')
  stop() {
    return this.plugins.invokeCapability('goose', 'goose.stop', {});
  }

  @Get('/status')
  status() {
    return this.plugins.invokeCapability('goose', 'goose.status', {});
  }

  @Get('/history')
  history() {
    return this.plugins.invokeCapability('goose', 'goose.history', {});
  }
}
