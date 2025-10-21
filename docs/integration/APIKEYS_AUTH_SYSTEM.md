# 🔐 APIKeys Plugin - Authentication & Authorization System

## Überblick

Das **APIKeys Plugin** verwaltet:
- ✅ **Benutzer & Authentifizierung** (Login, Passwörter, MFA)
- ✅ **API-Keys** (Generierung, Rotation, Revocation)
- ✅ **Rollen & Permissions** (RBAC - Role-Based Access Control)
- ✅ **Audit Logs** (Wer hat was wann getan)
- ✅ **Session Management** (JWT Tokens)

---

## 📊 Datenbankschema

### PostgreSQL Init Script

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
CREATE INDEX idx_users_username ON auth.users(username);
CREATE INDEX idx_users_email ON auth.users(email);
CREATE INDEX idx_api_keys_user_id ON auth.api_keys(user_id);
CREATE INDEX idx_api_keys_key_hash ON auth.api_keys(key_hash);
CREATE INDEX idx_sessions_user_id ON auth.sessions(user_id);
CREATE INDEX idx_audit_logs_user_id ON auth.audit_logs(user_id);

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
GRANT ALL PRIVILEGES ON SCHEMA auth TO postgres;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth TO postgres;
```

---

## 💻 APIKeys Service (Node.js)

**core/plugins/apikeys/src/auth.service.js** - Siehe separate Datei

Funktionalität:
- ✅ User Management (Create, Authenticate, Roles)
- ✅ API Key Management (Create, Validate, Revoke)
- ✅ Permission Checking (RBAC)
- ✅ Session Management (JWT)
- ✅ Audit Logging

---

## 🎨 Dashboard

**core/frontend/admin-ui/src/app/features/plugins/apikeys/apikeys-dashboard.component.ts**

Features:
- ✅ **API-Keys Tab**: Erstellen, Anzeigen, Widerrufen
- ✅ **Users Tab**: Benutzer verwalten, Rollen zuweisen
- ✅ **Roles Tab**: Rollen & Permissions anzeigen
- ✅ **Audit Tab**: Alle Aktionen protokollieren

---

## 🔗 API Endpoints

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

## 📋 Checkliste

- [ ] PostgreSQL Init-Script erstellen
- [ ] Auth-Schema & Tabellen in DB
- [ ] APIKeys Service implementieren
- [ ] Dashboard Component bauen
- [ ] API Endpoints hinzufügen
- [ ] JWT/Session Management
- [ ] Audit Logging
- [ ] Tests schreiben

---

## 🎯 Nächste Schritte

1. ✅ Docker Compose mit PostgreSQL aktualisieren
2. ✅ Init-Scripts für brainml + apikeys
3. ✅ Services implementieren
4. ✅ Dashboards bauen
5. ✅ Testen & Deployen

