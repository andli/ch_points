use ch_points::api;
use mockito;
use reqwest::blocking::Client;
use serde_json::json;

#[test]
fn test_fetch_deck_data_success() {
    let _m = mockito::mock("GET", "/v3/decks/all/sample_deck_id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "boards": {
                    "mainboard": {
                        "cards": {
                            "EbPg6": {
                                "card": {"name": "Black Lotus"}
                            },
                            "Xdf8ws": {
                                "card": {"name": "Mox Jet"}
                            }
                        }
                    }
                }
            })
            .to_string(),
        )
        .create();

    // Instantiate the HTTP client.
    let client = Client::new();
    let deck_id = "sample_deck_id";
    let mock_url = mockito::server_url();

    // Updated test call to include mock_url as the base URL argument.
    let response = api::fetch_deck_data(&client, &mock_url, deck_id);

    // Assertions...
    assert!(response.is_ok());
    let deck = response.expect("Failed to fetch deck");
    assert!(deck.boards.contains_key("mainboard"));
    let mainboard = deck.boards.get("mainboard").unwrap();
    assert_eq!(
        mainboard.cards.get("EbPg6").unwrap().card.name,
        "Black Lotus"
    );
    assert_eq!(mainboard.cards.get("Xdf8ws").unwrap().card.name, "Mox Jet");
}
