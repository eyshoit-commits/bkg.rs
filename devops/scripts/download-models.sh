#!/usr/bin/env bash
set -euo pipefail

# Farben für Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== bkg.rs Modell-Download ===${NC}\n"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")"/../.. && pwd)"
MODELS_DIR="$REPO_ROOT/models"
mkdir -p "$MODELS_DIR"

# Chat-Modell: Qwen2-0.5B (ca. 350MB)
CHAT_MODEL="$MODELS_DIR/Qwen2-0.5B-Instruct-Q5_K_M.gguf"
CHAT_URL="https://huggingface.co/Qwen/Qwen2-0.5B-Instruct-GGUF/resolve/main/Qwen2-0.5B-Instruct-Q5_K_M.gguf"

# Embedding-Modell: all-MiniLM-L6-v2 (ca. 22MB)
EMBED_MODEL="$MODELS_DIR/all-MiniLM-L6-v2-ggml-model-f16.gguf"
EMBED_URL="https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/ggml-model-f16.gguf"

download_file() {
  local url=$1
  local output=$2
  local name=$3

  if [ -f "$output" ]; then
    echo -e "${GREEN}✅ $name bereits vorhanden${NC}"
    return 0
  fi

  echo -e "${YELLOW}⬇️  Lade $name herunter...${NC}"
  echo "   URL: $url"
  echo "   Ziel: $output"
  echo ""

  if command -v curl &> /dev/null; then
    curl -L --progress-bar -o "$output" "$url"
  elif command -v wget &> /dev/null; then
    wget --show-progress -O "$output" "$url"
  else
    echo -e "${RED}❌ Weder curl noch wget gefunden${NC}"
    return 1
  fi

  if [ -f "$output" ]; then
    local size=$(du -h "$output" | cut -f1)
    echo -e "${GREEN}✅ $name heruntergeladen ($size)${NC}"
  else
    echo -e "${RED}❌ Download fehlgeschlagen${NC}"
    return 1
  fi
}

echo -e "${YELLOW}Modelle:${NC}"
echo "  1. Chat-Modell:      Qwen2-0.5B (ca. 350MB)"
echo "  2. Embedding-Modell: all-MiniLM-L6-v2 (ca. 22MB)"
echo ""

download_file "$CHAT_URL" "$CHAT_MODEL" "Chat-Modell"
echo ""
download_file "$EMBED_URL" "$EMBED_MODEL" "Embedding-Modell"

echo ""
echo -e "${GREEN}=== Download abgeschlossen ===${NC}"
echo ""
echo -e "${BLUE}Nächste Schritte:${NC}"
echo "  1. Starten mit Docker Compose:"
echo "     devops/scripts/docker-start.sh"
