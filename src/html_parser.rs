use crate::error::{Result, ScraperError};
use crate::types::{BusSchedule, PricingPlan, SeatAvailability};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

pub fn parse_schedules_html(html: &str) -> Result<Vec<BusSchedule>> {
    let document = Html::parse_document(html);
    let bus_selector = Selector::parse("section.busSvclistItem")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {:?}", e)))?;

    let mut schedules = Vec::new();
    for (index, bus_element) in document.select(&bus_selector).enumerate() {
        let schedule = parse_single_bus(bus_element, index + 1, html)?;
        schedules.push(schedule);
    }

    Ok(schedules)
}

fn parse_single_bus(element: ElementRef, bus_index: usize, full_html: &str) -> Result<BusSchedule> {
    let departure_time = extract_time(element, "dep")?;
    let arrival_time = extract_time(element, "arr")?;
    let available_plans = extract_availability(full_html, bus_index)?;

    Ok(BusSchedule {
        bus_number: format!("Bus_{}", bus_index),
        route_name: String::new(),
        departure_station: String::new(),
        departure_date: String::new(),
        departure_time,
        arrival_station: String::new(),
        arrival_date: String::new(),
        arrival_time,
        way_no: 0,
        available_plans,
    })
}

pub fn extract_time(element: ElementRef, dep_or_arr: &str) -> Result<String> {
    let class_selector = format!("li.{} p.time", dep_or_arr);
    let selector = Selector::parse(&class_selector)
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {:?}", e)))?;

    let time_element = element
        .select(&selector)
        .next()
        .ok_or_else(|| ScraperError::Parse(format!("Time element not found for {}", dep_or_arr)))?;

    let time_text = time_element.text().collect::<String>();
    extract_time_from_text(&time_text)
}

fn extract_time_from_text(text: &str) -> Result<String> {
    let re = Regex::new(r"(\d{1,2}:\d{2})")
        .map_err(|e| ScraperError::Parse(format!("Regex error: {}", e)))?;

    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| ScraperError::Parse(format!("Time not found in text: {}", text)))
}

pub fn extract_availability(html: &str, bus_index: usize) -> Result<Vec<PricingPlan>> {
    let document = Html::parse_document(html);
    let mut plans = Vec::new();

    let seat_class = format!("input.seat_{}", bus_index);
    let seat_selector = Selector::parse(&seat_class)
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {:?}", e)))?;

    for seat_input in document.select(&seat_selector) {
        let plan_index = extract_data_attribute(seat_input, "data-index")?;
        let seat_value = extract_value_attribute(seat_input)?;

        if seat_value == 1 {
            if let Ok(plan) = extract_plan_details(&document, bus_index, plan_index) {
                plans.push(plan);
            }
        }
    }

    Ok(plans)
}

fn extract_data_attribute(element: ElementRef, attr: &str) -> Result<u32> {
    element
        .value()
        .attr(attr)
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ScraperError::Parse(format!("Missing or invalid {}", attr)))
}

fn extract_value_attribute(element: ElementRef) -> Result<u8> {
    element
        .value()
        .attr("value")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ScraperError::Parse("Missing or invalid value attribute".to_string()))
}

fn extract_plan_details(document: &Html, bus_index: usize, plan_index: u32) -> Result<PricingPlan> {
    let price = extract_price(document, bus_index, plan_index)?;
    let (plan_id, button_text) = extract_plan_form_data(document, bus_index, plan_index)?;
    let remaining = parse_remaining_seats(&button_text);

    Ok(PricingPlan {
        plan_id,
        plan_index,
        plan_name: String::new(),
        price,
        display_price: format!("{}", price),
        availability: SeatAvailability::Available {
            remaining_seats: remaining,
        },
    })
}

fn extract_price(document: &Html, bus_index: usize, plan_index: u32) -> Result<u32> {
    let price_class = format!("input.price_{}", bus_index);
    let price_selector = Selector::parse(&price_class)
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {:?}", e)))?;

    for price_input in document.select(&price_selector) {
        let data_index = price_input.value().attr("data-index");
        if data_index == Some(&plan_index.to_string()) {
            return price_input
                .value()
                .attr("value")
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| ScraperError::Parse("Invalid price value".to_string()));
        }
    }

    Err(ScraperError::Parse(format!(
        "Price not found for bus {} plan {}",
        bus_index, plan_index
    )))
}

fn extract_plan_form_data(
    document: &Html,
    bus_index: usize,
    plan_index: u32,
) -> Result<(u32, String)> {
    let form_id = format!("form_{}_{}", bus_index, plan_index);
    let form_selector = Selector::parse(&format!("form#{}", form_id))
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {:?}", e)))?;

    let form = document
        .select(&form_selector)
        .next()
        .ok_or_else(|| ScraperError::Parse(format!("Form not found: {}", form_id)))?;

    let input_selector = Selector::parse("input[name='discntPlanNo']")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {:?}", e)))?;

    let plan_id = form
        .select(&input_selector)
        .next()
        .and_then(|input| input.value().attr("value"))
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ScraperError::Parse("Plan ID not found".to_string()))?;

    let button_selector = Selector::parse("button")
        .map_err(|e| ScraperError::Parse(format!("Invalid selector: {:?}", e)))?;

    let button_text = form
        .select(&button_selector)
        .next()
        .map(|btn| btn.text().collect::<String>())
        .unwrap_or_default();

    Ok((plan_id, button_text))
}

pub fn parse_remaining_seats(button_text: &str) -> Option<u32> {
    let re = Regex::new(r"残り(\d+)席").ok()?;
    re.captures(button_text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_extract_availability_with_available_seats() {
        let html = r#"
            <input type="hidden" class="seat_1" data-index="1" value="1">
            <input type="hidden" class="seat_1" data-index="2" value="2">
            <input type="hidden" class="price_1" data-index="1" value="2200">
            <form id="form_1_1">
                <input type="hidden" name="discntPlanNo" value="27775"/>
                <button>残り1席</button>
            </form>
        "#;

        let result = extract_availability(html, 1).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].plan_id, 27775);
        assert_eq!(result[0].price, 2200);
        assert!(matches!(
            result[0].availability,
            SeatAvailability::Available { .. }
        ));
    }

    #[test]
    fn test_extract_availability_sold_out() {
        let html = r#"
            <input type="hidden" class="seat_1" data-index="1" value="2">
            <input type="hidden" class="price_1" data-index="1" value="2200">
        "#;

        let result = extract_availability(html, 1).unwrap();
        assert_eq!(result.len(), 0); // Sold out plans are filtered
    }
}
