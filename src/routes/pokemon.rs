use crate::client::client_error;
use crate::client::pokemon_client::PokemonClient;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use super::PokemonResponse;

#[derive(Serialize, Deserialize)]
struct GetPokemonErrorOutput {
    error: String,
}

pub async fn get(
    pokemon_client: PokemonClient,
    pokemon_name: String,
) -> Result<impl warp::Reply, Infallible> {
    let response = pokemon_client.get_pokemon(&pokemon_name).await;
    match response {
        Ok(pokemon) => {
            let pokemon_response = PokemonResponse::from(pokemon);
            Ok(warp::reply::with_status(
                warp::reply::json(&pokemon_response),
                warp::http::StatusCode::OK,
            ))
        }
        Err(pokemon_error) => match pokemon_error {
            client_error::ClientError::PokemonNotFoundError => Ok(warp::reply::with_status(
                warp::reply::json(&GetPokemonErrorOutput {
                    error: "Failed to find pokemon".into(),
                }),
                warp::http::StatusCode::NOT_FOUND,
            )),
            _ => Ok(warp::reply::with_status(
                warp::reply::json(&GetPokemonErrorOutput {
                    error: "Failed to get pokemon".into(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::client::{
        pokemon_client::{FlavorTextEntry, Habitat, Language, Pokemon},
        translation_client::TranslationClient,
    };
    use serde_json::json;

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TextInput {
        text: String,
    }
    #[tokio::test]
    async fn it_successfully_get_a_pokemon() {
        // arrange
        let mock_server = MockServer::start().await;

        let generated_pokemon = Pokemon {
            id: 6,
            name: "charizard".into(),
            flavor_text_entries: vec![FlavorTextEntry {
                flavor_text: "Spits fire that is hot enough to melt boulders.\nKnown to cause forest fires unintentionally.".into(),
                language: Language {
					name: "en".into(),
					url: "https://pokeapi.co/api/v2/language/9/".into()
				},
            }],
            is_legendary: false,
            habitat: Habitat {
                name: "urban".into(),
                url: "https://pokeapi.co/api/v2/pokemon-habitat/8/".into(),
            },
        };

        let mock_pokemon_response =
            ResponseTemplate::new(200).set_body_json(json!(generated_pokemon));

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/charizard"))
            .respond_with(mock_pokemon_response)
            .mount(&mock_server)
            .await;

        // act
        let filter = crate::routes::routes(
            PokemonClient::new(mock_server.uri()),
            TranslationClient::new(mock_server.uri(), None),
        );
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "{\"name\":\"charizard\",\"description\":\"Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally.\",\"is_legendary\":false,\"habitat\":\"urban\"}");
    }

    #[tokio::test]
    async fn it_returns_404_on_invalid_pokemon() {
        // arrange
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/invalidPokemon"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        // act
        let filter = crate::routes::routes(
            PokemonClient::new(mock_server.uri()),
            TranslationClient::new(mock_server.uri(), None),
        );
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/invalidPokemon")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 404);
        assert_eq!(res.body(), "{\"error\":\"Failed to find pokemon\"}");
    }
}
