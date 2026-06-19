# People Data Analytics

This app is a learning project and attempt to create a data-centric model and Graphql API of employee skills, capabilities, certifications and work over time.

- [ ] Model people and their roles on teams
- [ ] Model people's skills and validate them based on their work
- [ ] Model how teams fit into an org hierarchy
- [ ] Model organizational capacity and work in progress
- [ ] Time-series modelling of changes to the organization over time as people change roles, learn and evolve.

It also includes :

- [x] User models
- [x] Automated Admin Generation
- [x] Authentication and sign-in

## Dependencies

- Rust (stable; workspace uses the 2021 edition and cargo resolver v3)
- PostgreSQL 14+
- `diesel_cli` with the postgres feature (for running migrations locally):
  `cargo install diesel_cli --no-default-features --features postgres`

This is a Cargo **workspace**. The runnable crate is `graphql_api` (binary
name `graphql_api`); `errors` is a helper member.

## Environment variables

Copy `.env.example` to `.env` and fill in values. The application loads `.env`
at startup (via `dotenv`).

| Variable | Required | Format / example | Purpose |
|---|---|---|---|
| `DATABASE_URL` | Yes | `postgres://user:pass@host/db?sslmode=require` | Postgres connection (Diesel). Use `sslmode=disable` only for local dev. |
| `SECRET_KEY` | Yes | long random string (`openssl rand -hex 32`) | Presence checked at startup. |
| `JWT_SECRET_KEY` | Yes | long random string, ≥32 bytes | HMAC secret signing/verifying JWTs (HS256). Rotating invalidates all tokens. |
| `ADMIN_NAME` | Yes | `"Admin Name"` | Seeds the bootstrap admin user on first run. |
| `ADMIN_EMAIL` | Yes | `admin@example.com` | Bootstrap admin email / login. |
| `ADMIN_PASSWORD` | Yes | strong password | Bootstrap admin password. |
| `ENVIRONMENT` | No (default `test`) | `production` \| `test` | `production` binds to `HOST:PORT`; otherwise binds `0.0.0.0:8080`. |
| `HOST` | If `production` | `0.0.0.0` | Bind address (production only). |
| `PORT` | If `production` | `8080` (u16) | Bind port (production only). |
| `ALLOWED_ORIGINS` | No (default `http://localhost:3000,http://localhost:8080`) | comma-separated origins | CORS allow-list (exact match). |
| `DISABLE_SCOPED_AUTHZ` | No | `true` \| `1` | Disables hierarchy-scoped authorization (grandfather fallback to flat operator+). Unset = scoped authz enforced. |
| `BOOTSTRAP_SERVERS`, `SECURITY_PROTOCOL`, `SASL_MECHANISMS`, `SASL_USERNAME`, `SASL_PASSWORD` | No | — | Kafka settings. **Not active** — the kafka module is currently disabled in `src/lib.rs`. |

> Note: earlier docs referenced `PASSWORD_SECRET_KEY`. It is no longer read by
> the code and can be removed from existing `.env` files.

## Local setup

- Clone the repository
- `cp .env.example .env` and edit values
- `diesel migration run` (applies `migrations/`; the binary also embeds and runs
  pending migrations at startup)
- `cargo run -p graphql_api`

The API serves on `0.0.0.0:8080` by default (GraphQL at `/graphql`).

## Deployment (Docker)

The build context is the repository root. Three Dockerfiles are provided:

| File | Final base | Use |
|---|---|---|
| `Dockerfile.slim` | `debian:bookworm-slim` | **Recommended** production image (smallest, non-root). Used by docker-compose. |
| `Dockerfile` | `rust:latest` | Multi-stage, larger runtime image (debugging). |
| `Dockerfile.simple` | `rust:latest` | Single-stage; quick local builds. |

All build the `graphql_api` binary and ship the runtime assets it resolves at
startup: `graphql_api/templates` (Tera), `graphql_api/static` (served at
`/static`), and `seeds/` (CSV seed data). Migrations are embedded into the
binary at build time.

### docker-compose (Postgres + API)

```bash
cp .env.example .env            # set secrets; DATABASE_URL is overridden for the db service
docker compose up -d db         # start Postgres (with healthcheck + volume)
docker compose up people-data-api   # build (Dockerfile.slim) and run the API on :8080
docker compose logs -f
```

Compose injects `HOST`, `PORT`, and a `DATABASE_URL` pointing at the `db`
service, so the API reaches Postgres in-network regardless of the `DATABASE_URL`
in your `.env` (which should target `localhost` for non-Docker runs).

### Build a specific image directly

```bash
docker build -f Dockerfile.slim -t workforce-analytics-api .
docker run --env-file .env -e HOST=0.0.0.0 -e PORT=8080 -p 8080:8080 workforce-analytics-api
```

## Dan's notes

### Running on MacOS

```bash
cargo install diesel_cli --no-default-features --features postgres # if not already installed

# MacOS only clean up when done
brew install libpq
brew link --force libpq

cargo clean

docker compose down; sleep 2; docker compose up -d db; sleep 10; diesel migration run
docker compose exec -it db psql -U christopherallison -W workforce_analytics
docker compose logs -f

time docker compose build people-data-api
docker images | grep epi
docker compose up
```

### WiP/TODO

- [x] Working: Dockerfile.simple again: worked (arm64:4.25GB)
- [x] Working: Dockerfile.slim finally working with base rust-image (arm64:1.98GB)
- [x] Working: Dockerfile.slim try debian:buster (arm64:444MB)
- [x] Working: Dockerfile.slim try debian:buster-slim : (arm64:392MB amd64:447MB )
- [x] Try again on codespaces (amd64)
  - [x] Rename Dockerfile.slim to Dockerfile.slim
- [x] Docker as non root user (rusty)

- [ ] Can I just copy src and Cargo.(toml|lock) into the image?
  - [ ] If so, Fix Dockerfile.simple as well
  - [ ] If so, remove the .dockerignore file
- [ ] Accelerate build with rust crate cache?
- [ ] Re-validate all dependencies in Dockerfile.slim
- [ ] alpine base image (musl)
  - [ ] [LogRocket Blog article](https://blog.logrocket.com/packaging-a-rust-web-service-using-docker/)
    - [ ] [Associated Repo](https://github.com/zupzup/rust-docker-web/blob/main/debian/Dockerfile)
  - [ ] Working: Dockerfile.alpine (arm64: 1.98GB)
  - [ ] Working: Dockerfile.alpine (arm64: 1.98GB)
- [x] http response type for index.html Content-Type: text/html; charset=UTF-8
- [x] replace openssl with libssl-dev in Dockerfile.simple
- [ ] Add e2e tests
- [x] progress indication with logging function
- [ ] Where shall we run diesel migrations?
  - Can use docker-compose to wait for db startup, and run migrations to completion
  - Create a diesel container, with migrations folders mounted, and run migrations
  - https://stackoverflow.com/questions/35069027/docker-wait-for-postgresql-to-be-running
  - https://docs.docker.com/compose/startup-order/

### Container image sizes

You can control the image you build by selecting the Dockerfile.XX in the docker-compose.yml file.
| Image              | arm64  | amd64                    | description                                |
|--------------------|--------|--------------------------|--------------------------------------------|
| rust:1.68          | 4.25GB |                          | single stage (Dockerfile.simple)           |
| rust:1.68          | 1.98GB |                          | multi-stage                                |
| debian:buster      | 444MB  |                          | multi-stage                                |
| debian:buster-slim | 392MB  | 447MB (15 minutes build) | multi-stage (Dockerfile.slim)              |
| alpine:3.14        |        |                          | multi-stage musl based (Dockerfile.alpine) |

### Caching the build

To enable the caching of the compiling of the rust dependencies, you can modify the start of the Dockerfile.slim to add this:

```dockerfile
SNIPPET
```

*measured on M2 Mac Mini:*

| Image             | first | subsequent with only code change | description  |
|-------------------|-------|----------------------------------|--------------|
| Dockerfile.simple | 238s  |                                  | single stage |
| Dockerfile.slim   | 216s  |                                  | multi-stage  |
| Dockerfile.fast   |       |                                  | multi-stage  |
