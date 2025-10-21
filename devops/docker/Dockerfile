# syntax=docker/dockerfile:1.6

ARG CHAT_MODEL_FILE=synthetic-chat
ARG EMBEDDING_MODEL_FILE=synthetic-embed
ARG PROMPT_TEMPLATE=assistant

###############################
# Build llmserver plug-in
###############################
FROM rust:1.82 as llmserver-builder
WORKDIR /build/llmserver
COPY core/plugins/llmserver/Cargo.toml ./Cargo.toml
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
COPY core/plugins/llmserver/ ./
RUN cargo build --release

###############################
# Build NestJS API
###############################
FROM node:20-bullseye as api-builder
WORKDIR /build/api
COPY core/backend/gateway/package*.json ./
RUN npm install --legacy-peer-deps || npm install --force
COPY core/backend/gateway/ ./
RUN npm run build

###############################
# Build Angular frontend
###############################
FROM node:20-bullseye as web-builder
WORKDIR /build/web
COPY core/frontend/admin-ui/package*.json ./
RUN npm install --legacy-peer-deps || npm install --force
COPY core/frontend/admin-ui/ ./
RUN npm run build -- --configuration production

###############################
# Prepare RepoAgent virtualenv
###############################
FROM python:3.11-slim as repoagent-builder
WORKDIR /build/repoagent
COPY core/plugins/repoagent/ ./
RUN python -m venv /opt/repoagent && \
    . /opt/repoagent/bin/activate && \
    pip install --upgrade pip && \
    pip install .

###############################
# Install apikeys dependencies
###############################
FROM node:20-bullseye as apikeys-builder
WORKDIR /build/apikeys
COPY core/plugins/apikeys/package.json ./
RUN npm install --production
COPY core/plugins/apikeys/src ./src
COPY core/plugins/apikeys/start.sh core/plugins/apikeys/config.json ./

###############################
# Final runtime image
###############################
FROM debian:bookworm-slim
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      supervisor \
      nodejs \
      npm \
      python3 \
      python3-venv \
      sqlite3 \
      libssl3 \
      dumb-init \
      build-essential \
      python3-dev && \
    rm -rf /var/lib/apt/lists/*

# Non-root user
RUN useradd -m -u 1000 bkg
USER bkg
WORKDIR /srv/bkg

# Copy application artefacts
COPY --chown=bkg:bkg --from=api-builder /build/api/dist ./core/backend/gateway/dist
COPY --chown=bkg:bkg --from=api-builder /build/api/node_modules ./core/backend/gateway/node_modules
COPY --chown=bkg:bkg core/backend/gateway/package.json core/backend/gateway/tsconfig*.json ./core/backend/gateway/

COPY --chown=bkg:bkg --from=web-builder /build/web/dist/bkg-web ./core/frontend/admin-ui/dist
COPY --chown=bkg:bkg core/frontend/admin-ui/package.json ./core/frontend/admin-ui/

COPY --chown=bkg:bkg --from=llmserver-builder /build/llmserver/target/release/bkg-llmserver ./core/plugins/llmserver/target/release/bkg-llmserver
COPY --chown=bkg:bkg core/plugins/llmserver/start.sh core/plugins/llmserver/config.json ./core/plugins/llmserver/

COPY --chown=bkg:bkg --from=repoagent-builder /opt/repoagent ./core/plugins/repoagent/.venv
COPY --chown=bkg:bkg core/plugins/repoagent/start.sh core/plugins/repoagent/config.json ./core/plugins/repoagent/

COPY --chown=bkg:bkg --from=apikeys-builder /build/apikeys ./core/plugins/apikeys
COPY --chown=bkg:bkg core/plugins/plugins.json ./core/plugins/plugins.json

COPY --chown=bkg:bkg devops/docker/supervisord.conf devops/docker/start.sh ./devops/docker/
COPY --chown=bkg:bkg docs ./docs
COPY --chown=bkg:bkg models ./models

ENV CHAT_MODEL_FILE=${CHAT_MODEL_FILE}
ENV EMBEDDING_MODEL_FILE=${EMBEDDING_MODEL_FILE}
ENV PROMPT_TEMPLATE=${PROMPT_TEMPLATE}
ENV BKG_DATABASE_PATH=/data/bkg.db
ENV NODE_ENV=production

VOLUME ["/data", "/srv/models"]
EXPOSE 43119 43117

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/srv/bkg/devops/docker/start.sh"]
