import { HttpException, HttpStatus } from '@nestjs/common';
import { ChatController } from './chat.controller';

describe('ChatController', () => {
  const mockPluginService = (state: any, response: any) => ({
    getPlugin: jest.fn().mockReturnValue(state),
    invokeCapability: jest.fn().mockResolvedValue(response),
  });

  it('forwards chat requests to llmserver', async () => {
    const plugins = mockPluginService({ status: 'running' }, { id: '123' });
    const controller = new ChatController(plugins as any);
    const result = await controller.chat({ messages: [] } as any);
    expect(result).toEqual({ id: '123' });
    expect(plugins.invokeCapability).toHaveBeenCalledWith('llmserver', 'llm.chat', { messages: [] });
  });

  it('throws when llm plugin is unavailable', async () => {
    const controller = new ChatController(mockPluginService(undefined, null) as any);
    await expect(controller.chat({ messages: [] } as any)).rejects.toMatchObject({
      status: HttpStatus.SERVICE_UNAVAILABLE,
    });
  });
});
