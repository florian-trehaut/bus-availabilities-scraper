# Bus Scraper - Command Runner

# === Dev ===
dev:
    cargo leptos watch

build:
    cargo leptos build --release

# === Quality ===
fmt:
    cargo fmt --all

lint:
    cargo clippy --all-targets --all-features -- -D warnings

check: fmt lint

# === Test ===
test:
    cargo test -p app --features ssr

# === Database ===
db-migrate:
    sea-orm-cli migrate up

db-fresh:
    sea-orm-cli migrate fresh

db-seed:
    SEED_FROM_ENV=true SEED_ROUTES_CATALOG=true cargo run -p server

# === Docker ===
docker-build:
    docker compose build

docker-up:
    docker compose up -d

docker-logs:
    docker compose logs -f

docker-down:
    docker compose down

# === Release ===
release: check test build

# === Claude Flow ===
cf-init:
    npx claude-flow@alpha init --force --project-name bus-scraper

cf-swarm task:
    npx claude-flow@alpha swarm "{{task}}" --claude

cf-hive:
    npx claude-flow@alpha hive-mind wizard

cf-status:
    npx claude-flow@alpha swarm status

cf-store key value ns="default":
    npx claude-flow@alpha memory store {{key}} "{{value}}" --namespace {{ns}}

cf-query q ns="default":
    npx claude-flow@alpha memory query "{{q}}" --namespace {{ns}}

cf-memory:
    npx claude-flow@alpha memory list --reasoningbank

cf-mcp:
    claude mcp add claude-flow npx claude-flow@alpha mcp start
