# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Development (hot reload)
cargo leptos watch

# Build release
cargo leptos build --release

# Run tests
cargo test --workspace --features ssr

# Run single test
cargo test --workspace --features ssr test_name

# Coverage
cargo tarpaulin --workspace --features ssr --timeout 300

# Lint
cargo clippy --workspace --features ssr --all-targets

# Format
cargo fmt --all

# E2E tests (Playwright)
npm run test:e2e
```

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Frontend  │────▶│   Server    │────▶│   Tracker   │
│  (Leptos)   │     │   (Axum)    │     │  (Tokio)    │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       │                   ▼                   ▼
       │            ┌─────────────┐     ┌─────────────┐
       │            │   SQLite    │     │  BusScraper │
       │            │  (SeaORM)   │     │  (reqwest)  │
       │            └─────────────┘     └─────────────┘
       │                                      │
       ▼                                      ▼
┌─────────────┐                        ┌─────────────┐
│    WASM     │                        │   Discord   │
│  (hydrate)  │                        │   Webhook   │
└─────────────┘                        └─────────────┘
```

**Crates:**
- `app/` — Shared: entities, repositories, scraper, notifier, API, components (SSR-gated)
- `server/` — Axum server, SSR rendering, background tracker
- `frontend/` — WASM hydration entry point
- `migration/` — SeaORM migrations

**Key patterns:**
- Leptos server functions via `#[server]` macro → `/api/{fn_name}` endpoints
- Context injection: `provide_context(db)`, `provide_context(scraper)` 
- Tracker spawns per-user-route Tokio tasks with hash-based change detection
- SSR feature flag: most `app/` modules are `#[cfg(feature = "ssr")]`

## Git Rules

- **NEVER commit without explicit user permission**
- Always show changes and wait for approval before committing

## Testing Principles

**Mock strategy:**
- Mock ONLY external dependencies (third-party APIs)
- Never mock: Database (use SQLite in-memory), internal logic, repositories

| Type | Mocks | Example |
|------|-------|---------|
| Unit | None | Pure functions, hash calculation |
| Integration | External APIs only | `MockServer` for BusScraper |
| E2E | None | Playwright with real `cargo leptos serve` |

## Code Principles

- **DRY**: Extract repeated logic, single source of truth
- **YAGNI**: No speculative features, build only what's required now
- **KISS**: Simplest solution, flat over nested
