import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { ServeStaticModule } from '@nestjs/serve-static';
import { join } from 'path';
import { AppService } from './app.service';
import { PluginModule } from './plugins/plugin.module';
import { ChatController } from './chat/chat.controller';
import { AuthController } from './auth/auth.controller';
import { AdminController } from './admin/admin.controller';
import { HealthController } from './health/health.controller';
import { DatabaseModule } from './storage/database.module';
import { ApiKeyGuard } from './common/guards/api-key.guard';

@Module({
  imports: [
    ConfigModule.forRoot({ isGlobal: true }),
    ServeStaticModule.forRoot({
      rootPath: join(process.cwd(), 'apps', 'bkg-web', 'dist'),
      serveRoot: '/',
      exclude: ['/v1*', '/auth*', '/admin*', '/health'],
    }),
    DatabaseModule,
    PluginModule,
  ],
  controllers: [ChatController, AuthController, AdminController, HealthController],
  providers: [AppService, ApiKeyGuard],
})
export class AppModule {}
