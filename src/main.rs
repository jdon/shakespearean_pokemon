pub mod client;
mod routes;
use client::{pokemon_client::PokemonClient, shakespeare_client::ShakespeareClient};

use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    port: u16,
    api_token: Option<String>,
    pokemon_api_base_url: Option<String>,
    shakespeare_api_base_url: Option<String>,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        match envy::from_env::<Config>() {
            Ok(config) => config,
            Err(error) => panic!("{:#?}", error),
        }
    };
    pub static ref POKEMON_CLIENT: PokemonClient = {
        match &CONFIG.pokemon_api_base_url {
            Some(base_url) => PokemonClient::new_with_base_url(base_url.into()),
            None => PokemonClient::new(),
        }
    };
    pub static ref SHAKESPEARE_CLIENT: ShakespeareClient = {
        match &CONFIG.shakespeare_api_base_url {
            Some(base_url) => {
                ShakespeareClient::new_with_base_url(base_url.into(), CONFIG.api_token.clone())
            }
            None => ShakespeareClient::new_with_base_url(
                client::shakespeare_client::BASE_URL.into(),
                CONFIG.api_token.clone(),
            ),
        }
    };
}

#[tokio::main]
async fn main() {
    warp::serve(routes::pokemon::pokemon_filter())
        .run(([0, 0, 0, 0], CONFIG.port))
        .await;
}
