# syntax=docker/dockerfile:1.6
FROM rust:1.76-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config.json ./
RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd --create-home --shell /usr/sbin/nologin brainml
WORKDIR /opt/brainml
COPY --from=builder /app/target/release/brainml ./brainml
COPY --from=builder /app/config.json ./config.json
USER brainml
ENV RUST_LOG=info
ENTRYPOINT ["/opt/brainml/brainml"]
