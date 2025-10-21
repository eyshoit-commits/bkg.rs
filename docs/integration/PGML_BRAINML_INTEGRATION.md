# 🧠 PostgresML + BrainML Integration Guide

## Überblick

**PostgresML (pgml)** ist eine ML/AI-Plattform, die direkt in PostgreSQL läuft. **BrainML** ist unser Hybrid-Search-Plugin. Diese Integration ermöglicht:

- ✅ **Vektorisierung** von Dokumenten direkt in der DB
- ✅ **Semantische Suche** mit pgml-Embeddings
- ✅ **ML-Modelle** ohne externe Services
- ✅ **Dashboard** für Experimente & Monitoring

---

## 📊 Architektur

```
┌─────────────────────────────────────────┐
│         BKG Control Center              │
│  (Angular Web UI - Port 43119)          │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│      NestJS API Gateway                 │
│  (core/backend/gateway - Port 43119)    │
└────────────────┬────────────────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
┌───▼──────────────┐  ┌──────▼──────────────┐
│   BrainML Plugin │  │  PostgresML Service │
│  (Rust - Hybrid) │  │  (Rust - ML/Vector) │
└───┬──────────────┘  └──────┬──────────────┘
    │                        │
    └────────────┬───────────┘
                 │
         ┌───────▼────────┐
         │  PostgreSQL DB │
         │  + pgml ext    │
         └────────────────┘
```

---

## 🚀 Setup & Installation

### 1. PostgreSQL + pgml Extension

**Docker Compose (devops/docker/docker-compose.yml):**

```yaml
version: '3.8'

services:
  # Bestehender BKG Container
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
      - ./init-pgml.sql:/docker-entrypoint-initdb.d/init-pgml.sql

volumes:
  postgres_data:
```

### 2. PostgreSQL Init Script

**devops/docker/init-pgml.sql:**

```sql
-- Enable pgml Extension
CREATE EXTENSION IF NOT EXISTS pgml;

-- Create BrainML Schema
CREATE SCHEMA IF NOT EXISTS brainml;

-- Documents Table (für Indexierung)
CREATE TABLE IF NOT EXISTS brainml.documents (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    embedding vector(384),  -- pgml embeddings (all-MiniLM-L6-v2)
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

-- Search Results Cache
CREATE TABLE IF NOT EXISTS brainml.search_results (
    id SERIAL PRIMARY KEY,
    query TEXT NOT NULL,
    results JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Grant Permissions
GRANT ALL PRIVILEGES ON SCHEMA brainml TO pgml;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA brainml TO pgml;
```

---

## 💻 BrainML Plugin Integration

### 1. Database Service (core/backend/gateway/src/storage/database.service.ts)

```typescript
import { Injectable } from '@nestjs/common';
import { Pool } from 'pg';

@Injectable()
export class DatabaseService {
  private pool: Pool;

  constructor() {
    this.pool = new Pool({
      connectionString: process.env.BKG_DATABASE_URL,
    });
  }

  async indexDocument(doc: {
    title: string;
    content: string;
    metadata?: Record<string, any>;
  }): Promise<void> {
    const query = `
      INSERT INTO brainml.documents (title, content, metadata, embedding)
      SELECT 
        $1,
        $2,
        $3,
        pgml.embed('all-MiniLM-L6-v2', $2)
      RETURNING id;
    `;
    
    await this.pool.query(query, [
      doc.title,
      doc.content,
      JSON.stringify(doc.metadata || {}),
    ]);
  }

  async semanticSearch(query: string, limit: number = 10): Promise<any[]> {
    const searchQuery = `
      SELECT 
        id,
        title,
        content,
        metadata,
        1 - (embedding <=> pgml.embed('all-MiniLM-L6-v2', $1)) AS similarity
      FROM brainml.documents
      ORDER BY similarity DESC
      LIMIT $2;
    `;
    
    const result = await this.pool.query(searchQuery, [query, limit]);
    return result.rows;
  }

  async trainClassifier(projectName: string, data: any[]): Promise<void> {
    const query = `
      SELECT * FROM pgml.train(
        $1,
        algorithm => 'xgboost',
        'classification',
        'brainml.training_data',
        'label'
      );
    `;
    
    await this.pool.query(query, [projectName]);
  }
}
```

### 2. BrainML Service (core/plugins/brainml/src/services/pgml.service.rs)

```rust
use sqlx::PgPool;
use serde_json::json;

pub struct PgmlService {
    pool: PgPool,
}

impl PgmlService {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn index_document(
        &self,
        title: &str,
        content: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO brainml.documents (title, content, metadata, embedding)
            SELECT 
              $1,
              $2,
              $3,
              pgml.embed('all-MiniLM-L6-v2', $2)
            RETURNING id
            "#,
        )
        .bind(title)
        .bind(content)
        .bind(metadata.unwrap_or(json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    pub async fn semantic_search(
        &self,
        query: &str,
        limit: i32,
    ) -> Result<Vec<SearchResult>, sqlx::Error> {
        let results: Vec<SearchResult> = sqlx::query_as(
            r#"
            SELECT 
              id,
              title,
              content,
              metadata,
              1 - (embedding <=> pgml.embed('all-MiniLM-L6-v2', $1)) AS similarity
            FROM brainml.documents
            ORDER BY similarity DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}

#[derive(sqlx::FromRow)]
pub struct SearchResult {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub similarity: f32,
}
```

---

## 🎨 Dashboard Integration

### 1. BrainML Dashboard Component

**core/frontend/admin-ui/src/app/features/plugins/brainml/brainml-dashboard.component.ts:**

```typescript
import { Component, OnInit } from '@angular/core';
import { ApiService } from '../../../services/api.service';

@Component({
  selector: 'app-brainml-dashboard',
  template: `
    <div class="brainml-dashboard">
      <h1>🧠 BrainML + PostgresML Dashboard</h1>
      
      <!-- Search Section -->
      <div class="search-section">
        <input 
          [(ngModel)]="searchQuery" 
          placeholder="Semantische Suche..."
          (keyup.enter)="search()"
        />
        <button (click)="search()">Suchen</button>
      </div>

      <!-- Results -->
      <div class="results" *ngIf="searchResults.length > 0">
        <div class="result-item" *ngFor="let result of searchResults">
          <h3>{{ result.title }}</h3>
          <p>{{ result.content | slice:0:200 }}...</p>
          <span class="similarity">Ähnlichkeit: {{ result.similarity | percent }}</span>
        </div>
      </div>

      <!-- pgml Dashboard Link -->
      <div class="pgml-link">
        <a href="http://localhost:8000" target="_blank">
          📊 PostgresML Dashboard öffnen
        </a>
      </div>
    </div>
  `,
  styles: [`
    .brainml-dashboard { padding: 20px; }
    .search-section { margin: 20px 0; }
    .result-item { 
      border: 1px solid #ddd; 
      padding: 10px; 
      margin: 10px 0;
      border-radius: 4px;
    }
    .similarity { color: #0066cc; font-weight: bold; }
    .pgml-link { margin-top: 30px; }
  `]
})
export class BrainmlDashboardComponent implements OnInit {
  searchQuery = '';
  searchResults: any[] = [];

  constructor(private api: ApiService) {}

  ngOnInit(): void {
    this.loadCollections();
  }

  async search(): Promise<void> {
    if (!this.searchQuery.trim()) return;
    
    try {
      this.searchResults = await this.api.semanticSearch(this.searchQuery).toPromise();
    } catch (error) {
      console.error('Search failed:', error);
    }
  }

  async loadCollections(): Promise<void> {
    try {
      const collections = await this.api.getCollections().toPromise();
      console.log('Collections:', collections);
    } catch (error) {
      console.error('Failed to load collections:', error);
    }
  }
}
```

### 2. API Endpoints (core/backend/gateway/src/brainml/brainml.controller.ts)

```typescript
import { Controller, Get, Post, Body, Query } from '@nestjs/common';
import { DatabaseService } from '../storage/database.service';

@Controller('/api/brainml')
export class BrainmlController {
  constructor(private db: DatabaseService) {}

  @Post('/index')
  async indexDocument(@Body() doc: any): Promise<any> {
    await this.db.indexDocument(doc);
    return { status: 'indexed' };
  }

  @Get('/search')
  async search(@Query('q') query: string, @Query('limit') limit: string): Promise<any[]> {
    return this.db.semanticSearch(query, parseInt(limit) || 10);
  }

  @Get('/collections')
  async getCollections(): Promise<any[]> {
    // Implementierung
    return [];
  }

  @Post('/train')
  async trainModel(@Body() config: any): Promise<any> {
    await this.db.trainClassifier(config.projectName, config.data);
    return { status: 'training_started' };
  }
}
```

---

## 📋 Checkliste für Setup

- [ ] PostgreSQL + pgml Extension in Docker hinzufügen
- [ ] `init-pgml.sql` erstellen & in Docker mounten
- [ ] `DatabaseService` mit pgml-Queries erweitern
- [ ] `BrainML` Rust-Service mit pgml integrieren
- [ ] API-Endpoints für Suche & Training hinzufügen
- [ ] Dashboard-Komponente implementieren
- [ ] pgml Dashboard (Port 8000) testen
- [ ] Umgebungsvariablen setzen (`BKG_DATABASE_URL`, `BKG_PGML_ENABLED`)

---

## 🔗 Ressourcen

- **pgml Docs:** https://postgresml.org/docs/
- **pgml GitHub:** https://github.com/postgresml/postgresml
- **pgml Dashboard:** http://localhost:8000 (nach Start)

---

## 🎯 Nächste Schritte

1. ✅ Docker Compose mit PostgreSQL + pgml aktualisieren
2. ✅ Init-SQL-Skript erstellen
3. ✅ BrainML Plugin mit pgml verbinden
4. ✅ API-Endpoints implementieren
5. ✅ Dashboard-Komponente bauen
6. ✅ Tests & Monitoring

