# 🚀 BKG v0.2 Complete Setup & Integration Guide

## Überblick

Dieses Patch führt dich durch die **komplette v0.2 Integration** mit:
1. **APIKeys Plugin** (Zentrale Auth & Key-Verwaltung)
2. **PostgresML + BrainML** (Vektoren, Semantische Suche, Hybrid Retrieval)
3. **Goose Plugin** (Load Testing mit Dashboard)
4. **RepoAgent Plugin** (Code Analysis mit Dashboard)
5. **Alle Dashboards** (Admin UI für alle Plugins)

---

## 📋 Voraussetzungen

- Docker & Docker Compose
- PostgreSQL 14+
- Node.js 20+
- Python 3.10+
- Rust 1.70+
- Git

---

## 🔐 STEP 1: APIKeys Plugin - Zentrale Auth

### 1.1 Database Schema

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

-- API Keys Table (ZENTRAL - für alle Plugins)
CREATE TABLE IF NOT EXISTS auth.api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    prefix VARCHAR(20),
    plugin_name VARCHAR(100),  -- goose, repoagent, pgml, brainml
    last_used TIMESTAMP,
    expires_at TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    scopes TEXT[] DEFAULT ARRAY['read'],
    metadata JSONB,  -- Plugin-spezifische Daten
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
CREATE INDEX IF NOT EXISTS idx_api_keys_plugin ON auth.api_keys(plugin_name);
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
    ('audit.read', 'Audit-Logs anzeigen', 'audit', 'read'),
    ('goose.read', 'Goose Tests anzeigen', 'goose', 'read'),
    ('goose.write', 'Goose Tests starten/stoppen', 'goose', 'write'),
    ('repoagent.read', 'RepoAgent Analysen anzeigen', 'repoagent', 'read'),
    ('repoagent.write', 'RepoAgent Analysen starten', 'repoagent', 'write'),
    ('pgml.read', 'PostgresML Models anzeigen', 'pgml', 'read'),
    ('pgml.write', 'PostgresML Models trainieren', 'pgml', 'write'),
    ('brainml.read', 'BrainML Documents anzeigen', 'brainml', 'read'),
    ('brainml.write', 'BrainML Documents verwalten', 'brainml', 'write')
ON CONFLICT (name) DO NOTHING;

-- Grant Permissions
GRANT ALL PRIVILEGES ON SCHEMA auth TO pgml;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth TO pgml;
```

### 1.2 Bootstrap Admin User

```bash
cd /home/wind/devel/bkg.rs/core/plugins/apikeys
npm install
BKG_DATABASE_URL=postgresql://pgml:pgml@localhost:5432/bkg node bootstrap.js
```

---

## 🧠 STEP 2: PostgresML + BrainML Integration

### 2.1 PostgresML Schema

**devops/docker/init-pgml.sql:**

```sql
-- Enable pgml Extension
CREATE EXTENSION IF NOT EXISTS pgml;

-- Create BrainML Schema
CREATE SCHEMA IF NOT EXISTS brainml;

-- Documents Table (für alle Plugins)
CREATE TABLE IF NOT EXISTS brainml.documents (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    embedding vector(384),
    plugin_source VARCHAR(100),  -- goose, repoagent, pgml
    metadata JSONB,
    api_key_id UUID REFERENCES auth.api_keys(id),
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

---

## 🎯 STEP 3: Plugin Dashboards

### 3.1 APIKeys Dashboard

**Endpoints:**
```
GET    /api/dashboard/apikeys
POST   /api/dashboard/apikeys
GET    /api/dashboard/apikeys/:id
DELETE /api/dashboard/apikeys/:id
POST   /api/dashboard/apikeys/:id/rotate
```

**Features:**
- User Management
- API-Key Management (zentral)
- Role Management
- Permission Management
- Audit Log Viewer

### 3.2 Goose Dashboard

**Endpoints:**
```
POST   /api/dashboard/goose/run
POST   /api/dashboard/goose/stop
GET    /api/dashboard/goose/status
GET    /api/dashboard/goose/history
GET    /api/dashboard/goose/metrics
```

**Features:**
- Load Test Configuration
- Scenario Builder
- Live Status Monitoring
- Performance Metrics
- History & Reports

### 3.3 RepoAgent Dashboard

**Endpoints:**
```
POST   /api/dashboard/repoagent/analyze
GET    /api/dashboard/repoagent/status
GET    /api/dashboard/repoagent/files
GET    /api/dashboard/repoagent/metrics
POST   /api/dashboard/repoagent/generate-docs
```

**Features:**
- Repository Browser
- Code Analysis Results
- Dependency Graph
- Documentation Preview
- Metrics Visualization

### 3.4 PostgresML Dashboard

**Endpoints:**
```
POST   /api/dashboard/pgml/models
GET    /api/dashboard/pgml/models
POST   /api/dashboard/pgml/models/:id/train
GET    /api/dashboard/pgml/models/:id/status
POST   /api/dashboard/pgml/predict
```

**Features:**
- Model Management
- Training Progress
- Performance Metrics
- Embedding Visualization
- Data Explorer

### 3.5 BrainML Dashboard

**Endpoints:**
```
POST   /api/dashboard/brainml/documents
GET    /api/dashboard/brainml/documents
POST   /api/dashboard/brainml/search
POST   /api/dashboard/brainml/pipelines
GET    /api/dashboard/brainml/pipelines/:id/status
```

**Features:**
- Document Management
- Hybrid Search Interface
- Pipeline Builder
- API-Key Management (zentral)
- Results Visualization

---

## 🔌 API-Key Flow

```
1. Admin erstellt User in APIKeys Dashboard
2. Admin generiert API-Key für User
3. API-Key wird zentral in auth.api_keys gespeichert
4. Plugin-Name wird in api_keys.plugin_name gespeichert
5. Metadata wird in api_keys.metadata gespeichert
6. Key wird in BrainML für Zugriff gespeichert
7. Plugin nutzt Key für Authentifizierung
8. Audit Log wird geschrieben
```

---

## 🚀 Docker Compose Setup

```yaml
version: '3.8'

services:
  postgres:
    image: ghcr.io/postgresml/postgresml:2.7.12
    ports:
      - "5432:5432"
      - "8000:8000"
    environment:
      - POSTGRES_PASSWORD=pgml
      - POSTGRES_USER=pgml
      - POSTGRES_DB=bkg
    volumes:
      - postgres_data:/var/lib/postgresql
      - ./init-apikeys.sql:/docker-entrypoint-initdb.d/01-apikeys.sql
      - ./init-pgml.sql:/docker-entrypoint-initdb.d/02-pgml.sql

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
      - BKG_API_PORT=43119
      - BKG_PLUGIN_BUS_PORT=43121

volumes:
  postgres_data:
```

---

## ✅ Verification Checklist

- [ ] PostgreSQL running
- [ ] pgml extension loaded
- [ ] auth schema created
- [ ] brainml schema created
- [ ] Admin user created
- [ ] API-Key generated
- [ ] APIKeys Dashboard accessible
- [ ] Goose Dashboard accessible
- [ ] RepoAgent Dashboard accessible
- [ ] PostgresML Dashboard accessible
- [ ] BrainML Dashboard accessible

---

## 📚 Feature Documentation

- [APIKeys Features](../features/apikeys_feat.md)
- [Goose Features](../features/goose_feat.md)
- [RepoAgent Features](../features/repoagent_feat.md)
- [PostgresML Features](../features/pgml_feat.md)
- [BrainML Features](../features/brainml_feat.md)

---

**Status**: Ready for Implementation ✅
