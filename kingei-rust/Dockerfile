# 云平台只构建本仓库时用。前端从远程克隆。本机 Compose 用上层仓库的 Dockerfile。
FROM node:22-bookworm-slim AS web
RUN apt-get update \
    && apt-get install -y --no-install-recommends git ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
ARG NUXT_REPO=https://github.com/Flattered8256/kingei-nuxt.git
ARG NUXT_REF=main
RUN git clone --depth 1 --branch "${NUXT_REF}" "${NUXT_REPO}" .
RUN corepack enable
ENV NUXT_TELEMETRY_DISABLED=1
RUN pnpm install --frozen-lockfile && pnpm generate

FROM rust:1-bookworm AS builder
WORKDIR /app
ARG DB_BACKEND=sqlite
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config cmake clang perl build-essential \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --no-default-features --features ${DB_BACKEND} \
    && cp /app/target/release/chat /app/chat

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ffmpeg ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --home-dir /app --create-home app \
    && mkdir -p /data \
    && chown app:app /data
WORKDIR /app
COPY --from=builder /app/chat /app/chat
COPY --from=web /src/.output/public /app/public
RUN chown -R app:app /app
USER app
ENV PORT=3000
ENV STATIC_DIR=/app/public
EXPOSE 3000
CMD ["./chat"]
