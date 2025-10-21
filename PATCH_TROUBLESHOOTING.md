# Patch-Fehler Troubleshooting Guide

**Problem**: "Failed to apply patch to repo /workspace/bkg.rs"

---

## 🔍 Ursachen

Der Fehler tritt auf, wenn **zwei Prozesse gleichzeitig** Dateien im Repo ändern:

| Prozess 1 | Prozess 2 | Konflikt |
|-----------|-----------|----------|
| Setup-Skript | Git Sync | package-lock.json |
| Autoformat | Build-Agent | Cargo.lock |
| Prebuild | Editor | README.md, docs/* |
| CI Pipeline | Devcontainer | workspace.json |

---

## 🚑 Schnelle Lösung (5 Minuten)

### Lokal (VSCode, Terminal)

```bash
# 1. Änderungen sichern
git diff > /tmp/patch_backup.diff

# 2. Workspace clearen
git restore --staged .
git restore .
git clean -fd

# 3. Lockfiles regenerieren
cargo update --dry-run
npm install --package-lock-only

# 4. Status prüfen
git status

# 5. Neu starten
npm start
```

### Automatisiert (unser Script)

```bash
./scripts/cleanup-workspace.sh
```

---

## 🔧 Umgebungs-spezifische Lösungen

### VSCode Devcontainer

**Problem**: `updateContentCommand` läuft parallel zu Edits

**Lösung**:
```json
{
  "updateContentCommand": "false",
  "postCreateCommand": "bash .devcontainer/post-create.sh"
}
```

**Warum**: 
- `updateContentCommand: false` → Keine automatischen Patches
- `postCreateCommand` → Nur einmalig beim Container-Start

### GitHub Actions CI

**Problem**: Mehrere Jobs modifizieren Lockfiles

**Lösung**:
```yaml
- name: Clean Workspace
  run: |
    git restore --staged .
    git restore .
    git clean -fd

- name: Regenerate Lockfiles
  run: |
    cargo update --dry-run
    npm install --package-lock-only
```

**Workflow**: `.github/workflows/ci-cleanup.yml`

### Docker Multi-Stage Build

**Problem**: Setup-Skript und Build-Agent schreiben gleichzeitig

**Lösung**:
```dockerfile
# Stage 1: Setup (keine Änderungen)
FROM rust:latest as setup
RUN cargo --version
RUN npm --version

# Stage 2: Build (nur COPY, kein Setup)
FROM setup as builder
COPY . /app
WORKDIR /app
RUN cargo build --release
```

### Replit / GitHub Codespaces

**Problem**: Automatische Syncs + Setup-Skripte

**Lösung**:
1. Devcontainer verwenden (`.devcontainer/devcontainer.json`)
2. `postCreateCommand` statt `updateContentCommand`
3. Lockfiles in `.gitignore` NICHT eintragen (sie gehören ins Repo!)

---

## 📋 Checkliste: Patch-Fehler beheben

- [ ] **Schritt 1**: Änderungen sichern
  ```bash
  git diff > /tmp/patch.diff
  ```

- [ ] **Schritt 2**: Workspace clearen
  ```bash
  git restore --staged .
  git restore .
  git clean -fd
  ```

- [ ] **Schritt 3**: Lockfiles regenerieren
  ```bash
  cargo update --dry-run
  npm install --package-lock-only
  cd frontend/admin-ui && npm install --package-lock-only
  ```

- [ ] **Schritt 4**: Status prüfen
  ```bash
  git status
  ```

- [ ] **Schritt 5**: Neu starten
  ```bash
  npm start
  # oder
  cargo run -p gateway
  ```

---

## 🛡️ Prävention: Langfristige Fixes

### ✅ Tipp 1: Lockfiles nicht im Setup-Skript ändern

**❌ FALSCH:**
```bash
#!/bin/bash
npm install  # ← Ändert package-lock.json!
cargo build  # ← Ändert Cargo.lock!
```

**✅ RICHTIG:**
```bash
#!/bin/bash
npm config set registry https://registry.npmjs.org/
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "crates-io-mirror"
...
EOF
# Keine Dateiänderungen!
```

### ✅ Tipp 2: Getrennte Verzeichnisse für Generatoren

Wenn du Docs/Code generierst (z.B. Codex AI):

**❌ FALSCH:**
```
docs/
├── next.md
├── architecture/
└── generated_architecture.md  ← Konflikt!
```

**✅ RICHTIG:**
```
docs/
├── next.md
├── architecture/
└── _generated/
    └── architecture.md  ← Separates Verzeichnis
```

### ✅ Tipp 3: CI-Pipeline mit Cleanup-Job

```yaml
jobs:
  cleanup:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: git restore --staged . && git restore . && git clean -fd

  build:
    needs: cleanup
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm install
```

### ✅ Tipp 4: Devcontainer mit `updateContentCommand: false`

```json
{
  "updateContentCommand": "false",
  "postCreateCommand": "bash .devcontainer/post-create.sh"
}
```

---

## 🎯 Für bkg.rs v0.2 spezifisch

### Setup-Strategie

```
1. Devcontainer startet
   ↓
2. postCreateCommand läuft (post-create.sh)
   - Rust/Node installieren
   - Registries konfigurieren
   - KEINE Dateiänderungen
   ↓
3. User startet Build
   - cargo build (generiert Cargo.lock)
   - npm install (generiert package-lock.json)
   ↓
4. Lockfiles sind clean, kein Konflikt
```

### Dokumentation generieren

```
docs/
├── next.md (manuell)
├── architecture/
│   └── plugin_system_v0.2.md (manuell)
└── _generated/
    ├── codex_output.md (Codex AI)
    └── api_spec.json (Generator)
```

---

## 🚨 Wenn der Fehler trotzdem auftritt

### Debug-Infos sammeln

```bash
# 1. Git Status
git status

# 2. Staged Changes
git diff --cached

# 3. Untracked Files
git ls-files --others --exclude-standard

# 4. Recent Commits
git log --oneline -10

# 5. Branch Info
git branch -a
```

### Notfall-Reset

```bash
# ⚠️ WARNUNG: Alle lokalen Änderungen gehen verloren!
git reset --hard origin/main
git clean -fd
```

---

## 📞 Häufige Fragen

**F: Verliere ich meine Änderungen?**  
A: Nein, wenn du `git diff > /tmp/patch.diff` machst. Später kannst du mit `git apply /tmp/patch.diff` wiederherstellen.

**F: Warum nicht einfach `git pull`?**  
A: `git pull` würde Merge-Konflikte verursachen. `git restore` ist sauberer.

**F: Muss ich Lockfiles committen?**  
A: **JA!** `package-lock.json` und `Cargo.lock` gehören ins Repo. Sie sichern die exakte Dependency-Version.

**F: Kann ich das automatisieren?**  
A: Ja! Nutze `.devcontainer/post-create.sh` + `.github/workflows/ci-cleanup.yml`

---

## 🎯 Zusammenfassung

| Problem | Lösung |
|---------|--------|
| Patch-Fehler lokal | `./scripts/cleanup-workspace.sh` |
| Patch-Fehler in CI | `.github/workflows/ci-cleanup.yml` |
| Patch-Fehler in Devcontainer | `updateContentCommand: false` |
| Parallel-Schreibzugriffe | Getrennte Verzeichnisse (`_generated/`) |
| Lockfile-Konflikte | Nur in Build-Phase ändern, nicht in Setup |

---

**Status**: ✅ **PREVENTION SETUP COMPLETE**

Dateien:
- ✅ `scripts/cleanup-workspace.sh` - Automatisches Cleanup
- ✅ `.devcontainer/devcontainer.json` - Devcontainer Config
- ✅ `.devcontainer/post-create.sh` - Setup Script
- ✅ `.github/workflows/ci-cleanup.yml` - CI Pipeline
- ✅ `PATCH_TROUBLESHOOTING.md` - Dieser Guide
