use crate::error::{Result, ScraperError};
use crate::types::{BusSchedule, PricingPlan, SeatAvailability};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use tracing::debug;

pub fn parse_schedules_html(html: &str, boarding_date: &str) -> Result<Vec<BusSchedule>> {
    let document = Html::parse_document(html);
    let bus_selector = Selector::parse("section.busSvclistItem")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    let mut schedules = Vec::new();
    for (index, bus_element) in document.select(&bus_selector).enumerate() {
        match parse_single_bus(bus_element, index + 1, html, boarding_date) {
            Ok(schedule) => {
                schedules.push(schedule);
            }
            Err(e) => {
                debug!("Failed to parse bus {}: {}", index + 1, e);
            }
        }
    }

    Ok(schedules)
}

fn parse_single_bus(
    element: ElementRef,
    bus_index: usize,
    _full_html: &str,
    boarding_date: &str,
) -> Result<BusSchedule> {
    let departure_time = extract_time(element, "dep")?;
    let arrival_time = extract_time(element, "arr")?;
    let available_plans = extract_plans_from_bus(element)?;

    Ok(BusSchedule {
        bus_number: format!("Bus_{bus_index}"),
        route_name: String::new(),
        departure_station: String::new(),
        departure_date: boarding_date.to_string(),
        departure_time,
        arrival_station: String::new(),
        arrival_date: String::new(),
        arrival_time,
        way_no: 0,
        available_plans,
    })
}

pub fn extract_time(element: ElementRef, dep_or_arr: &str) -> Result<String> {
    let class_selector = format!("li.{dep_or_arr} p.time");
    let selector = Selector::parse(&class_selector)
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    let time_element = element
        .select(&selector)
        .next()
        .ok_or_else(|| ScraperError::Parse(format!("Time element not found for {dep_or_arr}")))?;

    let time_text = time_element.text().collect::<String>();
    extract_time_from_text(&time_text)
}

fn extract_time_from_text(text: &str) -> Result<String> {
    let re = Regex::new(r"(\d{1,2}:\d{2})")
        .map_err(|e| ScraperError::Parse(format!("Regex error: {e}")))?;

    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| ScraperError::Parse(format!("Time not found in text: {text}")))
}

fn extract_plans_from_bus(bus_element: ElementRef) -> Result<Vec<PricingPlan>> {
    let mut plans = Vec::new();

    let form_selector = Selector::parse("form[name='selectPlan']")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    let seat_selector = Selector::parse("input[type='hidden'][class*='seat_']")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    for form in bus_element.select(&form_selector) {
        if let Some(seat_input) = form.select(&seat_selector).next() {
            let seat_value = extract_value_attribute(seat_input).unwrap_or(2);

            if seat_value == 1
                && let Ok(plan) = extract_plan_from_form(form)
            {
                plans.push(plan);
            }
        }
    }

    Ok(plans)
}

fn extract_value_attribute(element: ElementRef) -> Result<u8> {
    element
        .value()
        .attr("value")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ScraperError::Parse("Missing or invalid value attribute".to_string()))
}

fn extract_plan_from_form(form: ElementRef) -> Result<PricingPlan> {
    let input_selector = Selector::parse("input[name='discntPlanNo']")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    let plan_id = form
        .select(&input_selector)
        .next()
        .and_then(|input| input.value().attr("value"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let button_selector = Selector::parse("button")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    let button_text = form
        .select(&button_selector)
        .next()
        .map(|btn| btn.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let remaining = parse_remaining_seats(&button_text);

    let price = extract_price_from_form(form).unwrap_or(0);

    let seat_selector = Selector::parse("input[type='hidden'][class*='seat_']")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    let plan_index = form
        .select(&seat_selector)
        .next()
        .and_then(|input| input.value().attr("data-index"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    Ok(PricingPlan {
        plan_id,
        plan_index,
        plan_name: String::new(),
        price,
        display_price: if price > 0 {
            format!("{price}円")
        } else {
            String::new()
        },
        availability: SeatAvailability::Available {
            remaining_seats: remaining,
        },
    })
}

fn extract_price_from_form(form: ElementRef) -> Result<u32> {
    let price_selector = Selector::parse("p.price")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {e:?}")))?;

    let re = Regex::new(r"(\d+,?\d*)円")
        .map_err(|e| ScraperError::Parse(format!("Regex error: {e}")))?;

    let mut current = form.parent();
    while let Some(parent) = current {
        if let Some(parent_elem) = ElementRef::wrap(parent)
            && let Some(price_elem) = parent_elem.select(&price_selector).next()
        {
            let price_text = price_elem.text().collect::<String>();

            if let Some(price) = re
                .captures(&price_text)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().replace(',', ""))
                .and_then(|s| s.parse().ok())
            {
                return Ok(price);
            }
        }
        current = parent.parent();
    }

    Err(ScraperError::Parse("Price element not found".to_string()))
}

pub fn parse_remaining_seats(button_text: &str) -> Option<u32> {
    let re = Regex::new(r"残り(\d+)席").ok()?;
    re.captures(button_text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // === parse_remaining_seats TESTS ===

    #[test]
    fn test_parse_remaining_seats_with_number() {
        assert_eq!(parse_remaining_seats("残り1席"), Some(1));
        assert_eq!(parse_remaining_seats("残り8席"), Some(8));
        assert_eq!(parse_remaining_seats("残り10席"), Some(10));
    }

    #[test]
    fn test_parse_remaining_seats_sold_out() {
        assert_eq!(parse_remaining_seats("満　席"), None);
        assert_eq!(parse_remaining_seats("満席"), None);
    }

    #[test]
    fn test_parse_remaining_seats_available() {
        assert_eq!(parse_remaining_seats("空席あり"), None);
        assert_eq!(parse_remaining_seats("予約"), None);
    }

    #[test]
    fn test_parse_remaining_seats_invalid() {
        assert_eq!(parse_remaining_seats(""), None);
        assert_eq!(parse_remaining_seats("invalid"), None);
    }

    // === extract_time TESTS ===

    #[test]
    fn test_extract_time() {
        let html = r#"
            <section>
                <li class="dep">
                    <p class="time">6:45 発</p>
                </li>
            </section>
        "#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("section").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert_eq!(extract_time(element, "dep").unwrap(), "6:45");
    }

    #[test]
    fn test_extract_time_arrival() {
        let html = r#"
            <section>
                <li class="arr">
                    <p class="time">8:30 着</p>
                </li>
            </section>
        "#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("section").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert_eq!(extract_time(element, "arr").unwrap(), "8:30");
    }

    #[test]
    fn test_extract_time_double_digit_hour() {
        let html = r#"
            <section>
                <li class="dep">
                    <p class="time">12:30 発</p>
                </li>
            </section>
        "#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("section").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert_eq!(extract_time(element, "dep").unwrap(), "12:30");
    }

    #[test]
    fn test_extract_time_missing_returns_error() {
        let html = r#"<section><li class="dep"></li></section>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("section").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert!(extract_time(element, "dep").is_err());
    }

    // === extract_time_from_text TESTS ===

    #[test]
    fn test_extract_time_from_text_with_suffix() {
        assert_eq!(extract_time_from_text("6:45 発").unwrap(), "6:45");
        assert_eq!(extract_time_from_text("8:30 着").unwrap(), "8:30");
    }

    #[test]
    fn test_extract_time_from_text_with_whitespace() {
        assert_eq!(extract_time_from_text("  9:00  発  ").unwrap(), "9:00");
    }

    #[test]
    fn test_extract_time_from_text_no_time_returns_error() {
        assert!(extract_time_from_text("発").is_err());
        assert!(extract_time_from_text("").is_err());
    }

    // === parse_schedules_html TESTS ===

    #[test]
    fn test_parse_schedules_html_single_bus() {
        let html = r#"
            <html><body>
                <section class="busSvclistItem">
                    <ul>
                        <li class="dep"><p class="time">6:45 発</p></li>
                        <li class="arr"><p class="time">8:30 着</p></li>
                    </ul>
                    <div class="planArea">
                        <p class="price">12,000円</p>
                        <form name="selectPlan">
                            <input type="hidden" class="seat_0" value="1" data-index="0">
                            <input type="hidden" name="discntPlanNo" value="12345">
                            <button>残り3席</button>
                        </form>
                    </div>
                </section>
            </body></html>
        "#;

        let schedules = parse_schedules_html(html, "20251029").unwrap();
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].departure_time, "6:45");
        assert_eq!(schedules[0].arrival_time, "8:30");
        assert_eq!(schedules[0].departure_date, "20251029");
        assert_eq!(schedules[0].available_plans.len(), 1);
        assert_eq!(schedules[0].available_plans[0].plan_id, 12345);
        assert_eq!(schedules[0].available_plans[0].price, 12000);
    }

    #[test]
    fn test_parse_schedules_html_multiple_buses() {
        let html = r#"
            <html><body>
                <section class="busSvclistItem">
                    <ul>
                        <li class="dep"><p class="time">6:45 発</p></li>
                        <li class="arr"><p class="time">8:30 着</p></li>
                    </ul>
                    <div class="planArea">
                        <p class="price">12,000円</p>
                        <form name="selectPlan">
                            <input type="hidden" class="seat_0" value="1" data-index="0">
                            <input type="hidden" name="discntPlanNo" value="12345">
                            <button>残り3席</button>
                        </form>
                    </div>
                </section>
                <section class="busSvclistItem">
                    <ul>
                        <li class="dep"><p class="time">9:00 発</p></li>
                        <li class="arr"><p class="time">11:15 着</p></li>
                    </ul>
                    <div class="planArea">
                        <p class="price">10,500円</p>
                        <form name="selectPlan">
                            <input type="hidden" class="seat_0" value="1" data-index="0">
                            <input type="hidden" name="discntPlanNo" value="12346">
                            <button>残り8席</button>
                        </form>
                    </div>
                </section>
            </body></html>
        "#;

        let schedules = parse_schedules_html(html, "20251029").unwrap();
        assert_eq!(schedules.len(), 2);
        assert_eq!(schedules[0].departure_time, "6:45");
        assert_eq!(schedules[1].departure_time, "9:00");
    }

    #[test]
    fn test_parse_schedules_html_no_buses() {
        let html = r"<html><body><div>No buses available</div></body></html>";

        let schedules = parse_schedules_html(html, "20251029").unwrap();
        assert!(schedules.is_empty());
    }

    #[test]
    fn test_parse_schedules_html_sold_out_excluded() {
        let html = r#"
            <html><body>
                <section class="busSvclistItem">
                    <ul>
                        <li class="dep"><p class="time">12:30 発</p></li>
                        <li class="arr"><p class="time">14:45 着</p></li>
                    </ul>
                    <div class="planArea">
                        <p class="price">9,800円</p>
                        <form name="selectPlan">
                            <input type="hidden" class="seat_0" value="2" data-index="0">
                            <input type="hidden" name="discntPlanNo" value="12347">
                            <button>満席</button>
                        </form>
                    </div>
                </section>
            </body></html>
        "#;

        let schedules = parse_schedules_html(html, "20251029").unwrap();
        assert_eq!(schedules.len(), 1);
        // Sold out plans (seat_0 value="2") are excluded
        assert!(schedules[0].available_plans.is_empty());
    }

    // === extract_plans_from_bus TESTS ===

    #[test]
    fn test_extract_plans_from_bus_available() {
        let html = r#"
            <section class="busSvclistItem">
                <div class="planArea">
                    <p class="price">12,000円</p>
                    <form name="selectPlan">
                        <input type="hidden" class="seat_0" value="1" data-index="0">
                        <input type="hidden" name="discntPlanNo" value="12345">
                        <button>残り3席</button>
                    </form>
                </div>
            </section>
        "#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("section.busSvclistItem").unwrap();
        let element = document.select(&selector).next().unwrap();

        let plans = extract_plans_from_bus(element).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].plan_id, 12345);
        assert_eq!(plans[0].price, 12000);
        match &plans[0].availability {
            SeatAvailability::Available { remaining_seats } => {
                assert_eq!(*remaining_seats, Some(3));
            }
        }
    }

    #[test]
    fn test_extract_plans_from_bus_sold_out_excluded() {
        let html = r#"
            <section class="busSvclistItem">
                <div class="planArea">
                    <p class="price">9,800円</p>
                    <form name="selectPlan">
                        <input type="hidden" class="seat_0" value="2" data-index="0">
                        <input type="hidden" name="discntPlanNo" value="12347">
                        <button>満席</button>
                    </form>
                </div>
            </section>
        "#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("section.busSvclistItem").unwrap();
        let element = document.select(&selector).next().unwrap();

        let plans = extract_plans_from_bus(element).unwrap();
        assert!(plans.is_empty());
    }

    // === extract_value_attribute TESTS ===

    #[test]
    fn test_extract_value_attribute_valid() {
        let html = r#"<input type="hidden" value="5">"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("input").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert_eq!(extract_value_attribute(element).unwrap(), 5);
    }

    #[test]
    fn test_extract_value_attribute_missing() {
        let html = r#"<input type="hidden">"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("input").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert!(extract_value_attribute(element).is_err());
    }

    #[test]
    fn test_extract_value_attribute_invalid_number() {
        let html = r#"<input type="hidden" value="abc">"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("input").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert!(extract_value_attribute(element).is_err());
    }
}
