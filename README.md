# Highway Bus Availability Scraper

A full-stack Rust application that monitors seat availability on Japan's [HighwayBus.com](https://www.highwaybus.com/) and sends Discord notifications when seats become available.

## TL;DR

- Tracks overnight bus availability to Japanese mountain destinations (e.g., Kamikochi, Nagano)
- Sends real-time Discord alerts when seats open up
- Built with **Leptos 0.8** (SSR + WASM hydration), **Axum 0.8**, **SeaORM**, **Tokio**
- Concurrent per-user-route tracking with hash-based deduplication
- Supports date ranges, passenger configurations, and departure time filtering

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Frontend  │────▶│   Server    │────▶│   Tracker   │
│  (Leptos)   │     │   (Axum)    │     │  (Tokio)    │
└─────────────┘     └─────────────┘     └─────────────┘
                           │                   │
                           ▼                   ▼
                    ┌─────────────┐     ┌─────────────┐
                    │   SQLite    │     │  Scraper    │
                    │  (SeaORM)   │     │ (reqwest)   │
                    └─────────────┘     └─────────────┘
                                              │
                                              ▼
                                       ┌─────────────┐
                                       │  Discord    │
                                       │  Webhook    │
                                       └─────────────┘
```

**Crate structure:**
- `app/` — Shared domain logic, types, scraper, notifier, entities
- `server/` — Axum server, SSR rendering, background tracker
- `frontend/` — WASM hydration entry point
- `migration/` — SeaORM database migrations

## Why This Project?

I needed to book overnight buses from Tokyo to Kamikochi (Japanese Alps) for hiking trips. These buses sell out quickly—especially during peak seasons—and manually refreshing the booking page every few minutes wasn't practical.

So I built this tool to solve a real problem: automated monitoring with instant Discord notifications when seats become available.

## Tech Highlights

### Strict Linting Configuration

The workspace enforces production-grade code quality:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
dead_code = "deny"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
panic = "deny"
```

### Concurrent Per-User-Route Tracking

Each monitored route runs as an independent Tokio task with configurable intervals:

```rust
for user_route in user_routes {
    let tracker = UserTracker { user_route, scraper, db, notifier };
    tokio::spawn(async move {
        tracker.run().await;
    });
}
```

Uses `MissedTickBehavior::Skip` to prevent task backlog on slow responses.

### Hash-Based Change Detection

Avoids duplicate notifications by hashing schedule state:

```rust
fn calculate_state_hash(schedules: &[BusSchedule]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for schedule in schedules {
        schedule.departure_date.hash(&mut hasher);
        schedule.departure_time.hash(&mut hasher);
        for plan in &schedule.available_plans {
            plan.plan_id.hash(&mut hasher);
            plan.price.hash(&mut hasher);
            remaining_seats.hash(&mut hasher);
        }
    }
    hasher.finish()
}
```

### XML Parsing with quick-xml

Routes and stations are fetched from the HighwayBus API as XML and parsed with an event-based streaming approach:

```rust
let mut reader = Reader::from_str(xml);
reader.config_mut().trim_text(true);

loop {
    match reader.read_event_into(&mut buf) {
        Ok(Event::Start(e)) => match e.name().as_ref() {
            b"id" => current_id = Some(read_text(&mut reader)?),
            b"name" => current_name = Some(read_text(&mut reader)?),
            _ => {}
        },
        Ok(Event::Eof) => break,
        // ...
    }
}
```

### Domain Types with Comprehensive Tests

Strong typing for business logic with 12+ unit tests:

- `DateRange` — Supports both ISO (`2025-10-29`) and compact (`20251029`) formats
- `TimeFilter` — Departure time window filtering
- `PassengerCount` — Validation (1-12 passengers, gender-based counts for bus requirements)

```rust
#[test]
fn test_passenger_count_validation() {
    let too_many = PassengerCount { adult_men: 10, adult_women: 3, ..Default::default() };
    assert!(too_many.validate().is_err()); // Max 12 passengers
}
```

## Quick Start

### Prerequisites

- Rust 1.75+
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos)

```bash
cargo install cargo-leptos
rustup target add wasm32-unknown-unknown
```

### Run

```bash
cp .env.example .env  # Configure your Discord webhook
cargo leptos watch
```

### Docker

```bash
docker compose up -d
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite connection string | `sqlite://data/bus_scraper.db?mode=rwc` |
| `BASE_URL` | HighwayBus API base URL | `https://www.highwaybus.com/gp` |
| `ENABLE_TRACKER` | Enable background monitoring | `true` |
| `SEED_FROM_ENV` | Seed users/routes from env | `false` |
| `SEED_ROUTES_CATALOG` | Fetch routes catalog on startup | `false` |

## License

MIT
