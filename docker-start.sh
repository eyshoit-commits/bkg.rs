#!/usr/bin/env bash
set -euo pipefail

# Farben für Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== bkg.rs Docker Compose Startup ===${NC}\n"

# Überprüfe ob Docker installiert ist
if ! command -v docker &> /dev/null; then
  echo -e "${RED}❌ Docker ist nicht installiert${NC}"
  exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
  echo -e "${RED}❌ Docker Compose ist nicht installiert${NC}"
  exit 1
fi

# Überprüfe Modelle
echo -e "${YELLOW}Überprüfe Modelle...${NC}"
MODELS_DIR="/home/wind/devel/bkg.rs/models"
CHAT_MODEL="$MODELS_DIR/Qwen2-0.5B-Instruct-Q5_K_M.gguf"
EMBED_MODEL="$MODELS_DIR/all-MiniLM-L6-v2-ggml-model-f16.gguf"

if [ ! -f "$CHAT_MODEL" ]; then
  echo -e "${YELLOW}⚠️  Chat-Modell nicht gefunden: $CHAT_MODEL${NC}"
  echo "   Starten Sie: ./download-models.sh"
else
  echo -e "${GREEN}✅ Chat-Modell: $(basename $CHAT_MODEL) ($(du -h $CHAT_MODEL | cut -f1))${NC}"
fi

if [ ! -f "$EMBED_MODEL" ]; then
  echo -e "${YELLOW}⚠️  Embedding-Modell nicht gefunden: $EMBED_MODEL${NC}"
else
  echo -e "${GREEN}✅ Embedding-Modell: $(basename $EMBED_MODEL) ($(du -h $EMBED_MODEL | cut -f1))${NC}"
fi

echo ""
echo -e "${YELLOW}Starte Docker Compose...${NC}"
cd /home/wind/devel/bkg.rs

# Baue und starte Container
docker-compose up --build

echo -e "${GREEN}=== bkg.rs läuft ===${NC}"
echo -e "${BLUE}Frontend: http://localhost:43117${NC}"
echo -e "${BLUE}API:      http://localhost:43119${NC}"
