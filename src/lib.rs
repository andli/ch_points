pub mod api {

    use reqwest::blocking::Client;
    use scraper::{Html, Selector};
    use serde::Deserialize;
    use std::collections::{BTreeMap, HashMap};
    use std::error::Error;

    #[derive(Deserialize, Debug)]
    pub struct Deck {
        pub boards: HashMap<String, Board>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Board {
        pub cards: HashMap<String, Card>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Card {
        pub card: CardInfo,
    }

    #[derive(Deserialize, Debug)]
    pub struct CardInfo {
        pub name: String,
    }

    #[derive(Debug)]
    pub struct HttpError(reqwest::StatusCode);

    impl HttpError {
        // Public method to get the status code
        pub fn status_code(&self) -> reqwest::StatusCode {
            self.0
        }
    }

    impl std::fmt::Display for HttpError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "HTTP Error: {}", self.0)
        }
    }

    impl Error for HttpError {}

    pub fn fetch_deck_data(
        client: &Client,
        base_url: &str,
        deck_id: &str,
    ) -> Result<Deck, Box<dyn Error>> {
        let url = format!("{}/v3/decks/all/{}", base_url, deck_id);
        let response = client.get(&url).send()?;

        if !response.status().is_success() {
            return Err(Box::new(HttpError(response.status())));
        }

        let deck = response.json()?;
        Ok(deck)
    }

    pub fn fetch_points_list(
        client: &reqwest::blocking::Client,
    ) -> Result<BTreeMap<String, u8>, Box<dyn Error>> {
        let points_url = "https://www.canadianhighlander.ca/points-list/";
        let points_html = client.get(points_url).send()?.text()?;
        let points_doc = Html::parse_document(&points_html);
        let selector = Selector::parse("tr").unwrap();

        let mut points_map = BTreeMap::new();
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
                points_map.insert(card_name.trim().replace('’', "'").to_string(), points);
                // replace ’ with ' to be compatible with Moxfield data
            }
        }

        //println!("{:?}", points);
        Ok(points_map)
    }
}
