# 🚀 BKG v0.2 - Complete Implementation Guide

**All-in-One Documentation for BKG v0.2 with MAID, RepoMole, BrainDB, LLM Paradise, and APIKeys**

---

## 📋 Table of Contents

1. [Executive Summary](#executive-summary)
2. [Plugin Architecture](#plugin-architecture)
3. [APIKeys Plugin](#apikeys-plugin)
4. [BrainDB Plugin](#braindb-plugin)
5. [LLM Paradise Plugin](#llm-paradise-plugin)
6. [PostgresML (BrainML) Plugin](#postgresml-brainml-plugin)
7. [RepoMole Plugin](#repomole-plugin)
8. [MAID Plugin](#maid-plugin)
9. [Database Schema](#database-schema)
10. [API Endpoints](#api-endpoints)
11. [Dashboard Routes](#dashboard-routes)
12. [WebSocket Topics](#websocket-topics)
13. [Implementation Phases](#implementation-phases)
14. [Acceptance Criteria](#acceptance-criteria)

---

## Executive Summary

BKG v0.2 ist eine **vollständig modulare Plugin-Architektur** mit 6 Hauptkomponenten:

| Plugin | Funktion | Motto |
|--------|----------|-------|
| **APIKeys** | Zentrale Auth & Key-Verwaltung | Secure Everything |
| **BrainDB** | SQL/RAG/Vector Database | Data Foundation |
| **LLM Paradise** | ML/RAG/Vector Finding | AI Dreams Come True |
| **PostgresML** | ML Models & Embeddings | Move Models to DB |
| **RepoMole** | Code Analysis & Docs | Dig Deep into Code |
| **MAID** | Load Testing & Performance | Never Too Late |

---

## Plugin Architecture

### 🔐 APIKeys Plugin - Zentrale Authentication

**Kernfunktion**: Zentrale Verwaltung von Benutzern, API-Keys, Rollen und Permissions

#### Features
- User Registration, Profile Management, Activation
- API-Key Generation, Rotation, Expiration
- Role Management mit Hierarchie
- Permission Management (Resource + Action)
- Authentication Methods (Password, API Key, JWT, MFA)
- Session Management
- Audit Logging

#### API Endpoints
```
# User Management
POST   /api/auth/register            # Neuer User
POST   /api/auth/login               # Login
POST   /api/auth/logout              # Logout
GET    /api/auth/me                  # Current User
PUT    /api/auth/me                  # Update Profile
POST   /api/auth/password            # Change Password

# API Keys
POST   /api/apikeys                  # Neuer API-Key
GET    /api/apikeys                  # Meine Keys
GET    /api/apikeys/:id              # Key Details
PUT    /api/apikeys/:id              # Update Key
DELETE /api/apikeys/:id              # Revoke Key
POST   /api/apikeys/:id/rotate       # Rotate Key

# Roles & Permissions
GET    /api/roles                    # Alle Rollen
POST   /api/roles                    # Neue Rolle
GET    /api/permissions              # Alle Permissions

# Users (Admin)
GET    /api/users                    # Alle Users
POST   /api/users                    # Neuer User
GET    /api/users/:id                # User Details
DELETE /api/users/:id                # Delete User

# Audit
GET    /api/audit                    # Audit Logs
```

#### Dashboard
- User Management Panel
- API-Key Management Panel
- Role Management Panel
- Permission Management Panel
- Audit Log Viewer

---

### 🗄️ BrainDB Plugin - Database & Retrieval Engine

**Kernfunktion**: SQL, RAG, Vector Database Foundation

#### Features
- PostgreSQL Connection & Configuration
- Vector Store (pgvector)
- Hybrid Retrieval (Vector + Keyword Search)
- Document Management (Ingestion, Versioning, Deletion)
- RAG Pipeline Orchestration (DAG-based)
- Metadata Extraction & Filtering
- Caching & Performance Optimization

#### API Endpoints
```
POST   /api/braindb/documents        # Document hinzufügen
GET    /api/braindb/documents        # Documents auflisten
GET    /api/braindb/documents/:id    # Document Details
DELETE /api/braindb/documents/:id    # Document löschen
POST   /api/braindb/search           # Hybrid Search
POST   /api/braindb/pipelines        # Pipeline erstellen
GET    /api/braindb/pipelines        # Pipelines auflisten
POST   /api/braindb/pipelines/:id/run # Pipeline ausführen
GET    /api/braindb/pipelines/:id/status # Pipeline Status
```

#### Dashboard
- Document Management Interface
- Search Interface (Vector + Keyword)
- Pipeline Builder (Visual DAG Editor)
- Pipeline Execution Monitor
- Data Explorer
- Query Builder

---

### 🎉 LLM Paradise Plugin - ML/RAG/Vector Finding

**Kernfunktion**: Advanced AI/ML Integration for BrainDB

#### Features
- LLM Provider Configuration (OpenAI, Anthropic, Local Models)
- Chat Completion & Text Completion
- Embedding Generation (Multiple Models)
- Prompt Templates & Few-Shot Learning
- Chain-of-Thought Reasoning
- Vector Operations (Clustering, Dimensionality Reduction)
- RAG Features (Context Management, Multi-hop Reasoning)
- Tool Use & Function Calling

#### API Endpoints
```
POST   /api/llm-paradise/chat              # Chat with LLM
POST   /api/llm-paradise/complete          # Text Completion
POST   /api/llm-paradise/embed             # Generate Embeddings
POST   /api/llm-paradise/rag               # RAG Query
POST   /api/llm-paradise/prompts           # Prompt Management
GET    /api/llm-paradise/models            # Available Models
POST   /api/llm-paradise/models/:id/config # Model Configuration
GET    /api/llm-paradise/analytics         # Usage Analytics
```

#### Dashboard
- Chat Interface (Multi-turn Conversation)
- RAG Interface (Query + Retrieved Docs)
- Model Management (Selection, Parameter Tuning)
- Prompt Library (Templates, Versioning, Testing)
- Analytics Dashboard (Token Usage, Cost, Performance)

---

### 🧠 PostgresML (BrainML) Plugin - ML Models & Embeddings

**Kernfunktion**: Move models to the database, rather than constantly moving data to the models

#### Features
- **47+ ML Algorithms**: XGBoost, LightGBM, Random Forest, SVM, KNN, etc.
- **NLP Tasks**: Text Classification, Zero-Shot, Token Classification, Translation, Summarization, QA, Text Generation
- **Vector Database**: pgvector Extension, Semantic Search
- **LLM Fine-tuning**: On your own data
- **Performance**: 8-40X faster than HTTP microservices, Millions of TPS

#### SQL API
```sql
-- Training
SELECT * FROM pgml.train(
    'My Project',
    algorithm => 'xgboost',
    'classification',
    'my_table',
    'target_column'
);

-- Prediction
SELECT pgml.predict('My Project', ARRAY[0.1, 2.0, 5.0]) AS prediction;

-- Embeddings
SELECT pgml.embed('text-embedding-ada-002', 'Your text here') AS embedding;

-- Transform (NLP)
SELECT pgml.transform('text-classification', inputs => ARRAY['I love this!']) AS result;

-- Vector Search
SELECT * FROM documents
ORDER BY embedding <-> query_embedding
LIMIT 10;
```

#### Dashboard
- SQL Notebook (Write & Execute Queries)
- Model Management (Create, Train, Evaluate)
- Training Progress Visualization
- Performance Metrics (Accuracy, Loss, Validation)
- Embedding Visualization (t-SNE, UMAP)
- Data Explorer
- Experiment Tracking

---

### 🔍 RepoMole Plugin - Code Analysis & Documentation

**Kernfunktion**: LLM-Powered Repository-level Code Documentation Generation

#### Features
- **Automatic Git Change Detection**: Additions, Deletions, Modifications
- **AST-based Code Analysis**: Functions, Classes, Variables
- **Bidirectional Invocation Relationships**: Inter-object Dependencies
- **Multi-language Support**: Python (primary), Future: Java, C++, Go
- **Pre-commit Hooks**: Automatic documentation on git commit
- **Chat with Repo**: Q&A, Code Explanation
- **Scalability**: Tested on 270,000+ lines of code

#### CLI Commands
```bash
repomole run                          # Generate/update docs
repomole run --print-hierarchy        # Print repo structure
repomole clean                        # Remove cache
repomole diff                         # Preview changes
repomole chat-with-repo               # Start chat server

# Options
-m, --model TEXT                      # Model selection
-t, --temperature FLOAT               # Generation temperature
-tp, --target-repo-path PATH          # Repository path
-l, --language TEXT                   # Documentation language
```

#### API Endpoints
```
POST   /api/repomole/analyze         # Repository analysieren
GET    /api/repomole/status          # Status abfragen
GET    /api/repomole/files           # Datei-Liste
GET    /api/repomole/files/:id       # Datei-Details
POST   /api/repomole/generate-docs   # Docs generieren
GET    /api/repomole/metrics         # Code-Metriken
GET    /api/repomole/dependencies    # Dependencies
POST   /api/repomole/search          # Code durchsuchen
```

#### Dashboard
- Repository Browser (File tree navigation)
- Code Viewer (Syntax-highlighted)
- Documentation Preview (Markdown rendering)
- Hierarchy Visualization (Project structure)
- Change Timeline (Git history)
- Metrics Dashboard (Code statistics)
- Dependency Graph (Object relationships)
- Chat Interface (Q&A, Code explanation)

---

### 🚀 MAID Plugin - Load Testing & Performance

**Kernfunktion**: Load Testing & Performance Optimization (Goose renamed to MAID)

**Motto**: "MAID is never too late!"

#### Features
- **Configuration**: Ziel-Host, Users, Hatch Rate, Duration, Think Time
- **Lifecycle Control**: Start, Stop, Status, History
- **Monitoring**: Real-time Telemetry (CPU, Memory, RPS, Errors)
- **Dashboard**: Configuration, Live Stats, Logs, History
- **5 Komponenten**:
  1. **Extensions**: Custom Load Test Patterns
  2. **Recipe Generator**: LLM-based Recipe Generation
  3. **Prompt Library**: Zentrale Prompt-Verwaltung
  4. **Recipes**: Vordefinierte Szenarien
  5. **Deeplink Generator**: Shareable Test Links

#### API Endpoints
```
POST   /api/maid/run                 # Test starten
POST   /api/maid/stop                # Test stoppen
GET    /api/maid/status              # Status abfragen
GET    /api/maid/history             # Historie
GET    /api/maid/extensions          # Alle Extensions
POST   /api/maid/recipe-generator/generate # Recipe generieren
GET    /api/maid/prompts             # Prompt Library
GET    /api/maid/recipes             # Recipes auflisten
POST   /api/maid/deeplinks           # Deeplink generieren
```

#### Dashboard
- Load Test Configuration
- Scenario Builder
- Live Status Monitoring
- Performance Metrics
- History & Reports
- Extensions Management
- Recipe Generator Wizard
- Prompt Library Browser
- Deeplink Generator

---

## Database Schema

### auth.users
```sql
CREATE TABLE auth.users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    mfa_enabled BOOLEAN DEFAULT false,
    last_login TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### auth.api_keys
```sql
CREATE TABLE auth.api_keys (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    prefix VARCHAR(20),
    plugin_name VARCHAR(100),
    scopes TEXT[] DEFAULT ARRAY['read'],
    metadata JSONB,
    expires_at TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### braindb.documents
```sql
CREATE TABLE braindb.documents (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    embedding vector(384),
    plugin_source VARCHAR(100),
    metadata JSONB,
    api_key_id UUID REFERENCES auth.api_keys(id),
    created_at TIMESTAMP DEFAULT NOW()
);
```

### maid_runs
```sql
CREATE TABLE maid_runs (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    api_key_id UUID REFERENCES auth.api_keys(id),
    recipe_id VARCHAR(255),
    config_json JSONB NOT NULL,
    metrics_json JSONB,
    status VARCHAR(50),
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    total_requests INT,
    failed_requests INT,
    avg_response_time_ms FLOAT,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### repomole_analyses
```sql
CREATE TABLE repomole_analyses (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    repository_path VARCHAR(255) NOT NULL,
    status VARCHAR(50),
    total_files INT,
    analyzed_files INT,
    generated_docs INT,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);
```

---

## API Endpoints Summary

### Authentication
```
POST   /api/auth/register
POST   /api/auth/login
POST   /api/auth/logout
GET    /api/auth/me
```

### APIKeys
```
POST   /api/apikeys
GET    /api/apikeys
DELETE /api/apikeys/:id
```

### BrainDB
```
POST   /api/braindb/documents
GET    /api/braindb/documents
POST   /api/braindb/search
POST   /api/braindb/pipelines
```

### LLM Paradise
```
POST   /api/llm-paradise/chat
POST   /api/llm-paradise/embed
POST   /api/llm-paradise/rag
GET    /api/llm-paradise/models
```

### RepoMole
```
POST   /api/repomole/analyze
GET    /api/repomole/status
GET    /api/repomole/files
POST   /api/repomole/generate-docs
```

### MAID
```
POST   /api/maid/run
POST   /api/maid/stop
GET    /api/maid/status
GET    /api/maid/extensions
POST   /api/maid/recipe-generator/generate
```

---

## Dashboard Routes

```
/plugins/apikeys                      # APIKeys Dashboard
/plugins/braindb                      # BrainDB Dashboard
/plugins/llm-paradise                 # LLM Paradise Dashboard
/plugins/pgml                         # PostgresML Dashboard
/plugins/repomole                     # RepoMole Dashboard
/plugins/maid                         # MAID Dashboard

# Detailed Routes
/plugins/maid/run                     # Start New Test
/plugins/maid/results                 # Test Results
/plugins/maid/recipes                 # Recipe Library
/plugins/maid/extensions              # Extensions
/plugins/repomole/analyze             # Start Analysis
/plugins/repomole/chat                # Chat with Repo
/plugins/braindb/search               # Search Interface
/plugins/llm-paradise/chat            # Chat Interface
```

---

## WebSocket Topics

```
maid:status                           # Real-time Test Status
maid:metrics                          # Live Metrics
maid:logs                             # Live Logs

repomole:status                       # Analysis Status
repomole:progress                     # Analysis Progress
repomole:logs                         # Live Logs

braindb:search                        # Search Progress
llm-paradise:chat                     # Chat Streaming
llm-paradise:rag                      # RAG Results
```

---

## Implementation Phases

### Phase 1: Foundation (2 Wochen)
- [ ] APIKeys Plugin (Core Auth)
- [ ] BrainDB Plugin (Database)
- [ ] PostgresML Integration
- [ ] Admin Dashboard Scaffolding

### Phase 2: AI Integration (3 Wochen)
- [ ] LLM Paradise Plugin
- [ ] Chat Interface
- [ ] RAG Pipeline
- [ ] Model Management

### Phase 3: Analysis & Testing (2 Wochen)
- [ ] RepoMole Plugin
- [ ] MAID Plugin
- [ ] WebSocket Integration
- [ ] Real-time Monitoring

### Phase 4: Release (1 Woche)
- [ ] Documentation
- [ ] Testing & QA
- [ ] Deployment
- [ ] v0.2 Release

---

## Acceptance Criteria

- [x] All 6 Plugins documented
- [x] APIKeys zentrale Verwaltung
- [x] BrainDB + LLM Paradise split
- [x] RepoMole (RepoAgent renamed)
- [x] MAID (Goose renamed)
- [x] Database Schema complete
- [x] API Endpoints defined
- [x] Dashboard Routes defined
- [x] WebSocket Topics defined
- [x] Implementation Plan ready

---

## 🎯 Plugin Renamings

| Alt Name | Neu Name | Motto |
|----------|----------|-------|
| Goose | **MAID** | "MAID is never too late!" |
| RepoAgent | **RepoMole** | "Dig Deep into Your Code!" |
| BrainML | **BrainDB** | "Data Foundation" |
| - | **LLM Paradise** | "AI Dreams Come True!" |

---

## 📚 References

- [MAID Implementation Prompt](./MAID_IMPLEMENTATION_PROMPT.md)
- [RepoMole Implementation Prompt](./REPOMOLE_IMPLEMENTATION_PROMPT.md)
- [PostgresML Documentation](https://postgresml.org)
- [Goose Documentation](https://block.github.io/goose)
- [RepoAgent GitHub](https://github.com/OpenBMB/RepoAgent)

---

**Status**: ✅ BKG v0.2 Complete Guide READY for Implementation

**Last Updated**: 2025-10-21

**Version**: v0.2.0

---

*"Build it right, build it fast, build it together!"* 🚀
