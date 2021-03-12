use crate::client::client_error;
use crate::client::pokemon_client::get_description;
use crate::{POKEMON_CLIENT, SHAKESPEARE_CLIENT};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use warp::Filter;

pub fn pokemon_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("pokemon" / String)
        .and(warp::get())
        .and_then(get_pokemon)
}
#[derive(Serialize, Deserialize)]
pub struct GetPokemonOutput {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPokemonErrorOutput {
    error: String,
}

async fn get_pokemon(pokemon_name: String) -> Result<impl warp::Reply, Infallible> {
    let response = POKEMON_CLIENT.get_pokemon(&pokemon_name).await;
    match response {
        Ok(pokemon) => {
            let maybe_description = get_description(&pokemon.flavor_text_entries);
            if let Some(description) = maybe_description {
                let translation_response = SHAKESPEARE_CLIENT.get_translation(&description).await;
                return match translation_response {
                    Ok(translation) => {
                        let output = GetPokemonOutput {
                            name: pokemon_name,
                            description: translation,
                        };
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&output),
                            warp::http::StatusCode::OK,
                        ));
                    }
                    Err(err) => match err {
                        client_error::ClientError::ShakespeareTooManyRequestsError => {
                            Ok(warp::reply::with_status(
                                warp::reply::json(&GetPokemonErrorOutput {
                                    error: "Too many requests".into(),
                                }),
                                warp::http::StatusCode::TOO_MANY_REQUESTS,
                            ))
                        }
                        _ => Ok(warp::reply::with_status(
                            warp::reply::json(&GetPokemonErrorOutput {
                                error: "Failed to get shakespeare translation".into(),
                            }),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )),
                    },
                };
            }

            Ok(warp::reply::with_status(
                warp::reply::json(&GetPokemonErrorOutput {
                    error: "Failed to find pokemon description".into(),
                }),
                warp::http::StatusCode::NOT_FOUND,
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
    use std::env;

    use crate::client::{
        pokemon_client::{FlavorTextEntry, Language, Pokemon},
        shakespeare_client::{ShakespeareResponse, ShakespeareSuccess, ShakespeareTextContents},
    };
    use serde_json::json;

    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TextInput {
        text: String,
    }
    #[tokio::test]
    async fn it_successfully_gets_the_translated_pokemon_text() {
        // arrange
        let mock_server = MockServer::start().await;
        env::set_var("port", "5000");
        env::set_var("pokemon_api_base_url", mock_server.uri());
        env::set_var("shakespeare_api_base_url", mock_server.uri());

        let expected_text = TextInput {
            text: "Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally.".into(),
        };

        let expected_shakespeare_body = ShakespeareResponse {
            success: ShakespeareSuccess { total: 1 },
            contents: ShakespeareTextContents {
                translated: "Spits fire yond is hot enow to melt boulders. Known to cause forest fires unintentionally.".into(),
                text: "Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally.".into(),
                translation: "shakespeare".into(),
            },
        };
        let mock_shakespeare_response =
            ResponseTemplate::new(200).set_body_json(json!(expected_shakespeare_body));

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
        };

        let mock_pokemon_response =
            ResponseTemplate::new(200).set_body_json(json!(generated_pokemon));

        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .respond_with(mock_shakespeare_response)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/charizard"))
            .respond_with(mock_pokemon_response)
            .mount(&mock_server)
            .await;

        // act
        let filter = pokemon_filter();
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "{\"name\":\"charizard\",\"description\":\"Spits fire yond is hot enow to melt boulders. Known to cause forest fires unintentionally.\"}");
    }

    #[tokio::test]
    async fn it_returns_404_on_invalid_pokemon() {
        // arrange
        let mock_server = MockServer::start().await;
        env::set_var("port", "5000");
        env::set_var("pokemon_api_base_url", mock_server.uri());
        env::set_var("shakespeare_api_base_url", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/invalidPokemon"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        // act
        let filter = pokemon_filter();
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/invalidPokemon")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 404);
        assert_eq!(res.body(), "{\"error\":\"Failed to find pokemon\"}");
    }

    #[tokio::test]
    async fn it_returns_429_on_too_many_shakespeare_requests() {
        // arrange
        let mock_server = MockServer::start().await;
        env::set_var("port", "5000");
        env::set_var("pokemon_api_base_url", mock_server.uri());
        env::set_var("shakespeare_api_base_url", mock_server.uri());

        let expected_text = TextInput {
		   text: "Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally.".into(),
	   };

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
	   };

        let mock_pokemon_response =
            ResponseTemplate::new(200).set_body_json(json!(generated_pokemon));

        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/charizard"))
            .respond_with(mock_pokemon_response)
            .mount(&mock_server)
            .await;

        // act
        let filter = pokemon_filter();
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 429);
        assert_eq!(res.body(), "{\"error\":\"Too many requests\"}");
    }
}
