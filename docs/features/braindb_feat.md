# 🗄️ BrainDB - Database & Retrieval Engine

**SQL, RAG, Vector Database Foundation**

---

## 📋 Konfiguration & Setup

- PostgreSQL Connection String
- Vector Store Configuration (pgvector)
- RAG Pipeline Definition
- Caching Configuration
- Environment Variables Support
- Connection Pooling

---

## 🔍 Hybrid Retrieval Features

- **Vector Search**: Semantic Similarity (Cosine, L2, Inner Product)
- **Keyword Search**: Full-Text Search
- **Hybrid Ranking**: Combined Scoring
- **Reranking**: LLM-based Reranking
- **Filtering**: Metadata Filtering
- **Faceted Search**: Category Filtering

---

## 📚 Document Management

- **Document Ingestion**: Multiple Formats (PDF, TXT, MD, JSON)
- **Chunking Strategies**: Semantic, Fixed-Size, Sliding Window
- **Metadata Extraction**: Auto-tagging
- **Document Versioning**: Version Control
- **Document Deletion**: Soft/Hard Delete
- **Bulk Operations**: Batch Import/Export

---

## 🔄 RAG Pipeline Orchestration

- **Workflow Definition**: YAML/JSON
- **Task Sequencing**: DAG-based
- **Error Handling**: Retry Logic
- **Conditional Execution**: If/Then/Else
- **Parallel Execution**: Multi-threading
- **Pipeline Monitoring**: Status Tracking

---

## 🔌 REST API Endpoints

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
POST   /api/braindb/apikeys          # API-Key speichern
GET    /api/braindb/apikeys          # API-Keys auflisten
DELETE /api/braindb/apikeys/:id      # API-Key löschen
```

---

## 🎨 Dashboard Features

- **Document Management Interface**
  - Document List with Search/Filter
  - Document Upload & Preview
  - Bulk Operations
  - Metadata Editor

- **Search Interface**
  - Vector Search
  - Keyword Search
  - Hybrid Search
  - Advanced Filters
  - Search Results Visualization

- **Pipeline Builder**
  - Visual DAG Editor
  - Task Configuration
  - Error Handling Setup
  - Pipeline Testing

- **Pipeline Execution Monitor**
  - Real-time Status
  - Progress Tracking
  - Error Logs
  - Performance Metrics

- **API-Key Management Panel**
  - Key List
  - Key Creation
  - Key Rotation
  - Usage Statistics

- **Data Explorer**
  - Table Browser
  - Query Builder
  - Data Visualization
  - Export Options

---

## 🔐 Security & Permissions

- JWT Authentication (via APIKeys Plugin)
- API-Key Management (zentral)
- Role-Based Access Control
- Data Encryption (At-Rest & In-Transit)
- Audit Logging
- Query Validation
- Rate Limiting

---

## 💾 Data Persistence

- **Document Storage**: PostgreSQL
- **Vector Storage**: pgvector
- **Metadata Storage**: JSONB
- **Pipeline Definitions**: Versioned
- **Execution History**: Complete Logs
- **API-Key Vault**: Encrypted

---

## 🚀 Performance & Scalability

- **Vector Index Optimization**: IVFFlat, HNSW
- **Query Caching**: Result Caching
- **Batch Processing**: Efficient Bulk Operations
- **Async Operations**: Non-blocking Requests
- **Connection Pooling**: Efficient Management
- **Distributed Retrieval**: Horizontal Scaling

---

## 📊 Monitoring & Telemetry

- **Search Performance Metrics**
- **Pipeline Execution Metrics**
- **API-Key Usage Tracking**
- **Error Logging**
- **WebSocket Live Updates**
- **Health Checks**

---

## 🔗 Plugin Bus Integration

- Capability Registration
- Event Publishing
- Log Streaming
- Telemetry Publishing
- Health Status
- Cross-Plugin Communication

---

## 🔄 Integration mit anderen Plugins

- **PostgresML (BrainML)**: Vector Generation & Storage
- **RepoMole**: Document Ingestion
- **MAID**: Test Data Management
- **APIKeys**: Key Storage & Management

---

**Status**: ✅ Ready for Implementation
