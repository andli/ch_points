use reqwest;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Deck {
    boards: HashMap<String, Board>,
}

#[derive(Deserialize, Debug)]
struct Board {
    cards: HashMap<String, Card>,
}

#[derive(Deserialize, Debug)]
struct Card {
    card: CardInfo,
}

#[derive(Deserialize, Debug)]
struct CardInfo {
    name: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let deck_id = parse_deck_id()?;
    let client = reqwest::blocking::Client::new();
    let deck = fetch_deck_data(&client, &deck_id)?;
    let points_map = fetch_points_list(&client)?;
    let (total_points, pointed_cards_formatted) = calculate_deck_points(&deck, &points_map);

    println!(
        "Points: {} ({})",
        total_points,
        pointed_cards_formatted.join(", ")
    );

    Ok(())
}

fn parse_deck_id() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: cargo run <moxfield_deck_id>".into());
    }
    Ok(args[1].clone())
}

fn fetch_deck_data(
    client: &reqwest::blocking::Client,
    deck_id: &str,
) -> Result<Deck, Box<dyn Error>> {
    let url = format!("https://api2.moxfield.com/v3/decks/all/{}", deck_id);
    let response = client.get(&url).send()?;
    let deck = response.json()?;
    Ok(deck)
}

fn fetch_points_list(
    client: &reqwest::blocking::Client,
) -> Result<HashMap<String, u8>, Box<dyn Error>> {
    let points_url = "https://www.canadianhighlander.ca/points-list/";
    let points_html = client.get(points_url).send()?.text()?;
    let points_doc = Html::parse_document(&points_html);
    let selector = Selector::parse("tr").unwrap();

    let mut points_map = HashMap::new();
    for element in points_doc.select(&selector) {
        let td_elements: Vec<_> = element.select(&Selector::parse("td").unwrap()).collect();
        if td_elements.len() >= 2 {
            let card_name = td_elements[0].text().collect::<Vec<_>>().join("");
            let points = td_elements[1]
                .text()
                .collect::<String>()
                .parse::<u8>()
                .unwrap_or_else(|_| {
                    eprintln!("Failed to parse points for a card.");
                    0
                });
            points_map.insert(card_name.trim().to_string(), points);
        }
    }
    Ok(points_map)
}

fn calculate_deck_points(deck: &Deck, points_map: &HashMap<String, u8>) -> (i32, Vec<String>) {
    let mainboard = deck.boards.get("mainboard").expect("Mainboard not found");
    let mut total_points: u8 = 0;
    let mut pointed_cards = Vec::new();
    for (_, card) in &mainboard.cards {
        if let Some(&points) = points_map.get(&card.card.name) {
            if points > 0 {
                pointed_cards.push((card.card.name.clone(), points));
                total_points += points;
            }
        }
    }
    pointed_cards.sort(); //sort alphabetically first
    pointed_cards.sort_by(|a, b| b.1.cmp(&a.1));
    let pointed_cards_formatted: Vec<String> = pointed_cards
        .iter()
        .map(|(card_name, points)| format!("{} [{}]", card_name, points))
        .collect();
    (total_points.into(), pointed_cards_formatted)
}
