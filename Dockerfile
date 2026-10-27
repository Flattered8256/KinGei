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
USER app
ENV PORT=8000
EXPOSE 8000
CMD ["./chat"]
