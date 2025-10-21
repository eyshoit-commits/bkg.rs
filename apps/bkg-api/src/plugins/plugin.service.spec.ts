import { PluginService } from './plugin.service';
import { PluginConfig } from './plugin.types';

describe('PluginService', () => {
  const mockBus = () => ({
    request: jest.fn(),
    setConfig: jest.fn(),
    getPlugins: jest.fn().mockReturnValue([]),
    getPlugin: jest.fn(),
    updateState: jest.fn(),
    on: jest.fn(),
    ensureState: jest.fn(),
  });

  const mockDatabase = () => ({
    path: ':memory:',
    connection: {
      prepare: jest.fn().mockReturnValue({
        run: jest.fn(),
        all: jest.fn().mockReturnValue([]),
      }),
    },
  });

  it('delegates capability invocation to plugin bus', async () => {
    const bus = mockBus();
    bus.request.mockResolvedValue({ ok: true });
    const service = new PluginService(bus as any, mockDatabase() as any);
    await expect(service.invokeCapability('llmserver', 'llm.chat', { messages: [] })).resolves.toEqual({
      ok: true,
    });
    expect(bus.request).toHaveBeenCalledWith('llmserver', 'llm.chat', { messages: [] }, undefined);
  });

  it('persists plugin configuration updates', () => {
    const bus = mockBus();
    const database = mockDatabase();
    const service = new PluginService(bus as any, database as any);
    const config: PluginConfig = {
      name: 'demo',
      description: 'demo plugin',
      entrypoint: 'start.sh',
      capabilities: ['llm.chat'],
    };
    service['configs'].set('demo', config);
    service.saveConfig(config);
    expect(bus.setConfig).toHaveBeenCalledWith('demo', config);
    expect(bus.ensureState).toHaveBeenCalledWith('demo', config);
    expect(database.connection.prepare).toHaveBeenCalled();
  });
});
