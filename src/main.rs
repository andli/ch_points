use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::io::{self, Write};

use ch_points::api::{fetch_deck_data, fetch_points_list, Deck};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--list".to_string()) {
        return list_pointed_cards();
    }

    let deck_id = parse_deck_id()?;
    let client = reqwest::blocking::Client::new();
    let base_url = "https://api2.moxfield.com/";
    let deck = fetch_deck_data(&client, &base_url, &deck_id)?;
    let points_map = fetch_points_list(&client)?;
    let (total_points, pointed_cards_formatted) = calculate_deck_points(&deck, &points_map);

    let print_str = format!(
        "Points: {} ({})",
        total_points,
        pointed_cards_formatted.join(", ")
    );
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(print_str.to_owned()).unwrap();
    println!("{}", print_str);

    Ok(())
}

fn list_pointed_cards() -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let points_map = fetch_points_list(&client)?;
    for (card, points) in points_map {
        println!("{}\t{}", points, card);
    }
    Ok(())
}

fn parse_deck_id() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // Prompt user for the deck ID
        println!("Please enter the Moxfield deck ID:");
        io::stdout().flush()?; // Ensure the prompt is displayed immediately
        let mut deck_id = String::new();
        io::stdin().read_line(&mut deck_id)?;
        // Trim whitespace and check if the input is not empty
        let deck_id = deck_id.trim();
        if deck_id.is_empty() {
            return Err("No deck ID provided. Usage: cargo run <moxfield_deck_id>".into());
        }
        return Ok(deck_id.to_string());
    }
    Ok(args[1].clone())
}

fn calculate_deck_points(deck: &Deck, points_map: &BTreeMap<String, u8>) -> (i32, Vec<String>) {
    let mainboard = deck.boards.get("mainboard").expect("Mainboard not found");
    let mut total_points: u8 = 0;
    let mut pointed_cards = Vec::new();
    //println!("Mainboard:");
    for (_, card) in &mainboard.cards {
        //println!("{}", &card.card.name);
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
