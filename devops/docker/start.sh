#!/usr/bin/env bash
set -euo pipefail
mkdir -p /data
mkdir -p /srv/models
if [ -n "${CHAT_MODEL_FILE:-}" ] && [ ! -f "/srv/models/${CHAT_MODEL_FILE}" ]; then
  echo "[start] warning: chat model ${CHAT_MODEL_FILE} missing under /srv/models" >&2
fi
if [ -n "${EMBEDDING_MODEL_FILE:-}" ] && [ ! -f "/srv/models/${EMBEDDING_MODEL_FILE}" ]; then
  echo "[start] warning: embedding model ${EMBEDDING_MODEL_FILE} missing under /srv/models" >&2
fi
exec /usr/bin/supervisord -c /srv/bkg/devops/docker/supervisord.conf
