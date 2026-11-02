FROM node:22-bookworm-slim AS web
WORKDIR /src
RUN corepack enable
COPY kingei-nuxt/package.json kingei-nuxt/pnpm-lock.yaml kingei-nuxt/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY kingei-nuxt/ .
ENV NUXT_TELEMETRY_DISABLED=1
RUN pnpm generate

FROM rust:1-bookworm AS builder
WORKDIR /app
ARG DB_BACKEND=sqlite
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config cmake clang perl build-essential \
    && rm -rf /var/lib/apt/lists/*
COPY kingei-rust/Cargo.toml kingei-rust/Cargo.lock ./
COPY kingei-rust/src ./src
COPY kingei-rust/migrations ./migrations
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
