#!/usr/bin/env node
const WebSocket = require('ws');
const Database = require('better-sqlite3');
const bcrypt = require('bcryptjs');
const { randomBytes } = require('crypto');
const { v4: uuidv4 } = require('uuid');

const pluginName = process.env.BKG_PLUGIN_NAME || 'apikeys';
const busPort = parseInt(process.env.BKG_PLUGIN_BUS_PORT || '43121', 10);
const dbPath = process.env.BKG_DATABASE_PATH || '/data/bkg.db';
const adminPassword = process.env.ADMIN_PASSWORD;

const db = new Database(dbPath);

db.prepare(
  `CREATE TABLE IF NOT EXISTS sessions (
    token TEXT PRIMARY KEY,
    user TEXT NOT NULL,
    scopes TEXT NOT NULL,
    expires_at INTEGER NOT NULL
  )`,
).run();

ensureAdminUser();

function ensureAdminUser() {
  const existing = db.prepare('SELECT id FROM users WHERE name = ?').get('admin');
  if (existing) {
    return;
  }
  if (!adminPassword) {
    throw new Error('ADMIN_PASSWORD must be provided to initialize admin user');
  }
  const passwordHash = bcrypt.hashSync(adminPassword, 12);
  db.prepare('INSERT INTO users (id, name, password_hash) VALUES (?, ?, ?)').run(
    uuidv4(),
    'admin',
    passwordHash,
  );
  console.log('[apikeys] Admin user initialized');
}

function randomKey() {
  return randomBytes(32).toString('base64url');
}

function normalizeScopes(scopes) {
  if (Array.isArray(scopes)) {
    return scopes;
  }
  if (typeof scopes === 'string') {
    return scopes.split(',').map((scope) => scope.trim()).filter(Boolean);
  }
  return [];
}

function requiredScopeForRequest(payload) {
  const path = payload?.path || '';
  if (path.startsWith('/admin')) {
    return 'admin';
  }
  if (path.includes('/chat')) {
    return 'llm.chat';
  }
  if (path.includes('/embeddings')) {
    return 'llm.embed';
  }
  return 'basic';
}

function scopesAllow(scopes, required) {
  return scopes.includes('*') || scopes.includes(required) || (required === 'basic' && scopes.length > 0);
}

function validateToken(token, payload) {
  const now = Date.now();
  const session = db.prepare('SELECT token, user, scopes, expires_at FROM sessions WHERE token = ?').get(token);
  if (session) {
    if (session.expires_at < now) {
      db.prepare('DELETE FROM sessions WHERE token = ?').run(token);
      throw new Error('Session expired');
    }
    const scopes = JSON.parse(session.scopes);
    const required = requiredScopeForRequest(payload);
    if (!scopesAllow(scopes, required)) {
      throw new Error('Insufficient scope');
    }
    return { user: session.user, scopes };
  }
  const keys = db.prepare('SELECT key, user, scopes FROM api_keys').all();
  for (const record of keys) {
    if (bcrypt.compareSync(token, record.key)) {
      const scopes = JSON.parse(record.scopes);
      const required = requiredScopeForRequest(payload);
      if (!scopesAllow(scopes, required)) {
        throw new Error('Insufficient scope');
      }
      return { user: record.user, scopes };
    }
  }
  throw new Error('Invalid API key');
}

function createSession(user, scopes) {
  const token = randomKey();
  const expiresAt = Date.now() + 24 * 60 * 60 * 1000;
  db.prepare('INSERT INTO sessions (token, user, scopes, expires_at) VALUES (?, ?, ?, ?)').run(
    token,
    user,
    JSON.stringify(scopes),
    expiresAt,
  );
  return { token, user, scopes, expires_at: expiresAt };
}

function createApiKey(user, scopes) {
  const existing = db.prepare('SELECT id FROM users WHERE name = ?').get(user);
  if (!existing) {
    throw new Error('User not found');
  }
  const token = randomKey();
  const hash = bcrypt.hashSync(token, 12);
  db.prepare('INSERT INTO api_keys (key, user, scopes, created_at) VALUES (?, ?, ?, ?)').run(
    hash,
    user,
    JSON.stringify(scopes),
    Date.now(),
  );
  return { token, user, scopes };
}

function listApiKeys() {
  return db
    .prepare('SELECT key, user, scopes, created_at FROM api_keys')
    .all()
    .map((row) => ({
      id: row.key,
      user: row.user,
      scopes: JSON.parse(row.scopes),
      created_at: row.created_at,
      preview: row.key.slice(-8),
    }));
}

function revokeKey(identifier) {
  const result = db.prepare('DELETE FROM api_keys WHERE key = ?').run(identifier);
  if (result.changes === 0) {
    throw new Error('Key not found');
  }
  return { revoked: identifier };
}

function login({ username, password }) {
  const user = db.prepare('SELECT id, name, password_hash FROM users WHERE name = ?').get(username);
  if (!user) {
    throw new Error('Unknown user');
  }
  if (!bcrypt.compareSync(password, user.password_hash)) {
    throw new Error('Invalid credentials');
  }
  const scopes = ['admin', 'llm.chat', 'llm.embed', 'repo.analyze', 'repo.patch'];
  return createSession(user.name, scopes);
}

function handleRequest(message, ws) {
  const { requestId, capability, payload } = message;
  try {
    let result;
    if (capability === 'auth.login') {
      result = login(payload);
    } else if (capability === 'auth.validate') {
      result = validateToken(payload.token, payload);
    } else if (capability === 'auth.createKey') {
      const scopes = normalizeScopes(payload.scopes || []);
      result = createApiKey(payload.user, scopes);
    } else if (capability === 'auth.listKeys') {
      result = listApiKeys();
    } else if (capability === 'auth.revokeKey') {
      result = revokeKey(payload.id || payload.key);
    } else {
      throw new Error(`Unsupported capability ${capability}`);
    }
    ws.send(
      JSON.stringify({
        type: 'response',
        requestId,
        success: true,
        data: result,
      }),
    );
  } catch (error) {
    ws.send(
      JSON.stringify({
        type: 'response',
        requestId,
        success: false,
        error: error.message,
      }),
    );
  }
}

function connect() {
  const url = `ws://127.0.0.1:${busPort}`;
  const ws = new WebSocket(url);
  ws.on('open', () => {
    const heartbeat = setInterval(() => {
      ws.send(
        JSON.stringify({
          type: 'health',
          plugin: pluginName,
          status: 'up',
        }),
      );
    }, 10000);
    ws.send(
      JSON.stringify({
        type: 'register',
        plugin: pluginName,
        port: 'internal',
        capabilities: ['auth.login', 'auth.createKey', 'auth.revokeKey', 'auth.listKeys', 'auth.validate'],
        meta: {},
      }),
    );
    ws.send(
      JSON.stringify({
        type: 'log',
        plugin: pluginName,
        level: 'info',
        message: 'API key plug-in registered',
        timestamp: new Date().toISOString(),
      }),
    );
    ws.once('close', () => clearInterval(heartbeat));
  });
  ws.on('message', (data) => {
    try {
      const message = JSON.parse(data.toString());
      if (message.type === 'request') {
        handleRequest(message, ws);
      }
    } catch (error) {
      console.error('[apikeys] Failed to process message', error);
    }
  });
  ws.on('close', () => {
    console.warn('[apikeys] Plugin bus connection closed, retrying');
    setTimeout(connect, 3000);
  });
  ws.on('error', (error) => {
    console.error('[apikeys] Plugin bus error', error);
  });
}

connect();
