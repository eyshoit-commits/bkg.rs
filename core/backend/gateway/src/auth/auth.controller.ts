import { Body, Controller, Post } from '@nestjs/common';
import { PluginService } from '../plugins/plugin.service';
import { LoginDto } from './dto';

@Controller('/auth')
export class AuthController {
  constructor(private readonly plugins: PluginService) {}

  @Post('/login')
  async login(@Body() dto: LoginDto) {
    return this.plugins.invokeCapability('apikeys', 'auth.login', dto);
  }
}
