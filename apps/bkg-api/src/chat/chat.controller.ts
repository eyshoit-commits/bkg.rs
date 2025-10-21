import {
  Body,
  Controller,
  HttpException,
  HttpStatus,
  Post,
  UseGuards,
} from '@nestjs/common';
import { ChatCompletionDto, EmbeddingRequestDto } from './dto';
import { PluginService } from '../plugins/plugin.service';
import { ApiKeyGuard } from '../common/guards/api-key.guard';

@Controller('/v1')
@UseGuards(ApiKeyGuard)
export class ChatController {
  constructor(private readonly plugins: PluginService) {}

  @Post('/chat/completions')
  async chat(@Body() body: ChatCompletionDto) {
    const plugin = this.plugins.getPlugin('llmserver');
    if (!plugin || plugin.status !== 'running') {
      throw new HttpException('LLM server unavailable', HttpStatus.SERVICE_UNAVAILABLE);
    }
    try {
      return await this.plugins.invokeCapability('llmserver', 'llm.chat', body);
    } catch (error) {
      throw new HttpException((error as Error).message, HttpStatus.BAD_GATEWAY);
    }
  }

  @Post('/embeddings')
  async embed(@Body() body: EmbeddingRequestDto) {
    const plugin = this.plugins.getPlugin('llmserver');
    if (!plugin || plugin.status !== 'running') {
      throw new HttpException('Embedding service unavailable', HttpStatus.SERVICE_UNAVAILABLE);
    }
    try {
      return await this.plugins.invokeCapability('llmserver', 'llm.embed', body);
    } catch (error) {
      throw new HttpException((error as Error).message, HttpStatus.BAD_GATEWAY);
    }
  }
}
