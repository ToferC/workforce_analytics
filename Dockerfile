# syntax=docker/dockerfile:1
#
# Full multi-stage build of the `graphql_api` workspace binary. The runtime
# stays on the (larger) rust image, which is handy for debugging. For the
# small production image use Dockerfile.slim instead.
#
# Build context is the repository root (Cargo workspace: `graphql_api`, `errors`).

# ---- Build stage ------------------------------------------------------------
FROM rust:latest AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Whole workspace. Migrations are embedded at build time via embed_migrations!().
COPY Cargo.toml Cargo.lock ./
COPY graphql_api graphql_api
COPY errors errors
COPY migrations migrations

RUN cargo build --release --bin graphql_api

# ---- Runtime stage ----------------------------------------------------------
FROM rust:latest

WORKDIR /app

# Binary plus runtime assets (see Dockerfile.slim for path rationale).
COPY --from=build /app/target/release/graphql_api ./graphql_api
COPY graphql_api/templates graphql_api/templates
COPY graphql_api/static graphql_api/static
COPY seeds seeds

EXPOSE 8080

CMD ["./graphql_api"]
