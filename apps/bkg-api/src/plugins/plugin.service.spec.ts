import { PluginService } from './plugin.service';
import { PluginConfig } from './plugin.types';
import { spawn } from 'child_process';
import * as fs from 'fs';

jest.mock('child_process', () => ({
  spawn: jest.fn(),
}));

const spawnMock = spawn as jest.MockedFunction<typeof spawn>;

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

  afterEach(() => {
    spawnMock.mockReset();
    jest.restoreAllMocks();
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

  it('terminates plugin process when registration times out', async () => {
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
    const existsSpy = jest.spyOn(fs, 'existsSync').mockReturnValue(true);
    const child: any = {
      stdout: { on: jest.fn() },
      stderr: { on: jest.fn() },
      on: jest.fn().mockReturnThis(),
      kill: jest.fn(),
      pid: 1234,
      killed: false,
    };
    child.on.mockImplementation(function register(event: string, handler: (...args: any[]) => void) {
      if (event === 'exit') {
        child.exitHandler = handler;
      }
      return this;
    });
    spawnMock.mockReturnValue(child);
    const waitSpy = jest
      .spyOn(service as any, 'waitForPluginReady')
      .mockRejectedValue(new Error('Plugin demo did not register before timeout'));

    await expect(service.startPlugin('demo')).rejects.toThrow('Plugin demo did not register before timeout');

    expect(child.kill).toHaveBeenCalledWith('SIGKILL');
    expect(service['processes'].has('demo')).toBe(false);
    expect(bus.updateState).toHaveBeenCalledWith('demo', expect.any(Function));

    waitSpy.mockRestore();
    existsSpy.mockRestore();
  });
});
