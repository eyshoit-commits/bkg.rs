#!/bin/bash
# Workspace Cleanup Script - Behebe "Failed to apply patch" Fehler
# Verwendung: ./scripts/cleanup-workspace.sh

set -e

echo "🧹 WORKSPACE CLEANUP - Patch-Fehler beheben"
echo "=============================================="
echo ""

# 1. Aktuellen Status speichern
echo "📋 Schritt 1: Änderungen sichern..."
git diff > /tmp/patch_backup_$(date +%s).diff
echo "✅ Patch gespeichert: /tmp/patch_backup_*.diff"
echo ""

# 2. Staged Changes clearen
echo "📋 Schritt 2: Staged Changes clearen..."
git restore --staged . 2>/dev/null || true
echo "✅ Staged changes gelöscht"
echo ""

# 3. Working Directory clearen
echo "📋 Schritt 3: Working Directory clearen..."
git restore . 2>/dev/null || true
echo "✅ Working directory zurückgesetzt"
echo ""

# 4. Untracked Files entfernen (optional)
echo "📋 Schritt 4: Untracked Files prüfen..."
UNTRACKED=$(git clean -fd --dry-run 2>/dev/null | wc -l)
if [ "$UNTRACKED" -gt 0 ]; then
  echo "⚠️  $UNTRACKED untracked files gefunden"
  echo "   Führe aus: git clean -fd"
  git clean -fd
  echo "✅ Untracked files gelöscht"
else
  echo "✅ Keine untracked files"
fi
echo ""

# 5. Lockfiles neu generieren
echo "📋 Schritt 5: Lockfiles regenerieren..."

if [ -f "Cargo.toml" ]; then
  echo "  → Cargo.lock aktualisieren..."
  cargo update --dry-run 2>/dev/null || echo "  ⚠️  Cargo nicht verfügbar"
fi

if [ -f "package.json" ]; then
  echo "  → package-lock.json aktualisieren..."
  npm install --package-lock-only 2>/dev/null || echo "  ⚠️  npm nicht verfügbar"
fi

if [ -f "frontend/admin-ui/package.json" ]; then
  echo "  → frontend/admin-ui/package-lock.json aktualisieren..."
  cd frontend/admin-ui && npm install --package-lock-only 2>/dev/null || echo "  ⚠️  npm nicht verfügbar"
  cd ../..
fi

echo "✅ Lockfiles regeneriert"
echo ""

# 6. Status prüfen
echo "📋 Schritt 6: Status prüfen..."
git status
echo ""

echo "✅ WORKSPACE CLEANUP COMPLETE"
echo ""
echo "🎯 Nächste Schritte:"
echo "  1. npm start (oder cargo run)"
echo "  2. Falls noch Fehler: git clean -fd && npm install"
echo "  3. Bei CI-Fehler: Pipeline neu starten"
