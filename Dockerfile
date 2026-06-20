FROM node:22-bookworm-slim AS js-deps

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

WORKDIR /app

RUN corepack enable

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml turbo.json ./
COPY apps/api/package.json apps/api/package.json
COPY apps/web/package.json apps/web/package.json

RUN pnpm install --frozen-lockfile

FROM js-deps AS web-builder

COPY apps/web apps/web

RUN pnpm --filter @fosslate/web build

FROM rust:1-bookworm AS api-builder

WORKDIR /app/apps/api

COPY apps/api/Cargo.toml apps/api/Cargo.lock ./
COPY apps/api/migrations migrations
COPY apps/api/src src

RUN cargo build --release

FROM node:22-bookworm-slim AS runtime

WORKDIR /app

ENV NODE_ENV=production
ENV PORT=3000
ENV HOSTNAME=0.0.0.0
ENV API_HOST=127.0.0.1
ENV API_PORT=4000
ENV INTERNAL_API_URL=http://127.0.0.1:4000

COPY --from=web-builder /app/apps/web/.next/standalone ./
COPY --from=web-builder /app/apps/web/public ./apps/web/public
COPY --from=web-builder /app/apps/web/.next/static ./apps/web/.next/static
COPY --from=api-builder /app/apps/api/target/release/fosslate-api /app/fosslate-api
COPY docker/start.sh /app/start.sh

RUN chmod +x /app/start.sh

EXPOSE 3000

CMD ["/app/start.sh"]

