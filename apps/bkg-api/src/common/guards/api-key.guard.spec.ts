import { UnauthorizedException } from '@nestjs/common';
import { ApiKeyGuard } from './api-key.guard';

describe('ApiKeyGuard', () => {
  const createContext = (authorization?: string) => ({
    switchToHttp: () => ({
      getRequest: () => ({
        headers: authorization ? { authorization } : {},
        method: 'GET',
        path: '/v1/chat/completions',
      }),
    }),
  });

  it('rejects missing authorization header', async () => {
    const guard = new ApiKeyGuard({ invokeCapability: jest.fn() } as any);
    await expect(guard.canActivate(createContext() as any)).rejects.toBeInstanceOf(UnauthorizedException);
  });

  it('validates token via plugin service', async () => {
    const invokeCapability = jest.fn().mockResolvedValue({ user: 'admin' });
    const guard = new ApiKeyGuard({ invokeCapability } as any);
    await expect(
      guard.canActivate(createContext('Bearer test') as any),
    ).resolves.toBe(true);
    expect(invokeCapability).toHaveBeenCalledWith('apikeys', 'auth.validate', {
      token: 'test',
      method: 'GET',
      path: '/v1/chat/completions',
    });
  });
});
