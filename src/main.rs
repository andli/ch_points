use reqwest;
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
    // Assuming `cards` is a map with keys as card identifiers and values as `Card` structs
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
    // Get deck ID from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: cargo run <moxfield_deck_id>".into());
    }
    let deck_id = &args[1];

    // URL to the Moxfield deck API
    let url = format!("https://api2.moxfield.com/v3/decks/all/{}", deck_id);

    // Send GET request to the Moxfield API
    let client = reqwest::blocking::Client::new();
    let response = client.get(&url).send()?;
    let deck: Deck = response.json()?;

    // Access the mainboard directly using the key
    let mainboard = deck.boards.get("mainboard").ok_or("Mainboard not found")?;

    // Fetch Canadian Highlander points list
    let points_url = "https://www.canadianhighlander.ca/points-list/";
    let points_html = client.get(points_url).send()?.text()?;
    let points_doc = scraper::Html::parse_document(&points_html);
    let selector = scraper::Selector::parse("tr").unwrap();
    let mut points_map = HashMap::new();
    for element in points_doc.select(&selector) {
        let td_elements: Vec<_> = element
            .select(&scraper::Selector::parse("td").unwrap())
            .collect();
        if td_elements.len() >= 2 {
            let card_name = td_elements[0].text().collect::<Vec<_>>().join("");
            let points = td_elements[1]
                .text()
                .collect::<String>()
                .parse::<u8>()
                .unwrap_or(0);
            points_map.insert(card_name.trim().to_string(), points);
        }
    }

    // Calculate total points for the deck
    let mut total_points = 0;
    let mut pointed_cards = Vec::new();
    for (_, card) in &mainboard.cards {
        if let Some(&points) = points_map.get(&card.card.name) {
            if points > 0 {
                pointed_cards.push((card.card.name.clone(), points));
                total_points += points;
            }
        }
    }
    // Sort by points value
    pointed_cards.sort_by(|a, b| b.1.cmp(&a.1));

    // Print pointed cards in the desired format
    let pointed_cards_formatted: Vec<String> = pointed_cards
        .iter()
        .map(|(card_name, points)| format!("{} [{}]", card_name, points))
        .collect();
    println!(
        "Points: {} ({})",
        total_points,
        pointed_cards_formatted.join(", ")
    );

    Ok(())
}
