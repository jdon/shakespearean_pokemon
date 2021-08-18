mod pokemon;
mod translated;

use warp::Filter;

use crate::client::{
    pokemon_client::{Pokemon, PokemonClient},
    translation_client::TranslationClient,
};
use serde::{Deserialize, Serialize};

pub fn routes(
    pokemon_client: PokemonClient,
    translation_client: TranslationClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let clone_pokemon_client = pokemon_client.clone();
    let get_pokemon_route = warp::path!("pokemon" / String)
        .and(warp::get())
        .and_then(move |name| pokemon::get(clone_pokemon_client.clone(), name));

    let get_translated_pokemon = warp::path!("pokemon" / "translated" / String)
        .and(warp::get())
        .and_then(move |name| {
            translated::get(pokemon_client.clone(), translation_client.clone(), name)
        });

    warp::get()
        .and(get_translated_pokemon)
        .or(get_pokemon_route)
}

#[derive(Serialize, Deserialize)]
pub struct PokemonResponse {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "isLegendary")]
    pub is_legendary: bool,
    pub habitat: String,
}

impl From<Pokemon> for PokemonResponse {
    fn from(pokemon: Pokemon) -> Self {
        let description = pokemon.get_description();
        Self {
            name: pokemon.name,
            is_legendary: pokemon.is_legendary,
            habitat: pokemon.habitat.name,
            description,
        }
    }
}
