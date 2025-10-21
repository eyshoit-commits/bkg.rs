#!/usr/bin/env node

const { Pool } = require('pg');
const bcrypt = require('bcrypt');
const crypto = require('crypto');

// Colors
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
};

const log = {
  info: (msg) => console.log(`${colors.blue}ℹ ${msg}${colors.reset}`),
  success: (msg) => console.log(`${colors.green}✓ ${msg}${colors.reset}`),
  error: (msg) => console.log(`${colors.red}✗ ${msg}${colors.reset}`),
  warn: (msg) => console.log(`${colors.yellow}⚠ ${msg}${colors.reset}`),
};

async function main() {
  console.log(`
${colors.blue}╔════════════════════════════════════════════════════════════╗${colors.reset}
${colors.blue}║     BKG Bootstrap - Admin User & API-Key Setup             ║${colors.reset}
${colors.blue}╚════════════════════════════════════════════════════════════╝${colors.reset}
`);

  // Configuration from environment or defaults
  const adminUsername = process.env.ADMIN_USERNAME || 'admin';
  const adminEmail = process.env.ADMIN_EMAIL || 'admin@bkg.local';
  const adminPassword = process.env.ADMIN_PASSWORD || crypto.randomBytes(8).toString('hex');
  const databaseUrl = process.env.BKG_DATABASE_URL || process.env.DATABASE_URL || 'postgresql://pgml:pgml@localhost:5432/bkg';

  console.log(`${colors.blue}📋 Configuration:${colors.reset}`);
  console.log(`  Username: ${adminUsername}`);
  console.log(`  Email: ${adminEmail}`);
  console.log(`  Database: ${databaseUrl}`);
  console.log('');

  const pool = new Pool({ connectionString: databaseUrl });

  try {
    // Test connection
    log.info('Testing database connection...');
    await pool.query('SELECT 1');
    log.success('Database connection successful');
    console.log('');

    // Create admin user
    log.info('Creating admin user...');
    const passwordHash = await bcrypt.hash(adminPassword, 10);

    const userResult = await pool.query(
      `INSERT INTO auth.users (username, email, password_hash, full_name, is_active)
       VALUES ($1, $2, $3, $4, true)
       ON CONFLICT (username) DO UPDATE SET password_hash = $3
       RETURNING id`,
      [adminUsername, adminEmail, passwordHash, 'Administrator']
    );

    const adminId = userResult.rows[0].id;
    log.success(`Admin user created: ${adminId}`);
    console.log('');

    // Assign admin role
    log.info('Assigning admin role...');
    const roleResult = await pool.query('SELECT id FROM auth.roles WHERE name = $1', ['admin']);

    if (roleResult.rows.length === 0) {
      log.error('Admin role not found. Ensure database is initialized.');
      process.exit(1);
    }

    const adminRoleId = roleResult.rows[0].id;

    await pool.query(
      `INSERT INTO auth.user_roles (user_id, role_id)
       VALUES ($1, $2)
       ON CONFLICT DO NOTHING`,
      [adminId, adminRoleId]
    );

    log.success('Admin role assigned');
    console.log('');

    // Generate API Key
    log.info('Generating API key...');
    const apiKeyRandom = crypto.randomBytes(16).toString('hex');
    const apiKeyPrefix = 'bkg_live_';
    const apiKeyFull = apiKeyPrefix + apiKeyRandom;
    const apiKeyHash = crypto.createHash('sha256').update(apiKeyFull).digest('hex');

    const keyResult = await pool.query(
      `INSERT INTO auth.api_keys (user_id, name, key_hash, prefix, scopes, is_active)
       VALUES ($1, $2, $3, $4, $5, true)
       RETURNING id`,
      [adminId, 'Bootstrap Admin Key', apiKeyHash, apiKeyPrefix, ['read', 'write', 'admin']]
    );

    const apiKeyId = keyResult.rows[0].id;
    log.success('API key generated');
    console.log('');

    // Log bootstrap action
    try {
      await pool.query(
        `INSERT INTO auth.audit_logs (user_id, action, resource_type, resource_id)
         VALUES ($1, $2, $3, $4)`,
        [adminId, 'bootstrap_admin', 'user', adminId]
      );
    } catch (e) {
      // Ignore if audit table doesn't exist yet
    }

    // Display credentials
    console.log(`
${colors.green}╔════════════════════════════════════════════════════════════╗${colors.reset}
${colors.green}║              ✓ BOOTSTRAP SUCCESSFUL                        ║${colors.reset}
${colors.green}╚════════════════════════════════════════════════════════════╝${colors.reset}
`);

    console.log(`${colors.yellow}📝 Admin Credentials:${colors.reset}`);
    console.log(`  ${colors.blue}Username:${colors.reset} ${colors.green}${adminUsername}${colors.reset}`);
    console.log(`  ${colors.blue}Email:${colors.reset} ${colors.green}${adminEmail}${colors.reset}`);
    console.log(`  ${colors.blue}Password:${colors.reset} ${colors.green}${adminPassword}${colors.reset}`);
    console.log('');

    console.log(`${colors.yellow}🔑 API Key:${colors.reset}`);
    console.log(`  ${colors.blue}Key ID:${colors.reset} ${colors.green}${apiKeyId}${colors.reset}`);
    console.log(`  ${colors.blue}Full Key:${colors.reset} ${colors.green}${apiKeyFull}${colors.reset}`);
    console.log(`  ${colors.blue}Scopes:${colors.reset} ${colors.green}read, write, admin${colors.reset}`);
    console.log('');

    console.log(`${colors.yellow}📌 Usage:${colors.reset}`);
    console.log(`  ${colors.blue}Login:${colors.reset}`);
    console.log(`    curl -X POST http://localhost:43119/api/auth/login \\`);
    console.log(`      -H 'Content-Type: application/json' \\`);
    console.log(`      -d '{"username": "${adminUsername}", "password": "${adminPassword}"}'`);
    console.log('');
    console.log(`  ${colors.blue}API Request:${colors.reset}`);
    console.log(`    curl -H 'Authorization: Bearer ${apiKeyFull}' \\`);
    console.log(`      http://localhost:43119/api/plugins`);
    console.log('');

    console.log(`${colors.yellow}⚠️  IMPORTANT:${colors.reset}`);
    console.log('  - Save these credentials in a secure location');
    console.log('  - Never share the API key or password');
    console.log('  - Change the password after first login');
    console.log('');

    console.log(`${colors.blue}✓ Setup complete! You can now access BKG at:${colors.reset}`);
    console.log('  http://localhost:43119');
    console.log('');

    // Save credentials to file
    const fs = require('fs');
    const credentialsFile = '/tmp/bkg-bootstrap-credentials.json';
    fs.writeFileSync(
      credentialsFile,
      JSON.stringify(
        {
          username: adminUsername,
          email: adminEmail,
          password: adminPassword,
          apiKeyId,
          apiKeyFull,
          createdAt: new Date().toISOString(),
        },
        null,
        2
      )
    );

    log.success(`Credentials saved to: ${credentialsFile}`);

    await pool.end();
  } catch (error) {
    log.error(`Failed: ${error.message}`);
    console.error(error);
    process.exit(1);
  }
}

main();
