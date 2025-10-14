# Stage 1: Planner - Generate recipe.json
FROM lukemathwalker/cargo-chef:latest-rust-1.85-alpine AS planner
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - Cache dependencies and build
FROM lukemathwalker/cargo-chef:latest-rust-1.85-alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig

# Configure parallel builds
ENV CARGO_BUILD_JOBS=4

# Copy recipe and build dependencies (cached layer with BuildKit cache mounts)
COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/migration ./migration
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code and build application
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release

# Strip debug symbols
RUN strip target/release/bus-availabilities-scaper

# Stage 3: Runtime - Minimal Alpine image
FROM alpine:3.20 AS runtime
WORKDIR /app

# Install only CA certificates for HTTPS
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/release/bus-availabilities-scaper /usr/local/bin/

# Create data directory and non-root user
RUN mkdir -p /app/data && \
    adduser -D -u 1000 scraper && \
    chown -R scraper:scraper /app

USER scraper

ENTRYPOINT ["bus-availabilities-scaper"]
