# Highway Bus API Reference

## Workflow API (5 étapes hiérarchiques)

L'API suit la structure des dropdowns en cascade du site web :

1. **Routes** → `fetch_routes(area_id)` - Toutes les routes d'une zone
2. **Departure Stations** → `fetch_departure_stations(route_id)` - Stations de départ pour une route
3. **Arrival Stations** → `fetch_arrival_stations(route_id, departure)` - Destinations valides
4. **Available Dates** → `fetch_available_dates(route_id, departure, arrival)` - Dates disponibles
5. **Schedules + Availability** → `fetch_schedules(request, date)` - Horaires précis + disponibilité + prix

## Endpoints

### Étapes 1-4 : Pulldown API
- **URL**: `https://www.highwaybus.com/gp/ajaxPulldown` (POST, XML)
- **Modes**: `line:full`, `station_geton`, `station_getoff`, `date`

### Étape 5 : Schedules
- **URL**: `https://www.highwaybus.com/gp/reservation/rsvPlanList` (GET, HTML)
- **Params**: 14 paramètres query incluant date, stations, et détails passagers

## Headers Obligatoires

```
User-Agent: Mozilla/5.0 ...
Referer: https://www.highwaybus.com/
```

Sans ces headers → block 403

## XML Non-Standard

Format inhabituel - multiples éléments avec même tag au même niveau :

```xml
<data>
  <id>2</id>
  <id>110</id>
  <name>Route A</name>
  <id>120</id>
  <name>Route B</name>
</data>
```

Solution : Parsers stateful custom (`parse_routes`, `parse_stations`, `parse_dates`)
- Accumuler `current_id` + `current_name`
- Flush record complet quand nouveau `<id>` détecté
- Voir `app/src/scraper.rs` pour implémentation

## Retry Logic

- Retry UNIQUEMENT sur 503 Service Unavailable
- Exponential backoff : 1s, 2s, 4s (MAX_RETRIES=3)
- Autres erreurs → fail fast

## Cookies

Le client doit persister les cookies entre requêtes (`.cookie_store(true)`)
