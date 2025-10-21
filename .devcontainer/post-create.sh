#!/bin/bash
# Post-Create Script für VSCode Devcontainer
# Wichtig: Keine Dateiänderungen hier! Nur Installation.

set -e

echo "🚀 bkg.rs Development Environment Setup"
echo "========================================"
echo ""

# 1. Rust Setup
echo "📦 Rust Setup..."
rustup update
rustup component add rustfmt clippy
echo "✅ Rust ready"
echo ""

# 2. Node Setup
echo "📦 Node Setup..."
npm config set registry https://registry.npmjs.org/
npm install -g @angular/cli
echo "✅ Node ready"
echo ""

# 3. Cargo Registry Fix (CRITICAL)
echo "📦 Cargo Registry Setup..."
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "crates-io-mirror"

[source.crates-io-mirror]
registry = "https://github.com/rust-lang/crates.io-index"
EOF
echo "✅ Cargo registry configured"
echo ""

# 4. Dependencies (NICHT in diesem Script ändern!)
echo "📦 Dependencies werden beim ersten Build installiert"
echo "   → cargo build wird lockfiles generieren"
echo "   → npm install wird package-lock.json generieren"
echo ""

# 5. Git Config
echo "📦 Git Setup..."
git config --global user.email "dev@bkg.rs" 2>/dev/null || true
git config --global user.name "bkg.rs Developer" 2>/dev/null || true
echo "✅ Git configured"
echo ""

# 6. Workspace Cleanup Script
echo "📦 Workspace Tools..."
chmod +x scripts/cleanup-workspace.sh
echo "✅ Cleanup script ready: ./scripts/cleanup-workspace.sh"
echo ""

echo "✅ SETUP COMPLETE"
echo ""
echo "🎯 Nächste Schritte:"
echo "  1. cargo build -p gateway"
echo "  2. cd frontend/admin-ui && npm install"
echo "  3. npm start"
echo ""
echo "⚠️  Bei Patch-Fehlern:"
echo "  ./scripts/cleanup-workspace.sh"
