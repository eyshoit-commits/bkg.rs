# 🔐 APIKeys Plugin - Vollständige Feature-Liste

Zentrale Authentication & Authorization

---

## 📋 User Management

- **User Registration**: Sign-up Flow
- **User Profile**: Name, Email, Avatar
- **User Activation**: Email Verification
- **User Deactivation**: Soft Delete
- **User Deletion**: Hard Delete
- **User Search**: By Email, Username
- **Bulk Operations**: Import/Export Users

## 🔑 API-Key Management

- **Key Generation**: Secure Random Keys
- **Key Prefix**: Branded Prefixes (e.g., `bkg_live_`)
- **Key Rotation**: Automatic Rotation Schedule
- **Key Expiration**: Configurable TTL
- **Key Scopes**: Fine-grained Permissions
- **Key Rate Limiting**: Per-key Quotas
- **Key Audit**: Usage Tracking
- **Key Revocation**: Immediate Disable

## 👥 Role Management

- **Role Creation**: Custom Roles
- **Role Assignment**: User → Role Mapping
- **Role Hierarchy**: Parent/Child Roles
- **Role Cloning**: Template-based
- **Role Deletion**: With Reassignment
- **Bulk Role Assignment**: Batch Operations

## 🔒 Permission Management

- **Permission Definition**: Resource + Action
- **Permission Assignment**: Role → Permission
- **Permission Inheritance**: From Parent Roles
- **Permission Validation**: At Request Time
- **Permission Audit**: Logging

## 🔐 Authentication Methods

- **Password Auth**: Bcrypt Hashing
- **API Key Auth**: Bearer Token
- **JWT Tokens**: Stateless Sessions
- **OAuth2 Integration**: (Optional)
- **MFA Support**: TOTP, SMS
- **Session Management**: Expiration, Revocation

## 🔌 REST API Endpoints

```
# User Management
POST   /api/auth/register            # Neuer User
POST   /api/auth/login               # Login
POST   /api/auth/logout              # Logout
GET    /api/auth/me                  # Current User
PUT    /api/auth/me                  # Update Profile
POST   /api/auth/password            # Change Password
POST   /api/auth/mfa/enable          # MFA aktivieren
POST   /api/auth/mfa/verify          # MFA verifizieren

# API Keys
POST   /api/apikeys                  # Neuer API-Key
GET    /api/apikeys                  # Meine Keys
GET    /api/apikeys/:id              # Key Details
PUT    /api/apikeys/:id              # Update Key
DELETE /api/apikeys/:id              # Revoke Key
POST   /api/apikeys/:id/rotate       # Rotate Key

# Roles
GET    /api/roles                    # Alle Rollen
POST   /api/roles                    # Neue Rolle
PUT    /api/roles/:id                # Update Rolle
DELETE /api/roles/:id                # Delete Rolle

# Permissions
GET    /api/permissions              # Alle Permissions
POST   /api/permissions              # Neue Permission
DELETE /api/permissions/:id          # Delete Permission

# Users (Admin)
GET    /api/users                    # Alle Users
POST   /api/users                    # Neuer User
GET    /api/users/:id                # User Details
PUT    /api/users/:id                # Update User
DELETE /api/users/:id                # Delete User
POST   /api/users/:id/roles          # Assign Role
DELETE /api/users/:id/roles/:roleId  # Remove Role

# Audit
GET    /api/audit                    # Audit Logs
GET    /api/audit/users/:userId      # User Audit
```

## 🎨 Dashboard Features

- **User Management Panel**
  - User List with Search/Filter
  - User Details & Edit
  - User Creation Form
  - Bulk User Operations

- **API-Key Management Panel**
  - Key List with Metadata
  - Key Creation Wizard
  - Key Rotation Interface
  - Key Revocation Confirmation
  - Usage Statistics

- **Role Management Panel**
  - Role List & Hierarchy
  - Role Creation/Edit
  - Permission Assignment
  - Role Cloning
  - Bulk Role Operations

- **Permission Management Panel**
  - Permission List
  - Permission Creation
  - Resource/Action Mapping
  - Permission Audit

- **Audit Log Viewer**
  - Filterable Log List
  - Timeline View
  - Export Options
  - Search Functionality

## 🔐 Security Features

- **Password Security**
  - Bcrypt Hashing (10+ rounds)
  - Password Strength Validation
  - Password History
  - Forced Password Change

- **API Key Security**
  - Secure Random Generation
  - Hashed Storage (SHA256)
  - Rate Limiting
  - IP Whitelisting (Optional)
  - Rotation Enforcement

- **Session Security**
  - JWT Signing
  - Token Expiration
  - Refresh Token Rotation
  - Session Revocation

- **Audit & Logging**
  - All Auth Events Logged
  - Failed Attempt Tracking
  - Suspicious Activity Detection
  - Export Audit Logs

## 💾 Data Persistence

- **Users Table**: id, username, email, password_hash, etc.
- **API Keys Table**: id, user_id, key_hash, prefix, scopes, etc.
- **Roles Table**: id, name, description, permissions
- **Permissions Table**: id, resource, action, description
- **Sessions Table**: id, user_id, token_hash, expires_at
- **Audit Logs Table**: id, user_id, action, resource, timestamp

## 🚀 Performance & Scalability

- Connection Pooling
- Query Caching
- Index Optimization
- Batch Operations
- Async Processing

## 📊 Monitoring & Telemetry

- Login Success/Failure Rates
- API-Key Usage Metrics
- Session Duration Tracking
- Failed Auth Attempts
- Suspicious Activity Alerts

## 🔗 Plugin Bus Integration

- Capability Registration
- Event Publishing (User Created, Key Rotated, etc.)
- Log Streaming
- Telemetry Publishing
- Health Status

## 🔄 Integration mit anderen Plugins

- **BrainML**: Key Storage & Management
- **Goose**: Key Usage for Load Tests
- **RepoAgent**: Key Usage for Analysis
- **PostgresML**: Key Usage for ML Operations

---

**Status**: Ready for Implementation
