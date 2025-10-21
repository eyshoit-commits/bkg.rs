# syntax=docker/dockerfile:1.6

ARG CHAT_MODEL_FILE=synthetic-chat
ARG EMBEDDING_MODEL_FILE=synthetic-embed
ARG PROMPT_TEMPLATE=assistant

###############################
# Build llmserver plug-in
###############################
FROM rust:latest as llmserver-builder
WORKDIR /build/llmserver
COPY plugins/llmserver/Cargo.toml ./Cargo.toml
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
COPY plugins/llmserver/ ./
RUN cargo build --release

###############################
# Build NestJS API
###############################
FROM node:20-bullseye as api-builder
WORKDIR /build/api
COPY apps/bkg-api/package.json apps/bkg-api/package-lock.json* ./
RUN npm install --legacy-peer-deps || npm install --force
COPY apps/bkg-api/ ./
RUN npm run build

###############################
# Build Angular frontend
###############################
FROM node:20-bullseye as web-builder
WORKDIR /build/web
COPY apps/bkg-web/package.json apps/bkg-web/package-lock.json* ./
RUN npm install --legacy-peer-deps || npm install --force
COPY apps/bkg-web/ ./
RUN npm run build -- --configuration production

###############################
# Prepare RepoAgent virtualenv
###############################
FROM python:3.11-slim as repoagent-builder
WORKDIR /build/repoagent
COPY plugins/repoagent/ ./
RUN python -m venv /opt/repoagent && \
    . /opt/repoagent/bin/activate && \
    pip install --upgrade pip && \
    pip install .

###############################
# Install apikeys dependencies
###############################
FROM node:20-bullseye as apikeys-builder
WORKDIR /build/apikeys
COPY plugins/apikeys/package.json ./
RUN npm install --production
COPY plugins/apikeys/src ./src
COPY plugins/apikeys/start.sh plugins/apikeys/config.json ./

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
      dumb-init && \
    rm -rf /var/lib/apt/lists/*

# Non-root user
RUN useradd -m -u 1000 bkg
USER bkg
WORKDIR /srv/bkg

# Copy application artefacts
COPY --chown=bkg:bkg --from=api-builder /build/api/dist ./apps/bkg-api/dist
COPY --chown=bkg:bkg --from=api-builder /build/api/node_modules ./apps/bkg-api/node_modules
COPY --chown=bkg:bkg apps/bkg-api/package.json apps/bkg-api/tsconfig*.json ./apps/bkg-api/

COPY --chown=bkg:bkg --from=web-builder /build/web/dist/bkg-web ./apps/bkg-web/dist
COPY --chown=bkg:bkg apps/bkg-web/package.json ./apps/bkg-web/

COPY --chown=bkg:bkg --from=llmserver-builder /build/llmserver/target/release/bkg-llmserver ./plugins/llmserver/target/release/bkg-llmserver
COPY --chown=bkg:bkg plugins/llmserver/start.sh plugins/llmserver/config.json ./plugins/llmserver/

COPY --chown=bkg:bkg --from=repoagent-builder /opt/repoagent ./plugins/repoagent/.venv
COPY --chown=bkg:bkg plugins/repoagent/start.sh plugins/repoagent/config.json ./plugins/repoagent/

COPY --chown=bkg:bkg --from=apikeys-builder /build/apikeys ./plugins/apikeys
COPY --chown=bkg:bkg plugins/plugins.json ./plugins/plugins.json

COPY --chown=bkg:bkg docker/supervisord.conf docker/start.sh ./docker/
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
CMD ["/srv/bkg/docker/start.sh"]
