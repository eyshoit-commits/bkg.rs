# 🚀 BKG v0.2 Setup & Integration Guide

## Überblick

Dieses Patch führt dich durch die **2-Step Integration** von:
1. **PostgresML + BrainML** (Vektoren, Semantische Suche)
2. **APIKeys + Auth System** (Benutzer, Rollen, Permissions)

---

## 📋 Voraussetzungen

- Docker & Docker Compose
- PostgreSQL 14+
- Node.js 20+
- Git

---

## 🔧 STEP 1: PostgresML + BrainML Integration

### 1.1 Docker Compose aktualisieren

**devops/docker/docker-compose.yml:**

```yaml
version: '3.8'

services:
  # BKG Application
  bkg-app:
    build:
      context: ../..
      dockerfile: devops/docker/Dockerfile
    ports:
      - "43119:43119"
      - "43121:43121"
    depends_on:
      - postgres
    environment:
      - BKG_DATABASE_URL=postgresql://pgml:pgml@postgres:5432/bkg
      - BKG_PGML_ENABLED=true
      - BKG_API_PORT=43119
      - BKG_PLUGIN_BUS_PORT=43121
      - CHAT_MODEL_FILE=Qwen2-0.5B-Instruct-Q5_K_M.gguf
      - EMBEDDING_MODEL_FILE=all-MiniLM-L6-v2-ggml-model-f16.gguf
    volumes:
      - bkg-data:/data
      - ./models:/srv/models:ro
    networks:
      - bkg-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:43119/health"]
      interval: 10s
      timeout: 5s
      retries: 5

  # PostgreSQL + pgml Extension
  postgres:
    image: ghcr.io/postgresml/postgresml:2.7.12
    ports:
      - "5432:5432"
      - "8000:8000"  # pgml Dashboard
    environment:
      - POSTGRES_PASSWORD=pgml
      - POSTGRES_USER=pgml
      - POSTGRES_DB=bkg
    volumes:
      - postgres_data:/var/lib/postgresql
      - ./init-pgml.sql:/docker-entrypoint-initdb.d/01-pgml.sql
      - ./init-apikeys.sql:/docker-entrypoint-initdb.d/02-apikeys.sql
    networks:
      - bkg-network
    healthcheck:
      test: ["CMD", "pg_isready", "-U", "pgml"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  bkg-data:

networks:
  bkg-network:
    driver: bridge
```

### 1.2 PostgresML Init Script

**devops/docker/init-pgml.sql:**

```sql
-- Enable pgml Extension
CREATE EXTENSION IF NOT EXISTS pgml;

-- Create BrainML Schema
CREATE SCHEMA IF NOT EXISTS brainml;

-- Documents Table
CREATE TABLE IF NOT EXISTS brainml.documents (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    embedding vector(384),
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Create Index für Vektorsuche
CREATE INDEX IF NOT EXISTS idx_documents_embedding 
ON brainml.documents USING ivfflat (embedding vector_cosine_ops);

-- Collections Table
CREATE TABLE IF NOT EXISTS brainml.collections (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Document-Collection Mapping
CREATE TABLE IF NOT EXISTS brainml.document_collections (
    document_id INT REFERENCES brainml.documents(id) ON DELETE CASCADE,
    collection_id INT REFERENCES brainml.collections(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, collection_id)
);

-- Grant Permissions
GRANT ALL PRIVILEGES ON SCHEMA brainml TO pgml;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA brainml TO pgml;
```

### 1.3 Start Docker

```bash
cd /home/wind/devel/bkg.rs

# Build & Start
docker compose -f devops/docker/docker-compose.yml up -d

# Wait for services
sleep 15

# Check status
docker compose -f devops/docker/docker-compose.yml ps

# Test connections
curl http://localhost:43119/health
curl http://localhost:5432 -v  # PostgreSQL
# pgml Dashboard: http://localhost:8000
```

### 1.4 Verify BrainML Setup

```bash
# Check pgml extension
docker exec -it bkg-app psql postgresql://pgml:pgml@postgres:5432/bkg -c "SELECT * FROM pgml.version();"

# Check brainml schema
docker exec -it bkg-app psql postgresql://pgml:pgml@postgres:5432/bkg -c "SELECT * FROM information_schema.tables WHERE table_schema = 'brainml';"
```

---

## 🔐 STEP 2: APIKeys + Auth System Integration

### 2.1 APIKeys Init Script

**devops/docker/init-apikeys.sql:**

```sql
-- Enable Extensions
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS uuid-ossp;

-- Create Auth Schema
CREATE SCHEMA IF NOT EXISTS auth;

-- Users Table
CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_secret VARCHAR(255),
    last_login TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Roles Table
CREATE TABLE IF NOT EXISTS auth.roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Permissions Table
CREATE TABLE IF NOT EXISTS auth.permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    resource VARCHAR(100),
    action VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Role-Permission Mapping
CREATE TABLE IF NOT EXISTS auth.role_permissions (
    role_id INT REFERENCES auth.roles(id) ON DELETE CASCADE,
    permission_id INT REFERENCES auth.permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- User-Role Mapping
CREATE TABLE IF NOT EXISTS auth.user_roles (
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    role_id INT REFERENCES auth.roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- API Keys Table
CREATE TABLE IF NOT EXISTS auth.api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    prefix VARCHAR(20),
    last_used TIMESTAMP,
    expires_at TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    scopes TEXT[] DEFAULT ARRAY['read'],
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Sessions Table
CREATE TABLE IF NOT EXISTS auth.sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Audit Logs Table
CREATE TABLE IF NOT EXISTS auth.audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id VARCHAR(255),
    changes JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create Indexes
CREATE INDEX IF NOT EXISTS idx_users_username ON auth.users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON auth.users(email);
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON auth.api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON auth.sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON auth.audit_logs(user_id);

-- Insert Default Roles
INSERT INTO auth.roles (name, description) VALUES
    ('admin', 'Vollständiger Zugriff'),
    ('user', 'Standard-Benutzer'),
    ('viewer', 'Nur-Lese-Zugriff'),
    ('developer', 'Entwickler mit API-Zugriff')
ON CONFLICT (name) DO NOTHING;

-- Insert Default Permissions
INSERT INTO auth.permissions (name, description, resource, action) VALUES
    ('plugins.read', 'Plugins anzeigen', 'plugins', 'read'),
    ('plugins.write', 'Plugins verwalten', 'plugins', 'write'),
    ('users.read', 'Benutzer anzeigen', 'users', 'read'),
    ('users.write', 'Benutzer verwalten', 'users', 'write'),
    ('apikeys.read', 'API-Keys anzeigen', 'api-keys', 'read'),
    ('apikeys.write', 'API-Keys erstellen', 'api-keys', 'write'),
    ('audit.read', 'Audit-Logs anzeigen', 'audit', 'read')
ON CONFLICT (name) DO NOTHING;

-- Grant Permissions
GRANT ALL PRIVILEGES ON SCHEMA auth TO pgml;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth TO pgml;
```

### 2.2 Bootstrap Admin User

```bash
# Navigate to apikeys plugin
cd /home/wind/devel/bkg.rs/core/plugins/apikeys

# Install dependencies
npm install

# Run bootstrap script
BKG_DATABASE_URL=postgresql://pgml:pgml@localhost:5432/bkg \
ADMIN_USERNAME=admin \
ADMIN_EMAIL=admin@bkg.local \
node bootstrap.js
```

### 2.3 Output Example

```
╔════════════════════════════════════════════════════════════╗
║              ✓ BOOTSTRAP SUCCESSFUL                        ║
╚════════════════════════════════════════════════════════════╝

📝 Admin Credentials:
  Username: admin
  Email: admin@bkg.local
  Password: your-secure-password

🔑 API Key:
  Key ID: 550e8400-e29b-41d4-a716-446655440000
  Full Key: bkg_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6

📌 Usage:
  Login:
    curl -X POST http://localhost:43119/api/auth/login \
      -H 'Content-Type: application/json' \
      -d '{"username": "admin", "password": "your-secure-password"}'

  API Request:
    curl -H 'Authorization: Bearer bkg_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6' \
      http://localhost:43119/api/plugins

✓ Setup complete! You can now access BKG at:
  http://localhost:43119
```

---

## ✅ Verification Checklist

### Step 1: PostgresML
- [ ] Docker containers running (`docker ps`)
- [ ] PostgreSQL healthy (`curl http://localhost:5432`)
- [ ] pgml extension loaded (`SELECT * FROM pgml.version()`)
- [ ] brainml schema created (`SELECT * FROM information_schema.tables WHERE table_schema = 'brainml'`)
- [ ] pgml Dashboard accessible (http://localhost:8000)

### Step 2: APIKeys
- [ ] Auth schema created (`SELECT * FROM information_schema.tables WHERE table_schema = 'auth'`)
- [ ] Admin user created (`SELECT * FROM auth.users WHERE username = 'admin'`)
- [ ] Admin role assigned (`SELECT * FROM auth.user_roles`)
- [ ] API key generated (`SELECT * FROM auth.api_keys`)
- [ ] Bootstrap credentials saved (`cat /tmp/bkg-bootstrap-credentials.json`)

---

## 🔗 API Endpoints (Step 2)

```
POST   /api/auth/register          # Neuer Benutzer
POST   /api/auth/login             # Login
POST   /api/auth/logout            # Logout
GET    /api/auth/me                # Aktuelle User-Info

POST   /api/apikeys                # Neuer API-Key
GET    /api/apikeys                # Meine API-Keys
DELETE /api/apikeys/:id            # API-Key widerrufen

GET    /api/users                  # Alle Benutzer (Admin)
POST   /api/users/:id/roles        # Rolle zuweisen
DELETE /api/users/:id              # Benutzer löschen

GET    /api/roles                  # Alle Rollen
GET    /api/permissions            # Alle Permissions

GET    /api/audit                  # Audit-Logs
```

---

## 🎯 Nächste Schritte

1. ✅ Step 1: PostgresML + BrainML
2. ✅ Step 2: APIKeys + Auth
3. 🔧 Implementiere Services (Rust, Node.js, TypeScript)
4. 🎨 Baue Dashboards (Angular Components)
5. 📝 Schreibe Tests
6. 🚀 Deploy v0.2

---

## 📚 Weitere Dokumentation

- [PostgresML + BrainML Integration](./PGML_BRAINML_INTEGRATION.md)
- [APIKeys Auth System](./APIKEYS_AUTH_SYSTEM.md)
- [Project Structure](../CORE_STRUCTURE.md)

