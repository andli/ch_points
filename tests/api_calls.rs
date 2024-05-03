use ch_points::api; // Import all necessary structs and functions from the api module
use ch_points::api::HttpError; // Import the HttpError directly if defined within the api module
use mockito;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_fetch_deck_data_failed() {
    let _m = mockito::mock("GET", "/v3/decks/all/sample_deck_id")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body("{\"error\": \"Deck not found\"}")
        .create();

    let client = Client::new();
    let deck_id = "sample_deck_id";
    let mock_url = mockito::server_url();

    let response = api::fetch_deck_data(&client, &mock_url, deck_id);
    assert!(response.is_err());

    // Expecting an error due to the 404 status
    let error = response.unwrap_err();
    if let Some(http_error) = error.downcast_ref::<HttpError>() {
        assert_eq!(http_error.status_code(), StatusCode::NOT_FOUND);
    } else {
        panic!(
            "Expected HttpError but found different error type: {}",
            error
        );
    }
}

#[test]
fn test_fetch_deck_data_success() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/moxfield-example.json");

    let json_data = fs::read_to_string(path).expect("Unable to read moxfield-example.json");

    let _m = mockito::mock("GET", "/v3/decks/all/sample_deck_id")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json_data)
        .create();

    let client = Client::new();
    let deck_id = "sample_deck_id";
    let mock_url = mockito::server_url();

    let response = api::fetch_deck_data(&client, &mock_url, deck_id);
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
