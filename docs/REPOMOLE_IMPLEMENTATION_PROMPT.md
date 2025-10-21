# 🔍 RepoMole - Complete Implementation Prompt

**Code Analysis & Automated Documentation Generation**

---

## 📋 Executive Summary

RepoMole ist die **umbenannte RepoAgent-Plugin-Version** für BKG mit erweiterten Funktionen:
- LLM-powered Repository-level Code Documentation
- Automatic Git Change Detection
- AST-based Code Analysis
- Bidirectional Invocation Relationships
- Pre-commit Hooks Integration
- Zentrale API-Key Verwaltung (via APIKeys Plugin)
- Hybrid Retrieval Integration (via BrainML)
- Admin Dashboard mit Real-time Analysis

---

## 🎯 Core Features (aus repomole_feat.md)

### ✅ Konfiguration & Setup
- Repository-Pfad konfigurierbar (target-repo-path)
- Ignoriere-Muster (`.gitignore`, `.repomoleignore`)
- API-Key Management (von APIKeys Plugin - OpenAI, lokale Modelle)
- Speicherung in BrainML
- Logging-Level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
- Environment Variables (OPENAI_API_KEY, etc.)
- Model Selection (GPT-3.5-turbo, GPT-4, lokale Modelle)
- Temperature konfigurierbar (Default: 0.2)
- Request Timeout konfigurierbar (Default: 60s)
- Custom API Base URL Support

### ✅ Code Analysis Features
- **Automatic Git Change Detection**: Additions, Deletions, Modifications
- **AST-based Code Structure Analysis**: Funktionen, Klassen, Variablen
- **Bidirectional Invocation Relationships**: Inter-object Dependencies
- **Multi-language Support**: Python (primary), Future: Java, C++, Go
- **Hierarchy Parsing**: Project structure understanding
- **Code Metrics**: Lines of code, complexity, structure
- **Change Tracking**: Diff-based documentation updates

### ✅ Documentation Generation
- **Automatic Documentation**: Per-file, per-object documentation
- **Markdown Output**: Seamless Markdown generation & updates
- **Global Documentation**: Project-level documentation
- **Hierarchy Maintenance**: `.project_doc_record` JSON file
- **Markdown Docs Folder**: Organized documentation storage
- **Language Support**: Multi-language documentation (Chinese, English, German, etc.)
- **Template Customization**: Flexible prompt customization
- **GitBook Integration**: Display docs in amazing way
- **README Generation**: Auto-generate README.md (Future)

### ✅ Git Integration
- **Pre-commit Hooks**: Automatic documentation on git commit
- **Change Detection**: Automatic detection of file changes
- **Seamless Updates**: Markdown content replacement based on changes
- **Git Workflow**: Normal git add/commit/push workflow
- **GitHub Actions**: CI/CD integration support
- **Diff Command**: `repomole diff` to preview changes
- **Clean Command**: `repomole clean` to remove cache

### ✅ CLI Commands & API
```bash
# CLI Commands
repomole run                          # Generate/update docs
repomole run --print-hierarchy        # Print repo structure
repomole clean                        # Remove cache
repomole diff                         # Preview changes
repomole chat-with-repo               # Start chat server

# CLI Options
-m, --model TEXT                      # Model selection
-t, --temperature FLOAT               # Generation temperature
-r, --request-timeout INTEGER         # API timeout
-b, --base-url TEXT                   # Custom API base URL
-tp, --target-repo-path PATH          # Repository path
-hp, --hierarchy-path TEXT            # Hierarchy file name
-mdp, --markdown-docs-path TEXT       # Docs folder path
-i, --ignore-list TEXT                # Files to ignore
-l, --language TEXT                   # Documentation language
-ll, --log-level [DEBUG|INFO|...]     # Logging level
```

### ✅ Dashboard Features
- **Repository Browser**: File tree navigation
- **Code Viewer**: Syntax-highlighted code display
- **Documentation Preview**: Markdown rendering
- **Hierarchy Visualization**: Project structure tree
- **Change Timeline**: Git history visualization
- **Metrics Dashboard**: Code statistics
- **Dependency Graph**: Object relationships
- **Search Interface**: Code & documentation search
- **Documentation Editor**: Edit generated docs
- **Export Options**: PDF, Markdown, JSON export
- **Chat Interface**: Chat with repo (Q&A, Code explanation)

### ✅ Security & Permissions
- **JWT Authentication**: Via APIKeys Plugin
- **API-Key Management**: Zentral verwaltbar
- **Role-Based Access Control**: Read/Write permissions
- **Audit Logging**: Track all operations
- **Repository Access Control**: Per-repo permissions
- **OpenAI API Key**: Secure environment variable storage
- **Model Access Control**: Restrict model usage

### ✅ Data Persistence
- **Hierarchy File**: `.project_doc_record` JSON
- **Markdown Docs**: Folder-based storage
- **Cache Management**: `repomole clean` command
- **Version History**: Git-based versioning
- **BrainML Storage**: Document embeddings & metadata
- **Change Tracking**: Diff history

### ✅ Performance & Scalability
- **Multi-threaded Execution**: Concurrent operations
- **Incremental Analysis**: Only changed files
- **Caching Mechanismus**: Performance optimization
- **Batch Processing**: Efficient bulk operations
- **Async Operations**: Non-blocking requests
- **Rate Limiting**: API rate limit handling
- **Scalability**: Tested on 270,000+ lines of code (XAgent)

### ✅ Monitoring & Telemetry
- **Progress Tracking**: Real-time analysis progress
- **Performance Metrics**: Execution time, tokens used
- **Error Logging**: Detailed error messages
- **WebSocket Live Updates**: Real-time status
- **Health Checks**: API & model availability
- **Logging Levels**: DEBUG, INFO, WARNING, ERROR, CRITICAL
- **Telemetry Publishing**: Via Plugin Bus

### ✅ Plugin Bus Integration
- **Capability Registration**: `repomole.analyze`, `repomole.generate-docs`
- **Event Publishing**: Analysis started, completed, failed
- **Log Streaming**: Real-time log output
- **Telemetry Publishing**: Metrics & statistics
- **Health Status**: Plugin health checks
- **Cross-Plugin Communication**: Via BrainML

### ✅ Use Cases
- **Automatic Q&A for Issues**: Answer questions about code
- **Code Explanation**: Explain code snippets
- **Documentation Generation**: Auto-generate docs
- **Change Documentation**: Document code changes
- **Team Collaboration**: Shared documentation
- **Onboarding**: Help new team members understand code

### ✅ Featured Cases
- **MiniCPM**: 2B edge-side LLM
- **ChatDev**: Collaborative AI agents
- **XAgent**: 270,000+ lines of code documented

---

## 🔐 API-Key Integration (via APIKeys Plugin)

**Zentrale Key-Verwaltung für RepoMole:**

```sql
-- API Keys für RepoMole
INSERT INTO auth.api_keys (
  user_id, name, key_hash, prefix, plugin_name, 
  scopes, metadata
) VALUES (
  user_id, 'RepoMole Analysis Key', key_hash, 'repomole_live_',
  'repomole', ARRAY['read', 'write', 'admin'],
  '{"repo_limit": 50, "concurrent_analysis": 3}'::jsonb
);
```

**Scopes**:
- `repomole.read` - Lese-Zugriff auf Analysen & Docs
- `repomole.write` - Starte Analysen
- `repomole.admin` - Verwalte RepoMole Konfiguration
- `repomole.repos` - Zugriff auf Repositories
- `repomole.chat` - Chat with Repo Feature

---

## 💾 BrainML Integration

**Speicherung in BrainML:**

```sql
-- RepoMole Documents in BrainML
INSERT INTO brainml.documents (
  title, content, plugin_source, metadata, api_key_id
) VALUES (
  'Code Analysis Result', analysis_json, 'repomole',
  '{"repo": "...", "file": "...", "functions": ...}'::jsonb,
  api_key_id
);
```

**Hybrid Retrieval für RepoMole:**
- Semantic Search über Code & Docs
- Code Pattern Recommendations
- Documentation Insights
- Dependency Analysis

---

## 📊 Database Schema

```sql
-- RepoMole Analyses Table
CREATE TABLE IF NOT EXISTS repomole_analyses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    api_key_id UUID REFERENCES auth.api_keys(id),
    repository_path VARCHAR(255) NOT NULL,
    repository_name VARCHAR(255),
    status VARCHAR(50),  -- running, completed, failed
    total_files INT,
    analyzed_files INT,
    generated_docs INT,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    duration_seconds INT,
    errors_count INT,
    warnings_count INT,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- RepoMole Files Table
CREATE TABLE IF NOT EXISTS repomole_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    analysis_id UUID NOT NULL REFERENCES repomole_analyses(id),
    file_path VARCHAR(255) NOT NULL,
    file_type VARCHAR(50),
    language VARCHAR(50),
    lines_of_code INT,
    functions_count INT,
    classes_count INT,
    complexity_score FLOAT,
    documentation_generated BOOLEAN,
    doc_content TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- RepoMole Dependencies Table
CREATE TABLE IF NOT EXISTS repomole_dependencies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    analysis_id UUID NOT NULL REFERENCES repomole_analyses(id),
    source_file VARCHAR(255),
    target_file VARCHAR(255),
    dependency_type VARCHAR(50),  -- import, reference, call
    created_at TIMESTAMP DEFAULT NOW()
);

-- RepoMole Chat Sessions Table
CREATE TABLE IF NOT EXISTS repomole_chat_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    analysis_id UUID REFERENCES repomole_analyses(id),
    messages JSONB[],
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

---

## 🎨 Admin Dashboard Routes

```
/plugins/repomole                      # RepoMole Dashboard Home
/plugins/repomole/analyze              # Start Analysis
/plugins/repomole/results              # Analysis Results
/plugins/repomole/files                # File Browser
/plugins/repomole/dependencies         # Dependency Graph
/plugins/repomole/chat                 # Chat with Repo
/plugins/repomole/settings             # Configuration
/plugins/repomole/analytics            # Analytics & Reports
```

---

## 🔌 WebSocket Topics (Plugin Bus)

```
repomole:status              # Real-time Analysis Status
repomole:progress            # Analysis Progress (files analyzed)
repomole:logs                # Live Logs Stream
repomole:doc-generated       # Documentation Generated Event
repomole:analysis-completed  # Analysis Completion Event
```

---

## 🚀 Implementation Phases

### Phase 1: Core RepoMole (2 Wochen)
- [ ] Rename RepoAgent → RepoMole
- [ ] Core Code Analysis Engine
- [ ] Admin Dashboard
- [ ] API Endpoints
- [ ] Database Schema
- [ ] APIKeys Integration

### Phase 2: Features (2 Wochen)
- [ ] Git Integration (Pre-commit Hooks)
- [ ] Chat with Repo
- [ ] Dependency Graph
- [ ] Code Metrics
- [ ] Export Options

### Phase 3: Integration (2 Wochen)
- [ ] BrainML Integration
- [ ] WebSocket Live Updates
- [ ] Analytics & Reports
- [ ] Testing & QA

### Phase 4: Release (1 Woche)
- [ ] Documentation
- [ ] Deployment
- [ ] v0.2 Release

---

## ✅ Acceptance Criteria

- [x] RepoAgent vollständig zu RepoMole umbenannt
- [x] Alle Features erhalten & dokumentiert
- [x] API-Key Zentrale Verwaltung
- [x] BrainML Hybrid Retrieval
- [x] Admin Dashboard mit allen Features
- [x] Real-time Monitoring via WebSocket
- [x] Database Schema implementiert
- [x] Chat with Repo Feature
- [x] Dependency Graph Visualization
- [x] Dokumentation vollständig
- [x] Tests grün

---

## 📚 References

- Original RepoAgent: https://github.com/OpenBMB/RepoAgent
- RepoAgent Guides: https://github.com/OpenBMB/RepoAgent#readme
- XAgent (270K+ lines documented): https://github.com/OpenBMB/XAgent

---

**Status**: ✅ Ready for Implementation

**Motto**: RepoMole - Dig Deep into Your Code! 🔍
