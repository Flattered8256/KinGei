FROM node:22-bookworm-slim AS build
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY . .
ENV NUXT_TELEMETRY_DISABLED=1
RUN pnpm generate

FROM nginx:1.27-alpine
RUN apk add --no-cache wget
COPY --from=build /app/.output/public /usr/share/nginx/html
EXPOSE 80
