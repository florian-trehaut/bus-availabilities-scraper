# Documentation Technique Complète - Highway Bus API

## Section 1: Architecture API Globale

### Vue d'Ensemble du Workflow

L'API Highway Bus suit une architecture en **2 phases distinctes** :

**Phase 1 : Sélection hiérarchique (AJAX/XML)**
- 4 endpoints AJAX pour la navigation en cascade
- Format XML non-standard avec éléments répétés
- Pas d'authentification mais headers obligatoires

**Phase 2 : Recherche et réservation (HTTP/HTML)**
- 1 endpoint GET pour la recherche complète
- Données embarquées dans le HTML de réponse
- Disponibilité et prix calculés côté serveur

### Diagramme des Dépendances

```
Homepage (/)
    ↓
[1] ajaxPulldown (mode=line:full) → Routes
    ↓
[2] ajaxPulldown (mode=station_geton) → Departure Stations
    ↓
[3] ajaxPulldown (mode=station_getoff) → Arrival Stations
    ↓
[4] ajaxPulldown (mode=date) → Available Dates
    ↓
[5] /gp/reservation/rsvPlanList (GET) → Schedules + Availability + Pricing
    ↓
[6] ajaxPlanInfoPrint → Plan Details (optionnel)
    ↓
[7] /gp/reservation/rsvPlanSelected (POST) → Seat Selection
```

### Headers Requis (CRITIQUE)

**Obligatoires sur TOUS les appels** :
```
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36
Referer: https://www.highwaybus.com/
```

Sans ces headers → **HTTP 403 Forbidden**

### Session et Cookies

- **Cookie store requis** : Le client doit persister les cookies entre requêtes
- **Pas de CSRF initial** : Les tokens CSRF sont générés dynamiquement lors de la recherche
- **Pas de login requis** : L'API est publique pour la consultation

---

## Section 2: Catalogue Exhaustif des Endpoints

### Endpoint 1: Fetch Routes

**URL** : `https://www.highwaybus.com/gp/ajaxPulldown`

**Méthode** : POST

**Headers** :
```
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36
Referer: https://www.highwaybus.com/
Content-Type: application/x-www-form-urlencoded
```

**Paramètres** :
| Nom | Type | Requis | Description | Exemple |
|-----|------|--------|-------------|---------|
| mode | string | ✓ | Doit être "line:full" | line:full |
| id | int | ✓ | Area ID (zone géographique) | 1 (Tokyo/Shinjuku) |

**Exemple curl** :
```bash
curl -X POST 'https://www.highwaybus.com/gp/ajaxPulldown' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36' \
  -H 'Referer: https://www.highwaybus.com/' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'mode=line:full&id=1'
```

**Réponse XML** :
```xml
<?xml version="1.0" encoding="UTF-8" ?>
<results>
<rosen>
    <num>34</num>
    <id>110</id>
        <name>新宿～富士五湖線</name>
        <switchChangeableFlg>1</switchChangeableFlg>
        <nearbyStationFlg></nearbyStationFlg>
    <id>120</id>
        <name>新宿～甲府線</name>
        <switchChangeableFlg>1</switchChangeableFlg>
        <nearbyStationFlg></nearbyStationFlg>
    <!-- ... -->
</rosen>
</results>
```

**Structure de données** :
- `num` : Nombre total de routes
- `id` : Route ID (ex: 110 = Shinjuku-Fuji Five Lakes)
- `name` : Nom de la route en japonais
- `switchChangeableFlg` : Flag de changement (1 = autorisé)
- `nearbyStationFlg` : Flag de station proche

---

### Endpoint 2: Fetch Departure Stations

**URL** : `https://www.highwaybus.com/gp/ajaxPulldown`

**Méthode** : POST

**Paramètres** :
| Nom | Type | Requis | Description | Exemple |
|-----|------|--------|-------------|---------|
| mode | string | ✓ | Doit être "station_geton" | station_geton |
| id | int | ✓ | Route ID (de Endpoint 1) | 110 |

**Exemple curl** :
```bash
curl -X POST 'https://www.highwaybus.com/gp/ajaxPulldown' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36' \
  -H 'Referer: https://www.highwaybus.com/' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'mode=station_geton&id=110'
```

**Réponse XML** :
```xml
<?xml version="1.0" encoding="UTF-8" ?>
<results>
<rosen>
    <num>54</num>
    <id>001</id>
        <name>バスタ新宿（南口）</name>
        <switchChangeableFlg></switchChangeableFlg>
        <nearbyStationFlg>1</nearbyStationFlg>
    <id>007</id>
        <name>新宿西口臨時便２６番のりば</name>
        <switchChangeableFlg></switchChangeableFlg>
        <nearbyStationFlg>1</nearbyStationFlg>
    <!-- ... -->
</rosen>
</results>
```

**Structure** :
- `id` : Station code (ex: 001 = Busta Shinjuku)
- `name` : Nom de la station
- `nearbyStationFlg` : 1 si station proche disponible

---

### Endpoint 3: Fetch Arrival Stations

**URL** : `https://www.highwaybus.com/gp/ajaxPulldown`

**Méthode** : POST

**Paramètres** :
| Nom | Type | Requis | Description | Exemple |
|-----|------|--------|-------------|---------|
| mode | string | ✓ | Doit être "station_getoff" | station_getoff |
| id | int | ✓ | Route ID | 110 |
| stationcd | string | ✓ | Departure station code | 001 |

**Exemple curl** :
```bash
curl -X POST 'https://www.highwaybus.com/gp/ajaxPulldown' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36' \
  -H 'Referer: https://www.highwaybus.com/' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'mode=station_getoff&id=110&stationcd=001'
```

**Réponse** : Même format XML que Endpoint 2 (liste de stations)

**Note** : La liste des stations d'arrivée est **filtrée** selon la station de départ

---

### Endpoint 4: Fetch Available Dates

**URL** : `https://www.highwaybus.com/gp/ajaxPulldown`

**Méthode** : POST

**Paramètres** :
| Nom | Type | Requis | Description | Exemple |
|-----|------|--------|-------------|---------|
| mode | string | ✓ | Doit être "date" | date |
| id | int | ✓ | Route ID | 110 |
| onStation | string | ✓ | Departure station code | 001 |
| offStation | string | ✓ | Arrival station code | 064 |

**Exemple curl** :
```bash
curl -X POST 'https://www.highwaybus.com/gp/ajaxPulldown' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36' \
  -H 'Referer: https://www.highwaybus.com/' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'mode=date&id=110&onStation=001&offStation=064'
```

**Réponse XML** :
```xml
<?xml version="1.0" encoding="UTF-8" ?>
<results>
<rosen>
    <num>31</num>
    <id>20251012</id>
        <name>2025/10/12(日)</name>
        <switchChangeableFlg></switchChangeableFlg>
        <nearbyStationFlg></nearbyStationFlg>
    <id>20251013</id>
        <name>2025/10/13(祝)</name>
        <switchChangeableFlg></switchChangeableFlg>
        <nearbyStationFlg></nearbyStationFlg>
    <!-- ... jusqu'à +30 jours -->
</rosen>
</results>
```

**Structure** :
- `id` : Date au format YYYYMMDD
- `name` : Date formatée en japonais avec jour de semaine
- Retourne ~30 jours de disponibilité

---

### Endpoint 5: Search Schedules + Availability + Pricing (CRITIQUE)

**URL** : `https://www.highwaybus.com/gp/reservation/rsvPlanList`

**Méthode** : GET

**Headers** :
```
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36
Referer: https://www.highwaybus.com/
```

**Paramètres** :
| Nom | Type | Requis | Description | Exemple |
|-----|------|--------|-------------|---------|
| mode | string | ✓ | Doit être "search" | search |
| route | int | ✓ | Area ID | 1 |
| lineId | int | ✓ | Route ID | 110 |
| onStationCd | string | ✓ | Departure station code | 001 |
| offStationCd | string | ✓ | Arrival station code | 064 |
| bordingDate | string | ✓ | Date YYYYMMDD | 20251012 |
| danseiNum | int | ✓ | Nombre d'hommes | 1 |
| zyoseiNum | int | ✓ | Nombre de femmes | 0 |
| adultMen | int | ✓ | Hommes adultes | 1 |
| adultWomen | int | ✓ | Femmes adultes | 0 |
| childMen | int | ✓ | Garçons enfants | 0 |
| childWomen | int | ✓ | Filles enfants | 0 |
| handicapAdultMen | int | ✓ | Hommes handicapés | 0 |
| handicapAdultWomen | int | ✓ | Femmes handicapées | 0 |
| handicapChildMen | int | ✓ | Garçons handicapés | 0 |
| handicapChildWomen | int | ✓ | Filles handicapées | 0 |
| nearCheckOnStation | string | ✗ | Stations proches départ | on/vide |
| nearCheckOffStation | string | ✗ | Stations proches arrivée | on/vide |

**Exemple curl** :
```bash
curl -G 'https://www.highwaybus.com/gp/reservation/rsvPlanList' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36' \
  -H 'Referer: https://www.highwaybus.com/' \
  --data-urlencode 'mode=search' \
  --data-urlencode 'route=1' \
  --data-urlencode 'lineId=110' \
  --data-urlencode 'onStationCd=001' \
  --data-urlencode 'offStationCd=064' \
  --data-urlencode 'bordingDate=20251012' \
  --data-urlencode 'danseiNum=1' \
  --data-urlencode 'zyoseiNum=0' \
  --data-urlencode 'adultMen=1' \
  --data-urlencode 'adultWomen=0' \
  --data-urlencode 'childMen=0' \
  --data-urlencode 'childWomen=0' \
  --data-urlencode 'handicapAdultMen=0' \
  --data-urlencode 'handicapAdultWomen=0' \
  --data-urlencode 'handicapChildMen=0' \
  --data-urlencode 'handicapChildWomen=0'
```

**Format de réponse** : HTML (pas JSON/XML!)

**Données embarquées dans le HTML** :

**1. Horaires** : Extraits du HTML
```html
<section class="busSvclistItem busSvclistItem_1">
    <section class="busSvclistItem_desc">
        <ul>
            <li class="dep">
                <p>バスタ新宿（南口）</p>
                <p class="day"><span>2025年</span>10<span>月</span>13<span>日</span></p>
                <p class="time">6:45 発</p>
            </li>
            <li class="arr">
                <p>河口湖駅</p>
                <p class="day"><span>2025年</span>10<span>月</span>13<span>日</span></p>
                <p class="time">8:30 着</p>
            </li>
        </ul>
    </section>
</section>
```

**2. Disponibilité sièges** : Hidden inputs
```html
<input type="hidden" class="seat_1" data-index="1" value="2">
<input type="hidden" class="seat_1" data-index="2" value="2">
<input type="hidden" class="seat_1" data-index="6" value="1">
<input type="hidden" class="seat_1" data-index="7" value="1">
```

**Valeurs** :
- `value="1"` : Sièges disponibles pour ce plan
- `value="2"` : Sièges NON disponibles pour ce plan

**3. Prix et plans tarifaires** :
```html
<input type="hidden" class="price_1" data-index="1" value="2200">
<input type='hidden' id='display_price_1' value='2,200'>

<form id="form_6">
    <input type="hidden" name="wayNo" value="3161"/>
    <input type="hidden" name="discntPlanNo" value="27775"/>
    <input type="hidden" name="busNo" value="1"/>
    <button>残り1席</button> <!-- Texte = disponibilité -->
</form>
```

**4. Informations de bus** :
```html
<dt>路線/便番号</dt>
<dd>新宿～富士五湖線&nbsp;1401 便</dd>
```

**Parsing HTML requis** :
- Chercher `class="busSvclistItem_N"` pour chaque bus
- Extraire horaires depuis `class="time"`
- Parser hidden inputs `seat_N` pour disponibilité
- Extraire `wayNo`, `discntPlanNo`, prix depuis forms
- Lire numéro de bus depuis modal headers

---

### Endpoint 6: Fetch Plan Details (Optionnel)

**URL** : `https://www.highwaybus.com/gp/reservation/ajaxPlanInfoPrint`

**Méthode** : GET

**Paramètres** :
| Nom | Type | Requis | Description | Exemple |
|-----|------|--------|-------------|---------|
| discntPlanNo | int | ✓ | Discount plan number | 27775 |
| reference | string | ✓ | Doit être "reservation" | reservation |

**Exemple curl** :
```bash
curl 'https://www.highwaybus.com/gp/reservation/ajaxPlanInfoPrint?discntPlanNo=27775&reference=reservation' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36' \
  -H 'Referer: https://www.highwaybus.com/gp/reservation/rsvPlanList'
```

**Réponse XML** :
```xml
<?xml version="1.0" encoding="UTF-8" ?>
<results>
<discntPlan>
    <discntNote>バスタ新宿と富士五湖方面を結ぶバスの運賃になります。</discntNote>
    <appliedCondNote>支払期限は予約日を含む３日以内...</appliedCondNote>
    <wayChangeNote>紙の乗車券（運行会社・コンビニ発行）は...</wayChangeNote>
    <repayFeeNote>乗車券記載のバス（便）が出発する前に限り...</repayFeeNote>
    <pointNote>窓口・コンビ二での決済は、ポイント付与対象外です。</pointNote>
    <settlementLimitNote>支払期限は予約日を含む３日以内...</settlementLimitNote>
    <etcNote>往復割引はありません。</etcNote>
    <linkUrl></linkUrl>
    <ladyOnlyFlg>0</ladyOnlyFlg>
    <wirelessFreeFlg>1</wirelessFreeFlg>
    <wirelessPayFlg>0</wirelessPayFlg>
    <tvFlg>0</tvFlg>
    <plugFlg>0</plugFlg>
    <blanketFlg>0</blanketFlg>
    <slipperFlg>0</slipperFlg>
    <pillowFlg>0</pillowFlg>
    <oneSeatFlg>0</oneSeatFlg>
    <relaxFlg>0</relaxFlg>
    <wideSeatFlg>0</wideSeatFlg>
    <drinkFlg>0</drinkFlg>
</discntPlan>
</results>
```

**Structure** :
- Conditions tarifaires (notes)
- Équipements du bus (flags 0/1)
- Règles de modification/remboursement

---

### Endpoint 7: Submit Reservation

**URL** : `https://www.highwaybus.com/gp/reservation/rsvPlanSelected`

**Méthode** : POST

**Paramètres** (extraits du form HTML) :
| Nom | Type | Description |
|-----|------|-------------|
| highwaybus.tokenHash | string | Token CSRF (extrait du HTML) |
| dispatch | string | "/reservation/rsvPlanSelected" |
| wayNo | int | Way number (ID du trajet) |
| discntPlanNo | int | Discount plan number |
| busNo | int | Bus number |
| ladyFlg | int | Flag femmes uniquement |
| danseiNum | int | Nombre hommes |
| zyoseiNum | int | Nombre femmes |
| adultMen/Women/etc | int | Détail passagers |

**Note** : Cet endpoint nécessite une session active et le token CSRF de la page de recherche

---

## Section 3: Workflow Complet de Scraping

### Workflow Optimal pour Obtenir Disponibilité + Horaires

```python
# Pseudo-code du workflow complet

async fn fetch_complete_availability(
    area_id: u32,
    route_id: u32,
    departure_station: &str,
    arrival_station: &str,
    date: &str,  # YYYYMMDD
    passengers: PassengerCount
) -> Result<Vec<BusSchedule>> {

    # Étape 1: Valider que la date est disponible
    dates = fetch_available_dates(route_id, departure_station, arrival_station).await?;
    if !dates.contains(date) {
        return Err("Date not available");
    }

    # Étape 2: Recherche complète (horaires + disponibilité + prix)
    let html = fetch_search_results(
        area_id,
        route_id,
        departure_station,
        arrival_station,
        date,
        passengers
    ).await?;

    # Étape 3: Parser le HTML
    let schedules = parse_html_schedules(html);

    # Pour chaque bus trouvé:
    for schedule in schedules {
        # Extraire horaires
        schedule.departure_time = extract_time(html, "dep", schedule.index);
        schedule.arrival_time = extract_time(html, "arr", schedule.index);
        schedule.bus_number = extract_bus_number(html, schedule.index);

        # Extraire disponibilité
        let seat_inputs = find_elements(html, f"seat_{schedule.index}");
        for plan in seat_inputs {
            if plan.value == "1" {
                # Sièges disponibles
                schedule.available_plans.push({
                    plan_id: plan.data_index,
                    available: true,
                    remaining_seats: extract_remaining(html, plan.data_index)
                });
            }
        }

        # Extraire prix
        schedule.prices = extract_prices(html, schedule.index);
        schedule.way_no = extract_way_no(html, schedule.index);
    }

    # Étape 4 (optionnel): Détails des plans
    for schedule in schedules {
        for plan in schedule.available_plans {
            plan.details = fetch_plan_info(plan.plan_id).await?;
        }
    }

    return Ok(schedules);
}
```

### Paramètres à Chaque Étape

**Étape 1 - Dates disponibles** :
```
mode=date
id=110
onStation=001
offStation=064
```

**Étape 2 - Recherche complète** :
```
mode=search
route=1
lineId=110
onStationCd=001
offStationCd=064
bordingDate=20251012
danseiNum=1
zyoseiNum=0
adultMen=1
adultWomen=0
childMen=0
childWomen=0
handicapAdultMen=0
handicapAdultWomen=0
handicapChildMen=0
handicapChildWomen=0
```

**Étape 3 - Détails plan** (pour chaque `discntPlanNo` trouvé) :
```
discntPlanNo=27775
reference=reservation
```

### Gestion des Erreurs

**503 Service Unavailable** :
- Retry avec exponential backoff (2^attempt secondes)
- Max 3 tentatives

**Autres erreurs HTTP** :
- Fail fast, ne pas retry

**HTML parsing** :
- Vérifier présence de `class="busSvclistItem"`
- Si aucun bus trouvé → route/date invalide ou complet

---

## Section 4: Structures de Données

### Route
```rust
struct Route {
    id: String,              // "110"
    name: String,            // "新宿～富士五湖線"
    switch_changeable: bool, // switchChangeableFlg
    has_nearby_stations: bool,
}
```

### Station
```rust
struct Station {
    code: String,            // "001", "064"
    name: String,            // "バスタ新宿（南口）"
    is_nearby: bool,         // nearbyStationFlg
}
```

### AvailableDate
```rust
struct AvailableDate {
    date_id: String,         // "20251012"
    display_name: String,    // "2025/10/12(日)"
}
```

### BusSchedule (NOUVEAU - structure complète)
```rust
struct BusSchedule {
    bus_number: String,      // "1401", "1341"
    route_name: String,      // "新宿～富士五湖線"

    departure_station: String,
    departure_date: String,   // "2025年10月13日"
    departure_time: String,   // "6:45"

    arrival_station: String,
    arrival_date: String,
    arrival_time: String,     // "8:30"

    way_no: u32,             // 3161
    available_plans: Vec<PricingPlan>,
}
```

### PricingPlan (NOUVEAU)
```rust
struct PricingPlan {
    plan_id: u32,            // discntPlanNo (27775, 27776...)
    plan_index: u32,         // data-index pour le parsing
    plan_name: String,       // "通常運賃（窓口・コンビニ購入用）"
    price: u32,              // 2200 (en yen)
    display_price: String,   // "2,200円"

    availability: SeatAvailability,

    # Détails (de ajaxPlanInfoPrint)
    details: Option<PlanDetails>,
}
```

### SeatAvailability (NOUVEAU)
```rust
enum SeatAvailability {
    Available { remaining_seats: Option<u32> },  // "残り1席", "残り8席"
    SoldOut,                                      // "満　席"
    Unknown,                                      // Pas de données
}

impl SeatAvailability {
    fn from_seat_value(value: u8, button_text: &str) -> Self {
        match value {
            1 => {
                let remaining = parse_remaining_seats(button_text);
                Self::Available { remaining_seats: remaining }
            },
            2 => Self::SoldOut,
            _ => Self::Unknown,
        }
    }
}

fn parse_remaining_seats(text: &str) -> Option<u32> {
    // "残り1席" -> Some(1)
    // "残り8席" -> Some(8)
    // "空席あり" -> None (juste disponible)
    // "満　席" -> utilisé avec value=2
}
```

### PlanDetails (NOUVEAU)
```rust
struct PlanDetails {
    # Conditions
    discount_note: String,
    applied_condition: String,
    way_change_note: String,
    repay_fee_note: String,
    point_note: String,
    settlement_limit: String,
    etc_note: String,
    link_url: String,

    # Équipements (flags)
    lady_only: bool,
    wireless_free: bool,
    wireless_pay: bool,
    tv: bool,
    power_plug: bool,
    blanket: bool,
    slipper: bool,
    pillow: bool,
    one_seat: bool,  # Siège indépendant
    relax_seat: bool,
    wide_seat: bool,
    drink: bool,
}
```

### PassengerCount (NOUVEAU)
```rust
struct PassengerCount {
    adult_men: u8,
    adult_women: u8,
    child_men: u8,
    child_women: u8,
    handicap_adult_men: u8,
    handicap_adult_women: u8,
    handicap_child_men: u8,
    handicap_child_women: u8,
}

impl PassengerCount {
    fn total_male(&self) -> u8 {
        self.adult_men + self.child_men + self.handicap_adult_men + self.handicap_child_men
    }

    fn total_female(&self) -> u8 {
        self.adult_women + self.child_women + self.handicap_adult_women + self.handicap_child_women
    }

    fn total(&self) -> u8 {
        self.total_male() + self.total_female()
    }

    fn validate(&self) -> Result<(), String> {
        if self.total() == 0 {
            return Err("Au moins 1 passager requis".into());
        }
        if self.total() > 12 {
            return Err("Maximum 12 passagers".into());
        }
        Ok(())
    }
}
```

---

## Section 5: Implémentation Rust Recommandée

### Nouvelles Fonctions à Ajouter

**Dans `src/scraper.rs`** :

```rust
use scraper::{Html, Selector};

impl BusScraper {
    /// Récupère horaires + disponibilité + prix pour une date
    pub async fn fetch_schedules(
        &self,
        request: &ScrapeRequest,
    ) -> Result<Vec<BusSchedule>> {
        let url = "https://www.highwaybus.com/gp/reservation/rsvPlanList";

        let response = self.client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", "https://www.highwaybus.com/")
            .query(&[
                ("mode", "search"),
                ("route", &request.area_id.to_string()),
                ("lineId", &request.route_id.to_string()),
                ("onStationCd", &request.departure_station),
                ("offStationCd", &request.arrival_station),
                ("bordingDate", &request.date),
                ("danseiNum", &request.passengers.total_male().to_string()),
                ("zyoseiNum", &request.passengers.total_female().to_string()),
                ("adultMen", &request.passengers.adult_men.to_string()),
                ("adultWomen", &request.passengers.adult_women.to_string()),
                ("childMen", &request.passengers.child_men.to_string()),
                ("childWomen", &request.passengers.child_women.to_string()),
                ("handicapAdultMen", &request.passengers.handicap_adult_men.to_string()),
                ("handicapAdultWomen", &request.passengers.handicap_adult_women.to_string()),
                ("handicapChildMen", &request.passengers.handicap_child_men.to_string()),
                ("handicapChildWomen", &request.passengers.handicap_child_women.to_string()),
            ])
            .send()
            .await?;

        let html = response.text().await?;
        self.parse_schedules_html(&html)
    }

    /// Parse le HTML pour extraire tous les horaires
    fn parse_schedules_html(&self, html: &str) -> Result<Vec<BusSchedule>> {
        let document = Html::parse_document(html);
        let mut schedules = Vec::new();

        // Sélecteur pour chaque bus
        let bus_selector = Selector::parse("section.busSvclistItem").unwrap();

        for (index, bus_element) in document.select(&bus_selector).enumerate() {
            let schedule = self.parse_single_bus(bus_element, index + 1, html)?;
            schedules.push(schedule);
        }

        Ok(schedules)
    }

    /// Parse un seul bus
    fn parse_single_bus(
        &self,
        element: scraper::ElementRef,
        index: usize,
        full_html: &str
    ) -> Result<BusSchedule> {
        // Extraire horaires
        let dep_time = self.extract_time(element, "dep")?;
        let arr_time = self.extract_time(element, "arr")?;

        // Extraire numéro de bus depuis modal
        let bus_number = self.extract_bus_number(full_html, index)?;

        // Extraire disponibilité et prix
        let available_plans = self.extract_pricing_plans(full_html, index)?;

        // Extraire wayNo
        let way_no = self.extract_way_no(full_html, index)?;

        Ok(BusSchedule {
            bus_number,
            route_name: "新宿～富士五湖線".to_string(), // Ou extraire du HTML
            departure_station: "バスタ新宿（南口）".to_string(),
            departure_date: self.extract_date(element, "dep")?,
            departure_time: dep_time,
            arrival_station: "河口湖駅".to_string(),
            arrival_date: self.extract_date(element, "arr")?,
            arrival_time: arr_time,
            way_no,
            available_plans,
        })
    }

    /// Extrait les plans tarifaires et leur disponibilité
    fn extract_pricing_plans(&self, html: &str, bus_index: usize) -> Result<Vec<PricingPlan>> {
        let document = Html::parse_document(html);
        let mut plans = Vec::new();

        // Chercher tous les hidden inputs seat_N
        let seat_selector = Selector::parse(&format!("input.seat_{}", bus_index)).unwrap();

        for seat_input in document.select(&seat_selector) {
            let plan_index: u32 = seat_input
                .value()
                .attr("data-index")
                .and_then(|s| s.parse().ok())
                .ok_or("Missing data-index")?;

            let seat_value: u8 = seat_input
                .value()
                .attr("value")
                .and_then(|s| s.parse().ok())
                .ok_or("Missing value")?;

            // Si disponible (value=1), chercher les détails
            if seat_value == 1 {
                let plan = self.extract_plan_details(html, plan_index)?;
                plans.push(plan);
            }
        }

        Ok(plans)
    }

    /// Récupère les détails d'un plan tarifaire
    pub async fn fetch_plan_details(&self, plan_id: u32) -> Result<PlanDetails> {
        let url = "https://www.highwaybus.com/gp/reservation/ajaxPlanInfoPrint";

        let response = self.client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", "https://www.highwaybus.com/gp/reservation/rsvPlanList")
            .query(&[
                ("discntPlanNo", plan_id.to_string()),
                ("reference", "reservation".to_string()),
            ])
            .send()
            .await?;

        let xml = response.text().await?;
        self.parse_plan_details_xml(&xml)
    }

    /// Parse le XML des détails de plan
    fn parse_plan_details_xml(&self, xml: &str) -> Result<PlanDetails> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut details = PlanDetails::default();
        let mut current_tag = String::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                }
                Event::Text(e) => {
                    let text = e.unescape()?.to_string();
                    match current_tag.as_str() {
                        "discntNote" => details.discount_note = text,
                        "appliedCondNote" => details.applied_condition = text,
                        "wayChangeNote" => details.way_change_note = text,
                        "repayFeeNote" => details.repay_fee_note = text,
                        "pointNote" => details.point_note = text,
                        "settlementLimitNote" => details.settlement_limit = text,
                        "etcNote" => details.etc_note = text,
                        "linkUrl" => details.link_url = text,
                        "ladyOnlyFlg" => details.lady_only = text == "1",
                        "wirelessFreeFlg" => details.wireless_free = text == "1",
                        "wirelessPayFlg" => details.wireless_pay = text == "1",
                        "tvFlg" => details.tv = text == "1",
                        "plugFlg" => details.power_plug = text == "1",
                        "blanketFlg" => details.blanket = text == "1",
                        "slipperFlg" => details.slipper = text == "1",
                        "pillowFlg" => details.pillow = text == "1",
                        "oneSeatFlg" => details.one_seat = text == "1",
                        "relaxFlg" => details.relax_seat = text == "1",
                        "wideSeatFlg" => details.wide_seat = text == "1",
                        "drinkFlg" => details.drink = text == "1",
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(details)
    }
}
```

### Nouvelles Structures dans `src/types.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassengerCount {
    pub adult_men: u8,
    pub adult_women: u8,
    pub child_men: u8,
    pub child_women: u8,
    pub handicap_adult_men: u8,
    pub handicap_adult_women: u8,
    pub handicap_child_men: u8,
    pub handicap_child_women: u8,
}

impl Default for PassengerCount {
    fn default() -> Self {
        Self {
            adult_men: 1,
            adult_women: 0,
            child_men: 0,
            child_women: 0,
            handicap_adult_men: 0,
            handicap_adult_women: 0,
            handicap_child_men: 0,
            handicap_child_women: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusSchedule {
    pub bus_number: String,
    pub route_name: String,
    pub departure_station: String,
    pub departure_date: String,
    pub departure_time: String,
    pub arrival_station: String,
    pub arrival_date: String,
    pub arrival_time: String,
    pub way_no: u32,
    pub available_plans: Vec<PricingPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingPlan {
    pub plan_id: u32,
    pub plan_index: u32,
    pub plan_name: String,
    pub price: u32,
    pub display_price: String,
    pub availability: SeatAvailability,
    pub details: Option<PlanDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeatAvailability {
    Available { remaining_seats: Option<u32> },
    SoldOut,
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanDetails {
    pub discount_note: String,
    pub applied_condition: String,
    pub way_change_note: String,
    pub repay_fee_note: String,
    pub point_note: String,
    pub settlement_limit: String,
    pub etc_note: String,
    pub link_url: String,

    pub lady_only: bool,
    pub wireless_free: bool,
    pub wireless_pay: bool,
    pub tv: bool,
    pub power_plug: bool,
    pub blanket: bool,
    pub slipper: bool,
    pub pillow: bool,
    pub one_seat: bool,
    pub relax_seat: bool,
    pub wide_seat: bool,
    pub drink: bool,
}
```

### Modifications de `src/config.rs`

```rust
#[derive(Debug, Clone)]
pub struct ScrapeRequest {
    pub area_id: u32,
    pub route_id: u32,
    pub departure_station: String,
    pub arrival_station: String,
    pub date: String,  // YYYYMMDD
    pub passengers: PassengerCount,  // NOUVEAU
}

impl ScrapeRequest {
    pub fn from_env() -> Result<Self> {
        // ... chargement existant ...

        let passengers = PassengerCount {
            adult_men: env::var("ADULT_MEN")?.parse().unwrap_or(1),
            adult_women: env::var("ADULT_WOMEN")?.parse().unwrap_or(0),
            child_men: env::var("CHILD_MEN")?.parse().unwrap_or(0),
            child_women: env::var("CHILD_WOMEN")?.parse().unwrap_or(0),
            handicap_adult_men: env::var("HANDICAP_ADULT_MEN")?.parse().unwrap_or(0),
            handicap_adult_women: env::var("HANDICAP_ADULT_WOMEN")?.parse().unwrap_or(0),
            handicap_child_men: env::var("HANDICAP_CHILD_MEN")?.parse().unwrap_or(0),
            handicap_child_women: env::var("HANDICAP_CHILD_WOMEN")?.parse().unwrap_or(0),
        };

        Ok(Self {
            area_id,
            route_id,
            departure_station,
            arrival_station,
            date: "".to_string(),  // Calculé dynamiquement
            passengers,
        })
    }
}
```

### Dépendances Additionnelles pour `Cargo.toml`

```toml
[dependencies]
scraper = "0.17"  # Pour parser le HTML
```

---

## Section 6: Edge Cases et Limitations

### Cas Particuliers Découverts

**1. Disponibilité par plan tarifaire** :
- Un même bus peut avoir plusieurs plans (通常運賃, ＷＥＢ割運賃)
- Chaque plan peut avoir une disponibilité différente
- Un plan peut être complet (`value="2"`) alors qu'un autre est disponible (`value="1"`)

**2. Nombre de passagers impact la disponibilité** :
- Le système filtre les plans selon le nombre de passagers
- Si on cherche 2 passagers, seuls les plans avec ≥2 sièges sont marqués `value="1"`
- La recherche avec 1 passager peut montrer plus de plans disponibles

**3. Stations proches** :
- `nearbyStationFlg=1` indique des stations alternatives
- Paramètres `nearCheckOnStation` et `nearCheckOffStation` activent cette recherche
- Augmente les résultats mais complexifie le parsing

**4. Bus multi-segments** :
- Certains bus ont des arrêts intermédiaires avec tarifs différents
- Chaque segment a son propre `wayNo`
- Le système ne gère pas les correspondances automatiques

**5. Dates et horaires** :
- Les dates disponibles sont retournées sur ~30 jours glissants
- Les horaires peuvent varier selon le jour de semaine (weekday vs weekend)
- Bus spéciaux (深夜便 = late night) avec numérotation différente

**6. Limite de passagers** :
- Maximum 12 passagers par réservation web
- Au-delà, contact téléphonique requis
- Enfants < 6 ans peuvent être gratuits si non assis

**7. Prix variables** :
- Plans tarifs multiples : normal, web discount, early bird, etc.
- Le prix peut changer selon la date de réservation
- Pas de prix de groupe automatique

### Limitations de l'API

**1. Pas d'endpoint temps réel pour disponibilité** :
- Obligation de charger toute la page HTML
- Pas de streaming des updates
- Polling nécessaire pour monitoring continu

**2. Format HTML non structuré** :
- Pas de JSON/XML pour les horaires
- Parsing HTML fragile (peut casser si design change)
- Indices CSS (`busSvclistItem_1`, `seat_1`) non garantis

**3. Rate limiting non documenté** :
- Pas de headers `X-RateLimit-*`
- Comportement 503 aléatoire sous charge
- Recommandation : max 1 req/5 secondes

**4. Session cookies** :
- Nécessaire de maintenir la session
- Cookie store requis dans le client HTTP
- Expiration non documentée

**5. Pas d'API de réservation complète** :
- L'endpoint 7 (`rsvPlanSelected`) nécessite interaction navigateur
- Sélection de sièges graphique seulement
- Paiement impossible via API

**6. Données non exhaustives** :
- Pas d'historique de prix
- Pas de prévisions de disponibilité
- Pas de métriques sur la fréquentation

### Recommandations pour Scraping Production

**1. Caching agressif** :
```rust
// Cache les routes/stations (changent rarement)
let routes_cache_ttl = Duration::from_secs(86400); // 24h

// Cache les dates disponibles (change quotidiennement)
let dates_cache_ttl = Duration::from_secs(3600); // 1h

// NE PAS cacher les horaires/disponibilité (temps réel)
```

**2. Rate limiting** :
```rust
use tokio::time::{sleep, Duration};

const MIN_REQUEST_INTERVAL: Duration = Duration::from_secs(5);

async fn throttled_request(&self) {
    sleep(MIN_REQUEST_INTERVAL).await;
    // ... faire la requête
}
```

**3. Monitoring** :
```rust
// Logger tous les changements de disponibilité
if previous_availability != current_availability {
    tracing::info!(
        bus_number = %schedule.bus_number,
        previous = ?previous_availability,
        current = ?current_availability,
        "Availability changed"
    );

    // Alerter l'utilisateur si un siège se libère
    if matches!(current_availability, SeatAvailability::Available { .. }) {
        send_notification(&schedule).await;
    }
}
```

**4. Fallback sur erreurs** :
```rust
// Si HTML parsing échoue, ne pas crash
match parse_schedules_html(html) {
    Ok(schedules) => schedules,
    Err(e) => {
        tracing::error!("HTML parsing failed: {}", e);
        // Retourner cache ou données partielles
        return get_cached_schedules().await;
    }
}
```

**5. Health checks** :
```rust
// Endpoint /health pour monitoring
async fn health_check(scraper: &BusScraper) -> bool {
    // Test avec une requête simple
    scraper.fetch_routes(1).await.is_ok()
}
```

**6. Gestion de la concurrence** :
```rust
use tokio::sync::Semaphore;

// Limiter à 3 requêtes simultanées
let semaphore = Arc::new(Semaphore::new(3));

async fn fetch_with_semaphore(&self, request: &ScrapeRequest) -> Result<Vec<BusSchedule>> {
    let _permit = self.semaphore.acquire().await?;
    self.fetch_schedules(request).await
}
```

---

## Résumé Exécutif

### Ce qui a été découvert

**✅ Endpoints API complets** :
1. 4 endpoints AJAX (XML) pour navigation hiérarchique
2. 1 endpoint GET (HTML) pour horaires + disponibilité + prix
3. 1 endpoint GET (XML) pour détails des plans tarifaires
4. 1 endpoint POST pour réservation (partiel - nécessite session)

**✅ Données complètes disponibles** :
- Routes et stations (IDs + noms)
- Dates disponibles (30 jours glissants)
- **Horaires précis** (départ/arrivée avec minutes)
- **Disponibilité en temps réel** (sièges restants par plan)
- **Prix par plan tarifaire** (normal, web discount, etc.)
- Détails des plans (conditions, équipements bus)

**✅ Workflow opérationnel** :
- Scraping possible sans login
- Headers User-Agent + Referer obligatoires
- Cookie store nécessaire
- Parsing HTML requis pour horaires/disponibilité

### Ce qui manque

**❌ Pas d'API temps réel dédiée** :
- Obligation de parser HTML
- Pas de WebSocket ou Server-Sent Events
- Polling manuel nécessaire

**❌ Pas de réservation automatique complète** :
- Sélection sièges graphique uniquement
- Paiement nécessite interaction navigateur
- API réservation partielle

**❌ Pas d'historique ou analytics** :
- Pas de données historiques de prix
- Pas de prévisions de disponibilité
- Pas de statistiques de fréquentation

### Pour le use case Shinjuku → Kawaguchiko

**Données obtenues** :
- ✅ Tous les horaires du jour (6:45, 6:55, 7:15, etc.)
- ✅ Disponibilité exacte ("残り1席", "残り8席", "満　席")
- ✅ Prix par plan (2200円 normal, 2000円 web discount)
- ✅ Numéros de bus (1401, 1341, etc.)
- ✅ Temps de trajet (départ 6:45 → arrivée 8:30)

**Implémentation recommandée** :
```rust
// Monitoring toutes les 5 minutes
let mut interval = tokio::time::interval(Duration::from_secs(300));

loop {
    interval.tick().await;

    let schedules = scraper.fetch_schedules(&ScrapeRequest {
        area_id: 1,
        route_id: 110,
        departure_station: "001".to_string(),
        arrival_station: "064".to_string(),
        date: today_yyyymmdd(),
        passengers: PassengerCount::default(),
    }).await?;

    for schedule in schedules {
        for plan in schedule.available_plans {
            if matches!(plan.availability, SeatAvailability::Available { .. }) {
                tracing::info!(
                    "🎫 Seat available: Bus {} at {} - {} - {} yen",
                    schedule.bus_number,
                    schedule.departure_time,
                    plan.plan_name,
                    plan.price
                );
            }
        }
    }
}
```
