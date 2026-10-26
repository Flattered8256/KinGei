FROM node:22-bookworm-slim AS build
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY . .
ENV NUXT_TELEMETRY_DISABLED=1
RUN pnpm build

FROM node:22-bookworm-slim
WORKDIR /app
COPY --from=build --chown=node:node /app/.output /app/.output
USER node
ENV NODE_ENV=production \
    HOST=0.0.0.0 \
    PORT=3000 \
    NITRO_HOST=0.0.0.0 \
    NITRO_PORT=3000 \
    NUXT_TELEMETRY_DISABLED=1
EXPOSE 3000
CMD ["node", ".output/server/index.mjs"]
