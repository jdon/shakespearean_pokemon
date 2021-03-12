use crate::client::client_error;
use crate::client::pokemon_client::get_description;
use crate::{POKEMON_CLIENT, SHAKESPEARE_CLIENT};
use std::convert::Infallible;
use warp::Filter;

pub fn pokemon_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("pokemon" / String)
        .and(warp::get())
        .and_then(get_pokemon)
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
                        return Ok(warp::reply::with_status(
                            translation,
                            warp::http::StatusCode::OK,
                        ));
                    }
                    Err(err) => match err {
                        client_error::ClientError::ShakespeareTooManyRequestsError => {
                            Ok(warp::reply::with_status(
                                "Too many requests".into(),
                                warp::http::StatusCode::TOO_MANY_REQUESTS,
                            ))
                        }
                        _ => Ok(warp::reply::with_status(
                            "Failed to get shakespeare translation".into(),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )),
                    },
                };
            }

            Ok(warp::reply::with_status(
                "Failed to find pokemon description".into(),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
        Err(pokemon_error) => match pokemon_error {
            client_error::ClientError::PokemonNotFoundError => Ok(warp::reply::with_status(
                "Failed to find pokemon".into(),
                warp::http::StatusCode::NOT_FOUND,
            )),
            _ => Ok(warp::reply::with_status(
                "Failed to get pokemon".into(),
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
    use serde::{Deserialize, Serialize};
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

        // acct
        let filter = pokemon_filter();
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "Spits fire yond is hot enow to melt boulders. Known to cause forest fires unintentionally.");
    }
}
