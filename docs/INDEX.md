# bkg.rs Dokumentation - Index

**Projekt**: bkg.rs - Modulares Plug-in-Framework für KI-Systeme  
**Version**: 0.2 (Planning)  
**Datum**: 2025-10-21

---

## 📚 Dokumentations-Übersicht

### 🚀 Schnelleinstieg

| Dokument | Zweck | Zielgruppe |
|----------|-------|-----------|
| **[../QUICK_START.md](../QUICK_START.md)** | 5-Minuten Schnellstart | Alle |
| **[../DEV_SETUP.md](../DEV_SETUP.md)** | Lokale Entwicklung | Entwickler |
| **[../DOCKER.md](../DOCKER.md)** | Docker Deployment | DevOps/Ops |
| **[../STATUS.md](../STATUS.md)** | Aktueller Status | Alle |

### 📋 Planung & Roadmap

| Dokument | Inhalt | Status |
|----------|--------|--------|
| **[next.md](next.md)** | Nächste Phase (v0.2) mit Zielen, Architektur, Roadmap | 📋 Planning |
| **[update/v0.1a.md](update/v0.1a.md)** | Refactor-Plan: Candle, RustyFace, Admin-Dashboard | 📋 Design |

### 🏗️ Architektur & Design

| Dokument | Fokus | Leser |
|----------|-------|-------|
| **[architecture/plugin_system_v0.2.md](architecture/plugin_system_v0.2.md)** | Detaillierte Plugin-Architektur mit Code-Beispielen | Architekten/Entwickler |
| **[architecture.md](architecture.md)** | Allgemeine Architektur-Übersicht | Alle |
| **[plugins.md](plugins.md)** | Plugin-Dokumentation | Plugin-Entwickler |

### 🔧 Betrieb & Deployment

| Dokument | Zweck | Zielgruppe |
|----------|-------|-----------|
| **[deployment.md](deployment.md)** | Deployment-Strategien | DevOps |
| **[operations.md](operations.md)** | Operationen & Monitoring | Ops-Team |

### 📖 Weitere Ressourcen

| Dokument | Beschreibung |
|----------|-------------|
| **[README.md](README.md)** | Dokumentations-Übersicht |

---

## 🎯 Nach Rolle

### 👨‍💻 Entwickler

1. **Einstieg**: [../QUICK_START.md](../QUICK_START.md)
2. **Setup**: [../DEV_SETUP.md](../DEV_SETUP.md)
3. **Architektur**: [architecture/plugin_system_v0.2.md](architecture/plugin_system_v0.2.md)
4. **Plugin-Entwicklung**: [plugins.md](plugins.md)
5. **Roadmap**: [next.md](next.md)

### 🏗️ Architekten

1. **Übersicht**: [architecture.md](architecture.md)
2. **Detailliert**: [architecture/plugin_system_v0.2.md](architecture/plugin_system_v0.2.md)
3. **Roadmap**: [next.md](next.md)
4. **Deployment**: [deployment.md](deployment.md)

### 🚀 DevOps/Ops

1. **Schnellstart**: [../DOCKER.md](../DOCKER.md)
2. **Deployment**: [deployment.md](deployment.md)
3. **Operations**: [operations.md](operations.md)
4. **Status**: [../STATUS.md](../STATUS.md)

### 📊 Projektmanagement

1. **Status**: [../STATUS.md](../STATUS.md)
2. **Roadmap**: [next.md](next.md)
3. **Refactor-Plan**: [update/v0.1a.md](update/v0.1a.md)

---

## 📊 Projekt-Timeline

```
v0.1a (Aktuell - 64%)
├── ✅ BrainML Integration
├── ✅ Adapter-Layer
├── ✅ Dokumentation
└── ⚠️ Proxy-Blocker

v0.2 (Planning - 4-6 Wochen)
├── Phase 1: Foundation (1-2 Wochen)
│   ├── Proxy-Fix
│   ├── Plugin-Registry
│   └── Candle-Plugin
├── Phase 2: Integration (2-3 Wochen)
│   ├── RustyFace-Plugin
│   ├── Admin-Dashboard
│   └── WebSocket-Integration
└── Phase 3: Polish (1-2 Wochen)
    ├── Telemetrie
    ├── Tests
    └── Release

v0.3 (Future)
├── Multi-Node Deployment
├── Kubernetes Support
└── Advanced Telemetry
```

---

## 🔑 Wichtige Konzepte

### Plugin-System

- **Plugin-Registry**: Zentrale Verwaltung aller Plug-ins
- **Plugin-Bus**: RPC-basierte Kommunikation zwischen Plug-ins
- **Hot-Swap**: Plug-ins zur Laufzeit laden/entladen
- **Isolation**: Jedes Plug-in in eigenem Prozess

### Technologie-Stack

**Backend:**
- Rust (Core, Plug-ins)
- TypeScript/NestJS (API)
- SQLite (Datenbank)

**Frontend:**
- Angular 17 (Admin-UI)
- Tailwind CSS (Styling)
- RxJS/Signals (State Management)

**Deployment:**
- Docker Compose (Entwicklung)
- Docker (Production)
- Kubernetes (optional)

---

## 📝 Dokumentations-Standards

### Struktur

Alle Dokumente folgen dieser Struktur:

```markdown
# Titel

**Datum**: YYYY-MM-DD  
**Status**: Draft/Review/Published  
**Autor**: Name

## Übersicht
## Inhalte
## Nächste Schritte
```

### Konventionen

- **Fett**: Wichtige Begriffe, Dateinamen
- **Code-Blöcke**: Rust, TypeScript, JSON, YAML
- **Tabellen**: Vergleiche, Übersichten
- **Emojis**: Status-Indikatoren (✅, ⚠️, 📋, etc.)

---

## 🔗 Externe Ressourcen

- **GitHub**: https://github.com/eyshoit-commits/bkg.rs
- **Rust Docs**: https://doc.rust-lang.org/
- **Angular Docs**: https://angular.io/docs
- **NestJS Docs**: https://docs.nestjs.com/

---

## ❓ FAQ

### Wo finde ich Informationen zu...?

**...lokaler Entwicklung?**  
→ [../DEV_SETUP.md](../DEV_SETUP.md)

**...Docker Deployment?**  
→ [../DOCKER.md](../DOCKER.md)

**...Plugin-Entwicklung?**  
→ [plugins.md](plugins.md) + [architecture/plugin_system_v0.2.md](architecture/plugin_system_v0.2.md)

**...Roadmap & Planung?**  
→ [next.md](next.md)

**...Architektur-Details?**  
→ [architecture/plugin_system_v0.2.md](architecture/plugin_system_v0.2.md)

**...Deployment-Strategien?**  
→ [deployment.md](deployment.md)

---

## 📞 Support & Kontakt

- **Issues**: https://github.com/eyshoit-commits/bkg.rs/issues
- **Discussions**: https://github.com/eyshoit-commits/bkg.rs/discussions
- **Autor**: lofmas

---

**Zuletzt aktualisiert**: 2025-10-21  
**Nächste Überprüfung**: 2025-10-28
