import 'reflect-metadata';
import { Logger, ValidationPipe } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import helmet from 'helmet';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create(AppModule, {
    cors: true,
  });
  app.use(helmet({ contentSecurityPolicy: false }));
  app.useGlobalPipes(
    new ValidationPipe({
      whitelist: true,
      transform: true,
      forbidUnknownValues: true,
    }),
  );
  const port = process.env.BKG_API_PORT ? Number.parseInt(process.env.BKG_API_PORT, 10) : 0;
  await app.listen(port, '0.0.0.0');
  const actualPort = (app.getHttpServer().address() as { port: number }).port;
  Logger.log(`API listening on port ${actualPort}`, 'Bootstrap');
}

bootstrap().catch((error) => {
  // eslint-disable-next-line no-console
  console.error('Failed to bootstrap application', error);
  process.exit(1);
});
