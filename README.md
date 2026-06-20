# Fosslate

> [!WARNING]  
> Still a WIP! Not ready for use

<!-- 
Fosslate is a self-hosted localisation platform aimed as a free alternative to Crowdin.

## Stack

- Monorepo: Turborepo with pnpm workspaces
- Frontend: Next.js App Router, TypeScript, Tailwind CSS v4
- Backend: Rust, Axum, Tokio, SQLx, Postgres
- Runtime: one Docker image running the Next standalone server and Rust API

## Install dependencies

```sh
corepack enable
pnpm install
```

## Run Postgres locally

```sh
docker compose up -d postgres
```

The local database URL is:

```sh
postgres://fosslate:fosslate@127.0.0.1:5432/fosslate
```

## Run SQLx migrations

The API runs embedded migrations on startup. To run them manually with SQLx CLI:

```sh
cargo install sqlx-cli --no-default-features --features rustls,postgres
DATABASE_URL=postgres://fosslate:fosslate@127.0.0.1:5432/fosslate sqlx migrate run --source apps/api/migrations
```

## Run the backend locally

```sh
cd apps/api
cp .env.example .env
cargo run
```

Health endpoints:

- `http://127.0.0.1:4000/health`
- `http://127.0.0.1:4000/api/v1/meta`

API documentation:

- `http://127.0.0.1:4000/docs`
- `http://127.0.0.1:4000/openapi.json`

## Run the frontend locally

```sh
INTERNAL_API_URL=http://127.0.0.1:4000 pnpm --filter @fosslate/web dev
```

Open `http://localhost:3000`.

## Root commands

```sh
pnpm dev
pnpm build
pnpm lint
pnpm format
```

## Build the Docker image

```sh
docker build -t fosslate:local .
```

## Run the full stack with Compose

```sh
docker compose up --build
```

The app is exposed at `http://localhost:3000`. Inside the app container, the API listens on `127.0.0.1:4000` and the frontend talks to it through `INTERNAL_API_URL`.

## Container publishing

Pushes to `main` build and publish:

- `ghcr.io/${{ github.repository }}:latest`
- `ghcr.io/${{ github.repository }}:${{ github.sha }}` -->
