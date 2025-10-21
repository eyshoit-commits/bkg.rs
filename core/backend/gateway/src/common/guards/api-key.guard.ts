import {
  CanActivate,
  ExecutionContext,
  Injectable,
  UnauthorizedException,
} from '@nestjs/common';
import { Request } from 'express';
import { PluginService } from '../../plugins/plugin.service';

declare global {
  namespace Express {
    interface Request {
      apiKey?: string;
    }
  }
}

@Injectable()
export class ApiKeyGuard implements CanActivate {
  constructor(private readonly plugins: PluginService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const request = context.switchToHttp().getRequest<Request>();
    const authHeader = request.headers.authorization;
    if (!authHeader?.startsWith('Bearer ')) {
      throw new UnauthorizedException('Missing bearer token');
    }
    const token = authHeader.slice('Bearer '.length).trim();
    try {
      await this.plugins.invokeCapability('apikeys', 'auth.validate', {
        token,
        method: request.method,
        path: request.path,
      });
      request['apiKey'] = token;
      return true;
    } catch (error) {
      throw new UnauthorizedException((error as Error).message);
    }
  }
}
