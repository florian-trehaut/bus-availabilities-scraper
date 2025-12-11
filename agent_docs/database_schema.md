# Database Schema (SeaORM + SQLite)

## Tables

### `users` - Configuration globale par utilisateur
| Column | Type | Description |
|--------|------|-------------|
| id | UUID, PK | |
| email | TEXT | |
| enabled | BOOLEAN | |
| notify_on_change_only | BOOLEAN | |
| scrape_interval_secs | INT | |
| discord_webhook_url | TEXT? | |
| created_at | TIMESTAMP | |

### `user_routes` - Routes suivies par user (1:N avec users)
| Column | Type | Description |
|--------|------|-------------|
| id | UUID, PK | |
| user_id | UUID, FK → users | |
| area_id, route_id | INT | |
| departure_station, arrival_station | TEXT | |
| date_start, date_end | TEXT | YYYY-MM-DD |
| departure_time_min, departure_time_max | TEXT? | HH:MM |
| created_at | TIMESTAMP | |

### `user_passengers` - Config passagers par route (1:1 avec user_routes)
| Column | Type | Description |
|--------|------|-------------|
| user_route_id | UUID, PK + FK | |
| adult_men, adult_women, ... | INT | 8 colonnes passagers |

### `route_states` - State tracking pour notify_on_change_only
| Column | Type | Description |
|--------|------|-------------|
| user_route_id | UUID, PK + FK | |
| last_seen_hash | TEXT | Hash des schedules disponibles |
| last_check | TIMESTAMP | |
| total_checks, total_alerts | INT | |

### `routes` - Catalogue Highway Bus (référence)
| Column | Type | Description |
|--------|------|-------------|
| route_id | TEXT, PK | |
| area_id | INT | Index |
| name | TEXT | |
| switch_changeable_flg | TEXT? | |
| created_at | TIMESTAMP | |

### `stations` - Catalogue stations (référence)
| Column | Type | Description |
|--------|------|-------------|
| station_id | TEXT, PK | |
| name | TEXT | |
| area_id | INT | |
| route_id | INT? | Index composite (area_id, route_id) |
| created_at | TIMESTAMP | |

## Seeding Workflow

```bash
# 1. Peupler catalogue (run once)
SEED_ROUTES_CATALOG=true cargo run

# 2. Seed user depuis .env (optionnel)
SEED_FROM_ENV=true cargo run

# 3. Run normal
cargo run
```
