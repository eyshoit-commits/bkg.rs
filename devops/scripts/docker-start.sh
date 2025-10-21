#!/usr/bin/env bash
set -euo pipefail

# Farben für Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== bkg.rs Docker Compose Startup ===${NC}\n"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")"/../.. && pwd)"
COMPOSE_FILE="$REPO_ROOT/devops/docker/docker-compose.yml"

# Überprüfe ob Docker installiert ist
if ! command -v docker &> /dev/null; then
  echo -e "${RED}❌ Docker ist nicht installiert${NC}"
  exit 1
fi

if ! docker compose version &> /dev/null; then
  if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ Docker Compose ist nicht installiert${NC}"
    exit 1
  fi
  COMPOSE_CMD="docker-compose"
else
  COMPOSE_CMD="docker compose"
fi

# Überprüfe Modelle
echo -e "${YELLOW}Überprüfe Modelle...${NC}"
MODELS_DIR="$REPO_ROOT/models"
CHAT_MODEL="$MODELS_DIR/Qwen2-0.5B-Instruct-Q5_K_M.gguf"
EMBED_MODEL="$MODELS_DIR/all-MiniLM-L6-v2-ggml-model-f16.gguf"

if [ ! -f "$CHAT_MODEL" ]; then
  echo -e "${YELLOW}⚠️  Chat-Modell nicht gefunden: $CHAT_MODEL${NC}"
  echo "   Starten Sie: devops/scripts/download-models.sh"
else
  echo -e "${GREEN}✅ Chat-Modell: $(basename "$CHAT_MODEL") ($(du -h "$CHAT_MODEL" | cut -f1))${NC}"
fi

if [ ! -f "$EMBED_MODEL" ]; then
  echo -e "${YELLOW}⚠️  Embedding-Modell nicht gefunden: $EMBED_MODEL${NC}"
else
  echo -e "${GREEN}✅ Embedding-Modell: $(basename "$EMBED_MODEL") ($(du -h "$EMBED_MODEL" | cut -f1))${NC}"
fi

echo ""
echo -e "${YELLOW}Starte Docker Compose...${NC}"

cd "$REPO_ROOT"
$COMPOSE_CMD -f "$COMPOSE_FILE" up --build

echo -e "${GREEN}=== bkg.rs läuft ===${NC}"
echo -e "${BLUE}Frontend: http://localhost:43117${NC}"
echo -e "${BLUE}API:      http://localhost:43119${NC}"
