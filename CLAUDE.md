# Highway Bus Availability Scraper

Scraper Rust pour monitorer la disponibilité des bus Highway Bus (Japon) - routes Shinjuku-Fuji Five Lakes.

## Architecture du Projet

### Workflow API (5 étapes hiérarchiques)

L'API suit la structure des dropdowns en cascade du site web :

1. **Routes** → `fetch_routes(area_id)` - Toutes les routes d'une zone
2. **Departure Stations** → `fetch_departure_stations(route_id)` - Stations de départ pour une route
3. **Arrival Stations** → `fetch_arrival_stations(route_id, departure)` - Destinations valides
4. **Available Dates** → `fetch_available_dates(route_id, departure, arrival)` - Dates disponibles
5. **Schedules + Availability** → `fetch_schedules(request, date)` - Horaires précis + disponibilité + prix

**Endpoints API** :
- Étapes 1-4 : `https://www.highwaybus.com/gp/ajaxPulldown` (POST, XML)
  - Paramètres : `mode` change selon l'étape (line:full, station_geton, station_getoff, date)
- Étape 5 : `https://www.highwaybus.com/gp/reservation/rsvPlanList` (GET, HTML)
  - 14 paramètres query incluant date, stations, et détails passagers
- Headers **OBLIGATOIRES** : User-Agent + Referer (sinon block)

### Structure des Modules

```
src/
├── main.rs           # Entry point - multi-user scheduler tokio + logging
├── scraper.rs        # Client HTTP + API calls + XML parsing
├── html_parser.rs    # HTML parsing for schedules + availability
├── types.rs          # Route, Station, BusSchedule, ScrapeRequest
├── config.rs         # Legacy .env config (for SEED_FROM_ENV)
├── error.rs          # ScraperError + Result alias
├── db.rs             # Database connection init
├── entities/         # SeaORM entities (users, routes, stations, etc.)
├── repositories.rs   # Database queries (user_routes, route_state)
├── notifier.rs       # Discord webhook notifications
├── seed.rs           # Seed user from .env
├── seed_routes.rs    # Seed routes catalog from API
└── migration/        # Database migrations (SeaORM)
```

**src/scraper.rs** - Core
- `BusScraper` avec reqwest + cookie store enabled
- Parsers XML custom pour format non-standard (étapes 1-4)
- HTML parser pour horaires + disponibilité (étape 5)
- `check_availability_full()` : itère sur plage de dates et filtre par horaires
- `fetch_schedules()` : récupère horaires + disponibilité pour une date
- Retry logic avec exponential backoff sur 503 (MAX_RETRIES=3)
- Headers requis sur toutes les requêtes

**src/html_parser.rs** - HTML parsing
- `parse_schedules_html()` : parse page HTML des résultats de recherche
- `extract_time()` : extrait horaires depuis éléments HTML
- `extract_availability()` : parse hidden inputs pour disponibilité sièges
- `parse_remaining_seats()` : regex pour extraire nombre de sièges restants

**src/types.rs** - Data structures
- `ScrapeRequest` : params de query (area, route, stations, date_range, passengers, time_filter)
- `PassengerCount` : 8 catégories de passagers (adult/child/handicap × men/women)
- `DateRange` : plage de dates de recherche avec méthode `dates()` pour générer toutes les dates
- `TimeFilter` : filtrage optionnel par horaires (departure_min, departure_max)
- `Route`, `Station`, `AvailableDate` : entités parsées du XML
- `BusSchedule` : horaires complets + disponibilité + plans tarifaires
- `PricingPlan` : plan tarifaire avec prix et disponibilité
- `SeatAvailability` : enum (Available, SoldOut, Unknown)
- `AvailabilityResult` : legacy output avec dates disponibles

**src/config.rs** - Legacy environment config
- Load `.env` via dotenvy
- Parse et valide tous les params
- Return `Config` avec `ScrapeRequest`
- **Usage** : Uniquement pour `SEED_FROM_ENV=true` (migration legacy → DB)

**src/db.rs** + **src/entities/** - Database layer
- SeaORM avec SQLite
- Entities : `users`, `user_routes`, `user_passengers`, `route_states`, `routes`, `stations`
- Init database + connection pool

**src/repositories.rs** - Database queries
- `get_all_active_user_routes()` : fetch tous les users actifs avec leurs routes + passengers
- `get_route_state()` / `update_route_state()` : tracking hash pour notify_on_change_only
- `get_station_name()` : lookup nom station depuis catalog

**src/notifier.rs** - Discord webhooks
- `send_startup_notification()` : notif au démarrage (user_count, route_count)
- `send_availability_alert()` : notif disponibilité avec embeds formatés
- Non-bloquant : log errors mais ne fail pas

**src/seed.rs** - Seed user from .env
- Crée user + user_route + passengers depuis `.env`
- Validation : vérifie que route_id et stations existent dans catalog

**src/seed_routes.rs** - Seed routes catalog from API
- Scrape toutes les routes de area_id via `fetch_routes()`
- Pour chaque route : scrape departure + arrival stations
- Insert dans tables `routes` + `stations` avec dedup
- Run once avec `SEED_ROUTES_CATALOG=true`

**src/error.rs** - Error handling
- `ScraperError` : Http, Parse, Config, ServiceUnavailable, InvalidResponse
- Auto-conversion depuis reqwest::Error (check 503 status)
- Type alias `Result<T>`

**src/main.rs** - Multi-user scheduler
- Load tous les users actifs depuis DB via `get_all_active_user_routes()`
- Spawn 1 tokio task par user_route avec intervalle personnalisé
- Chaque task : `tokio::time::interval` + `MissedTickBehavior::Skip`
- Hash-based state tracking pour `notify_on_change_only`
- Graceful shutdown sur SIGTERM/SIGINT

### Parsing XML Non-Standard

Format API inhabituel - multiples éléments avec même tag au même niveau :

```xml

  2
  110
  Route A
  120
  Route B

```

**Solution** : Parsers stateful custom (`parse_routes`, `parse_stations`, `parse_dates`)
- Accumuler `current_id` + `current_name`
- Flush record complet quand nouveau `<id>` détecté
- Utilise `quick-xml` avec state machine

### Patterns HTTP Client

- **Cookies** : Client builder avec `.cookie_store(true)` pour session persistence
- **Retry** : Exponential backoff **uniquement sur 503** ; autres erreurs = fail fast
- **Rate limiting** : Implicite via scheduler interval + delays additionnels dans retry
- **Headers** : User-Agent + Referer sur CHAQUE requête

## Architecture Base de Données

### Tables (SeaORM + SQLite)

**`users`** - Configuration globale par utilisateur
- `id` (UUID, PK)
- `email` (TEXT)
- `enabled` (BOOLEAN)
- `notify_on_change_only` (BOOLEAN)
- `scrape_interval_secs` (INT)
- `discord_webhook_url` (TEXT nullable)
- `created_at` (TIMESTAMP)

**`user_routes`** - Routes suivies par user (1:N avec users)
- `id` (UUID, PK)
- `user_id` (UUID, FK → users)
- `area_id`, `route_id` (INT)
- `departure_station`, `arrival_station` (TEXT)
- `date_start`, `date_end` (TEXT, format YYYY-MM-DD)
- `departure_time_min`, `departure_time_max` (TEXT nullable, HH:MM)
- `created_at` (TIMESTAMP)

**`user_passengers`** - Config passagers par route (1:1 avec user_routes)
- `user_route_id` (UUID, PK + FK)
- 8 colonnes passagers (INT) : `adult_men`, `adult_women`, etc.

**`route_states`** - State tracking pour notify_on_change_only
- `user_route_id` (UUID, PK + FK)
- `last_seen_hash` (TEXT) : hash des schedules disponibles
- `last_check` (TIMESTAMP)
- `total_checks`, `total_alerts` (INT)

**`routes`** - Catalogue Highway Bus (référence)
- `route_id` (TEXT, PK)
- `area_id` (INT)
- `name` (TEXT)
- `switch_changeable_flg` (TEXT nullable)
- `created_at` (TIMESTAMP)
- Index sur `area_id`

**`stations`** - Catalogue stations (référence)
- `station_id` (TEXT, PK)
- `name` (TEXT)
- `area_id` (INT)
- `route_id` (INT nullable) : première route associée
- `created_at` (TIMESTAMP)
- Index composite sur `(area_id, route_id)`

### Workflow de Seeding

**1. First run - Peupler catalogue (run once)**
```bash
SEED_ROUTES_CATALOG=true cargo run
```
Scrape toutes routes + stations de area_id=1 via API → tables `routes` + `stations`

**2. Seed user depuis .env (optionnel)**
```bash
SEED_FROM_ENV=true cargo run
```
Crée user + user_route + passengers depuis `.env` → validation contre catalog

**3. Run normal**
```bash
cargo run
```
Load users actifs depuis DB → spawn 1 task/route avec intervalle personnalisé

## Configuration (.env)

### Variables de seeding
```bash
# Database
DATABASE_URL=sqlite://data/bus_scraper.db?mode=rwc

# Seed routes catalog (run once)
SEED_ROUTES_CATALOG=false

# Seed user from .env (legacy migration)
SEED_FROM_ENV=false
```

### Variables pour SEED_FROM_ENV (legacy)
```bash
# Geographic area (1 = Tokyo/Shinjuku)
AREA_ID=1

# Route to monitor
# 110 = Shinjuku-Fuji Five Lakes
# 155 = Shinjuku-Kamikochi
ROUTE_ID=155

# Station codes
DEPARTURE_STATION=001  # Busta Shinjuku
ARRIVAL_STATION=498    # Kamikochi Bus Terminal

# Date range (ISO 8601 or YYYYMMDD)
DATE_START=2025-10-12
DATE_END=2025-10-19

# Time filter (optional, HH:MM format)
DEPARTURE_TIME_MIN=06:00
DEPARTURE_TIME_MAX=10:00

# Passenger counts (8 categories, total 1-12)
ADULT_MEN=1
ADULT_WOMEN=0
CHILD_MEN=0
CHILD_WOMEN=0
HANDICAP_ADULT_MEN=0
HANDICAP_ADULT_WOMEN=0
HANDICAP_CHILD_MEN=0
HANDICAP_CHILD_WOMEN=0

# Query interval in seconds (default 300 = 5 minutes)
SCRAPE_INTERVAL_SECS=300
```

Copier `.env.example` → `.env` avant de run.

## Commandes

```bash
# Check
cargo check

# Format + Lint + Test (AVANT CHAQUE COMMIT)
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test

# Run
cargo run

# Build release
cargo build --release
```

## Configuration Clippy STRICTE

### Cargo.toml - Lints

```toml
[package]
edition = "2021"

[lints.rust]
unsafe_code = "forbid"
dead_code = "deny"
unused_imports = "deny"
unused_variables = "deny"

[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }

# Critiques
unwrap_used = "deny"           # Jamais .unwrap()
expect_used = "deny"           # Jamais .expect()
panic = "deny"                 # Jamais panic!()
todo = "deny"                  # Pas de TODO en prod
indexing_slicing = "warn"      # Attention aux []
cognitive_complexity = "warn"  # Max 15 (voir clippy.toml)

# Performance
inefficient_to_string = "deny"
needless_collect = "deny"

[profile.release]
lto = true
codegen-units = 1
strip = true
```

### clippy.toml

```toml
cognitive-complexity-threshold = 15    # Défaut: 25
too-many-arguments-threshold = 5       # Défaut: 7
too-many-lines-threshold = 100         # Défaut: 100
disallowed-names = ["foo", "bar", "baz"]
```

## Règles de Code NON-NÉGOCIABLES

### 1. Error Handling

```rust
// ❌ INTERDIT
let data = result.unwrap();
let data = result.expect("failed");

// ✅ OBLIGATOIRE
let data = result?;
let data = result.map_err(|e| ScraperError::from(e))?;
```

### 2. HTTP Patterns Spécifiques

```rust
// Headers OBLIGATOIRES sur toutes les requêtes
let response = client
    .post("https://www.highwaybus.com/gp/ajaxPulldown")
    .header("User-Agent", "Mozilla/5.0 ...")
    .header("Referer", "https://www.highwaybus.com/")
    .form(&params)
    .send()
    .await?;
```

### 3. Retry Logic

```rust
// Retry UNIQUEMENT sur 503 Service Unavailable
const MAX_RETRIES: u32 = 3;

for attempt in 0..MAX_RETRIES {
    match make_request().await {
        Ok(data) => return Ok(data),
        Err(e) if is_503_error(&e) && attempt < MAX_RETRIES - 1 => {
            let delay = Duration::from_secs(2u64.pow(attempt));
            tokio::time::sleep(delay).await;
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

### 4. XML Parsing Pattern

```rust
// Parser stateful pour XML non-standard
let mut reader = Reader::from_str(xml);
let mut current_id = String::new();
let mut current_name = String::new();
let mut results = Vec::new();

loop {
    match reader.read_event()? {
        Event::Start(e) if e.name().as_ref() == b"id" => {
            // Nouveau ID → flush previous si complet
            if !current_id.is_empty() && !current_name.is_empty() {
                results.push(Route { id: current_id, name: current_name });
                current_id = String::new();
                current_name = String::new();
            }
        }
        Event::Text(e) => {
            // Accumuler text...
        }
        Event::Eof => break,
        _ => {}
    }
}
```

### 5. Scheduler Pattern

```rust
#[tokio::main]
async fn main() -> Result {
    let config = Config::load()?;
    let scraper = BusScraper::new();
    
    let mut interval = tokio::time::interval(
        Duration::from_secs(config.interval_secs)
    );
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    
    loop {
        interval.tick().await;
        
        match scraper.fetch_availability(&config.request).await {
            Ok(result) => tracing::info!("Success: {:?}", result),
            Err(e) => tracing::error!("Error: {}", e),
        }
    }
}
```

## Dépendances

```toml
[dependencies]
tokio = { version = "1.42", features = ["full"] }
reqwest = { version = "0.12", features = ["cookies", "json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
quick-xml = "0.36"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
scraper = "0.20"  # HTML parsing
regex = "1.10"    # Seat extraction
```

## Workflow Strict

### Pre-commit (obligatoire)

```bash
#!/bin/bash
cargo fmt --all -- --check || exit 1
cargo clippy --all-targets -- -D warnings || exit 1
cargo test || exit 1
```

Installer : `cp pre-commit .git/hooks/ && chmod +x .git/hooks/pre-commit`

### Checklist avant commit

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all-targets -- -D warnings` (0 warning)
- [ ] `cargo test` (tous passent)
- [ ] Pas de `.unwrap()` ou `.expect()` sans `#[allow]`
- [ ] Complexity < 15 par fonction

## Points d'Attention Spécifiques

### API Highway Bus

1. **Session cookies** : Le client doit persister les cookies entre requêtes
2. **Headers obligatoires** : Sans User-Agent + Referer → block 403
3. **Rate limiting** : Respecter interval minimum (5 minutes recommandé)
4. **Retry sur 503** : API renvoie parfois 503 temporaire → retry avec backoff

### XML Non-Standard

1. Pas de structure hiérarchique classique
2. Elements `<id>` et `<name>` au même niveau, répétés
3. Parser custom requis - ne pas utiliser de deserializer automatique
4. Toujours valider que `id` et `name` sont appariés

### Production

1. **Logs structurés** : JSON format pour parsing facile
2. **Timestamps UTC** : Utiliser chrono avec UTC explicite
3. **Graceful shutdown** : Handle SIGTERM/SIGINT proprement
4. **Health checks** : Exposer endpoint /health si déployé avec monitoring

## Debugging

```bash
# Logs détaillés
RUST_LOG=debug cargo run

# Logs HTTP reqwest
RUST_LOG=reqwest=debug cargo run

# Logs projet uniquement
RUST_LOG=highway_bus_scraper=trace cargo run

# Test parsing XML
cargo test parse_routes -- --nocapture
```