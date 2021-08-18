pub mod client;
mod routes;
use client::{pokemon_client::PokemonClient, translation_client::TranslationClient};

use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    port: u16,
    api_token: Option<String>,
    pokemon_api_base_url: String,
    translation_api_base_url: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        match envy::from_env::<Config>() {
            Ok(config) => config,
            Err(error) => panic!("{:#?}", error),
        }
    };
}

#[tokio::main]
async fn main() {
    println!("Starting server on port {}", CONFIG.port);
    let routes = crate::routes::routes(
        PokemonClient::new(CONFIG.pokemon_api_base_url.clone()),
        TranslationClient::new(CONFIG.translation_api_base_url.clone(), None),
    );
    warp::serve(routes).run(([0, 0, 0, 0], CONFIG.port)).await;
}
