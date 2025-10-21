# bkg.rs - Folder Structure Overview

**Status**: COMPLETE  
**Datum**: 2025-10-21

---

## 📊 Visual Folder Tree

```
bkg.rs/
│
├── 📁 core/                          ⭐ MAIN APPLICATION
│   ├── 📁 backend/                   🦀 Rust Services
│   │   ├── gateway/                  REST API + WebSocket
│   │   ├── ml-engine/                ML Models & Inference
│   │   ├── auth/                     JWT + API Keys
│   │   ├── vector/                   Vector Database
│   │   └── shared/                   Shared Types
│   │
│   ├── 📁 frontend/                  🌐 Angular UI
│   │   └── admin-ui/                 Admin Dashboard
│   │       ├── src/app/
│   │       │   ├── core/             Services & Guards
│   │       │   ├── shared/           Components & Pipes
│   │       │   ├── features/         Plugin Dashboards
│   │       │   └── stores/           State Management
│   │       └── [config files]
│   │
│   ├── 📁 plugins/                   🔌 Plugin Implementations
│   │   ├── brainml/                  Indexing & Query
│   │   ├── candle/                   ML Models (Hugging Face)
│   │   ├── rustyface/                Face Recognition
│   │   ├── llmserver/                LLM Inference
│   │   ├── repoagent/                Code Analysis
│   │   ├── apikeys/                  Authentication
│   │   └── plugins.json              Plugin Registry
│   │
│   ├── 📁 database/                  💾 Data Layer
│   │   ├── migrations/               SQL Migration Files
│   │   ├── schema.sql                Full Schema
│   │   └── seeds/                    Initial Data
│   │
│   └── 📁 config/                    ⚙️ Configuration
│       ├── docker-compose.yml
│       ├── Cargo.workspace.toml
│       └── package.json
│
├── 📁 devops/                        🚀 Deployment & DevOps
│   ├── 📁 docker/                    🐳 Docker Setup
│   │   ├── Dockerfile                Multi-stage Build
│   │   ├── supervisord.conf          Process Management
│   │   └── start.sh                  Entrypoint
│   │
│   ├── 📁 .devcontainer/             💻 VSCode Setup
│   │   ├── devcontainer.json         Config
│   │   └── post-create.sh            Setup Script
│   │
│   ├── 📁 .github/workflows/         🔄 CI/CD Pipelines
│   │   ├── ci-cleanup.yml
│   │   ├── ci-build.yml
│   │   ├── ci-test.yml
│   │   └── cd-deploy.yml
│   │
│   └── 📁 scripts/                   🛠️ Utility Scripts
│       ├── cleanup-workspace.sh
│       ├── setup-dev.sh
│       ├── build-docker.sh
│       └── deploy.sh
│
├── 📁 docs/                          📚 Documentation
│   ├── README.md
│   ├── INDEX.md
│   ├── CODEX_SYSTEM_PROMPT.md
│   ├── CORE_STRUCTURE.md
│   ├── FOLDER_STRUCTURE_V0.2.md
│   ├── TECH_STACK.md
│   ├── 📁 architecture/
│   │   └── plugin_system_v0.2.md
│   ├── 📁 implementation/
│   │   └── angular_adminui_setup.md
│   ├── 📁 update/
│   │   ├── v0.1a.md
│   │   ├── v0.1a.json
│   │   └── v0.2-phase1.md
│   ├── 📁 changelog/
│   │   └── CHANGELOG.md
│   └── [more docs]
│
├── 📁 models/                        🤖 ML Models
│   ├── all-MiniLM-L6-v2-ggml-model-f16.gguf
│   ├── Qwen2-0.5B-Instruct-Q5_K_M.gguf
│   └── README.md
│
├── 📁 apps/                          ⚠️ LEGACY (v0.1a)
│   ├── bkg-api/                      NestJS Backend
│   └── bkg-web/                      Angular 17 Frontend
│
├── 📁 plugins/                       ⚠️ LEGACY (v0.1a)
│   ├── llmserver/
│   ├── repoagent/
│   └── apikeys/
│
├── 📁 docker/                        ⚠️ LEGACY (v0.1a)
│   ├── Dockerfile
│   ├── supervisord.conf
│   └── start.sh
│
├── 📄 README.md                      📖 Main Documentation
├── 📄 LICENSE
├── 📄 .gitignore
├── 📄 .gitattributes
│
├── 📄 APPROVAL.md                    ✅ Approval Checklist
├── 📄 APPROVAL_COMBINED.json         ✅ Approval Status
├── 📄 FINAL_SUMMARY.md               📊 Final Summary
├── 📄 V0.2_IMPLEMENTATION_GUIDE.md   📋 Implementation Guide
├── 📄 DOWNLOAD_GUIDE.md              📥 Download Instructions
├── 📄 PATCH_TROUBLESHOOTING.md       🔧 Troubleshooting
├── 📄 TECH_STACK.md                  🏗️ Tech Stack
├── 📄 FOLDER_STRUCTURE_V0.2.md       📁 Detailed Structure
├── 📄 CORE_STRUCTURE.md              🎯 Core Architecture
├── 📄 FOLDER_OVERVIEW.md             📊 This File
│
└── 📄 docker-compose.yml             🐳 Dev Environment
```

---

## 🎯 Folder Purpose Guide

### ⭐ core/ - Main Application

**backend/** - Rust Microservices
- `gateway/` - REST API + WebSocket Hub
- `ml-engine/` - ML Model Services
- `auth/` - Authentication & Authorization
- `vector/` - Vector Database
- `shared/` - Shared Types & Utilities

**frontend/** - Angular Admin UI
- `admin-ui/` - Main Dashboard Application
  - `src/app/core/` - Services, Guards, Models
  - `src/app/shared/` - Reusable Components
  - `src/app/features/` - Feature Modules (Plugins)
  - `src/app/stores/` - State Management

**plugins/** - Plugin Implementations
- 6 Plugins (Rust, Python, Node.js)
- `plugins.json` - Plugin Registry

**database/** - Data Layer
- `migrations/` - SQL Migration Files
- `schema.sql` - Full Database Schema
- `seeds/` - Initial Data

**config/** - Configuration Files
- `docker-compose.yml` - Development Setup
- `Cargo.workspace.toml` - Rust Workspace
- `package.json` - Node Configuration

---

### 🚀 devops/ - Deployment & DevOps

**docker/** - Docker Configuration
- `Dockerfile` - Multi-stage Build
- `supervisord.conf` - Process Management
- `start.sh` - Container Entrypoint

**.devcontainer/** - VSCode Development
- `devcontainer.json` - Container Config
- `post-create.sh` - Setup Script

**.github/workflows/** - CI/CD Pipelines
- `ci-cleanup.yml` - Workspace Cleanup
- `ci-build.yml` - Build Pipeline
- `ci-test.yml` - Test Pipeline
- `cd-deploy.yml` - Deployment

**scripts/** - Utility Scripts
- `cleanup-workspace.sh` - Workspace Cleanup
- `setup-dev.sh` - Development Setup
- `build-docker.sh` - Docker Build
- `deploy.sh` - Deployment Script

---

### 📚 docs/ - Documentation

**Root Level**
- `README.md` - Project Overview
- `INDEX.md` - Documentation Index
- `CODEX_SYSTEM_PROMPT.md` - Codex AI Prompt
- `CORE_STRUCTURE.md` - Core Architecture
- `TECH_STACK.md` - Technology Stack

**architecture/** - Architecture Documentation
- `plugin_system_v0.2.md` - Plugin System Design

**implementation/** - Implementation Guides
- `angular_adminui_setup.md` - Frontend Setup

**update/** - Version Updates
- `v0.1a.md` - v0.1a Refactor Plan
- `v0.1a.json` - v0.1a JSON Specification
- `v0.2-phase1.md` - v0.2 Phase 1 Plan

**changelog/** - Change History
- `CHANGELOG.md` - Version History

---

### 🤖 models/ - ML Models

- `all-MiniLM-L6-v2-ggml-model-f16.gguf` - Embedding Model
- `Qwen2-0.5B-Instruct-Q5_K_M.gguf` - Chat Model
- `README.md` - Model Documentation

---

### ⚠️ Legacy Folders (v0.1a)

**apps/** - Old Monolith Structure
- `bkg-api/` - NestJS Backend
- `bkg-web/` - Angular 17 Frontend

**plugins/** - Old Plugin Structure
- Flat structure (being refactored)

**docker/** - Old Docker Setup
- Will be moved to `devops/docker/`

---

## 📊 Size & Complexity

| Folder | Files | Purpose | Status |
|--------|-------|---------|--------|
| **core/backend/** | 50+ | Microservices | ✅ Ready |
| **core/frontend/** | 40+ | Admin UI | ✅ Ready |
| **core/plugins/** | 30+ | Plugin Impl | ✅ Ready |
| **core/database/** | 10+ | Schema | ✅ Ready |
| **devops/** | 15+ | DevOps | ✅ Ready |
| **docs/** | 20+ | Documentation | ✅ Complete |
| **apps/** | 60+ | Legacy | ⚠️ Deprecated |
| **plugins/** | 30+ | Legacy | ⚠️ Deprecated |

---

## 🔄 Migration Path

### Current State (v0.1a)
```
apps/bkg-api/
apps/bkg-web/
plugins/
docker/
```

### Target State (v0.2)
```
core/backend/
core/frontend/
core/plugins/
core/database/
devops/
```

### Migration Steps

1. **Prepare** - Create new structure
2. **Copy** - Migrate code to new locations
3. **Test** - Verify functionality
4. **Cleanup** - Remove old folders
5. **Release** - v0.2.0 Release

---

## 📈 Folder Statistics

```
Total Folders:     ~50
Total Files:       ~200
Total Lines:       ~50,000+
Documentation:     ~5,000 lines
Code:              ~45,000 lines
```

---

## 🎯 Quick Navigation

### For Developers
```
core/frontend/admin-ui/src/app/
core/backend/gateway/src/
core/plugins/
```

### For DevOps
```
devops/docker/
devops/.devcontainer/
devops/.github/workflows/
devops/scripts/
```

### For Documentation
```
docs/
CODEX_SYSTEM_PROMPT.md
TECH_STACK.md
CORE_STRUCTURE.md
```

### For Approval & Status
```
APPROVAL_COMBINED.json
FINAL_SUMMARY.md
V0.2_IMPLEMENTATION_GUIDE.md
```

---

## 🚀 Key Takeaways

✅ **Organized** - Clear separation of concerns  
✅ **Scalable** - Microservices architecture  
✅ **Documented** - Comprehensive documentation  
✅ **DevOps-Ready** - Docker, CI/CD, Devcontainer  
✅ **Migration-Friendly** - Legacy & new side-by-side  

---

**Status**: ✅ **FOLDER STRUCTURE COMPLETE**

_Visual overview of bkg.rs project structure_  
_v0.2 Redesign with Core Architecture_
