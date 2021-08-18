use crate::client::{
    client_error,
    pokemon_client::PokemonClient,
    translation_client::{TranslationClient, TranslationType},
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use super::PokemonResponse;

#[derive(Serialize, Deserialize)]
struct GetTranslationErrorOutput {
    error: String,
}

pub async fn get(
    pokemon_client: PokemonClient,
    translation_client: TranslationClient,
    pokemon_name: String,
) -> Result<impl warp::Reply, Infallible> {
    let response = pokemon_client.get_pokemon(&pokemon_name).await;
    match response {
        Ok(pokemon) => {
            let mut translation_type = TranslationType::SHAKESPEARE;
            if pokemon.habitat.name == "cave" || pokemon.is_legendary {
                translation_type = TranslationType::YODA;
            }

            let description = match pokemon.get_description() {
                Some(desc) => {
                    let translation_response =
                        translation_client.get_translation(&desc, translation_type).await;
                    match translation_response {
                        Ok(translated_text) => Some(translated_text),
                        Err(_) => Some(desc), // Swallowing error as task says to use standard description if we fail to translate
                    }
                }
                None => None,
            };

            let response = PokemonResponse {
                name: pokemon.name,
                description,
                is_legendary: pokemon.is_legendary,
                habitat: pokemon.habitat.name,
            };

            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        }
        Err(pokemon_error) => match pokemon_error {
            client_error::ClientError::PokemonNotFoundError => Ok(warp::reply::with_status(
                warp::reply::json(&GetTranslationErrorOutput {
                    error: "Failed to find pokemon".into(),
                }),
                warp::http::StatusCode::NOT_FOUND,
            )),
            _ => Ok(warp::reply::with_status(
                warp::reply::json(&GetTranslationErrorOutput {
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
        translation_client::{TranslationResponse, TranslationSuccess, TranslationTextContents},
    };
    use serde_json::json;

    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TextInput {
        text: String,
    }
    #[tokio::test]
    async fn it_successfully_gets_the_translated_pokemon_text_for_non_cave_non_legendary_pokemon() {
        // arrange
        let mock_server = MockServer::start().await;

        let expected_text = TextInput {
            text: "Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally.".into(),
        };

        let expected_shakespeare_body = TranslationResponse {
            success: TranslationSuccess { total: 1 },
            contents: TranslationTextContents {
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
            is_legendary: false,
            habitat: Habitat {
                name: "urban".into(),
                url: "https://pokeapi.co/api/v2/pokemon-habitat/8/".into(),
            },
        };

        let mock_pokemon_response =
            ResponseTemplate::new(200).set_body_json(json!(generated_pokemon));

        Mock::given(method("POST"))
            .and(path("/translate/shakespeare.json"))
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
        let filter = crate::routes::routes(
            PokemonClient::new(mock_server.uri()),
            TranslationClient::new(mock_server.uri(), None),
        );
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/translated/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "{\"name\":\"charizard\",\"description\":\"Spits fire yond is hot enow to melt boulders. Known to cause forest fires unintentionally.\",\"isLegendary\":false,\"habitat\":\"urban\"}");
    }

    #[tokio::test]
    async fn it_successfully_gets_the_translated_pokemon_text_for_cave_pokemon() {
        // arrange
        let mock_server = MockServer::start().await;

        let expected_text = TextInput {
            text: "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.".into(),
        };

        let expected_yoda_body = TranslationResponse {
            success: TranslationSuccess { total: 1 },
            contents: TranslationTextContents {
                translated: "Forms colonies in perpetually dark places.And approach targets, uses ultrasonic waves to identify.".into(),
                text: "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.".into(),
                translation: "yoda".into(),
            },
        };
        let mock_yoda_response =
            ResponseTemplate::new(200).set_body_json(json!(expected_yoda_body));

        let generated_pokemon = Pokemon {
            id: 6,
            name: "zubat".into(),
            flavor_text_entries: vec![FlavorTextEntry {
                flavor_text: "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.".into(),
                language: Language {
					name: "en".into(),
					url: "https://pokeapi.co/api/v2/language/9/".into()
				},
            }],
            is_legendary: false,
            habitat: Habitat {
                name: "cave".into(),
                url: "https://pokeapi.co/api/v2/pokemon-habitat/8/".into(),
            },
        };

        let mock_pokemon_response =
            ResponseTemplate::new(200).set_body_json(json!(generated_pokemon));

        Mock::given(method("POST"))
            .and(path("/translate/yoda.json"))
            .and(body_json(expected_text))
            .respond_with(mock_yoda_response)
            .mount(&mock_server)
            .await;

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
            .path("/pokemon/translated/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "{\"name\":\"zubat\",\"description\":\"Forms colonies in perpetually dark places.And approach targets, uses ultrasonic waves to identify.\",\"isLegendary\":false,\"habitat\":\"cave\"}");
    }

    #[tokio::test]
    async fn it_successfully_gets_the_translated_pokemon_text_for_legendary_pokemon() {
        // arrange
        let mock_server = MockServer::start().await;

        let expected_text = TextInput {
            text: "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.".into(),
        };

        let expected_yoda_body = TranslationResponse {
            success: TranslationSuccess { total: 1 },
            contents: TranslationTextContents {
                translated: "Forms colonies in perpetually dark places.And approach targets, uses ultrasonic waves to identify.".into(),
                text: "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.".into(),
                translation: "yoda".into(),
            },
        };
        let mock_yoda_response =
            ResponseTemplate::new(200).set_body_json(json!(expected_yoda_body));

        let generated_pokemon = Pokemon {
            id: 6,
            name: "zubat".into(),
            flavor_text_entries: vec![FlavorTextEntry {
                flavor_text: "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.".into(),
                language: Language {
					name: "en".into(),
					url: "https://pokeapi.co/api/v2/language/9/".into()
				},
            }],
            is_legendary: true,
            habitat: Habitat {
                name: "urban".into(),
                url: "https://pokeapi.co/api/v2/pokemon-habitat/8/".into(),
            },
        };

        let mock_pokemon_response =
            ResponseTemplate::new(200).set_body_json(json!(generated_pokemon));

        Mock::given(method("POST"))
            .and(path("/translate/yoda.json"))
            .and(body_json(expected_text))
            .respond_with(mock_yoda_response)
            .mount(&mock_server)
            .await;

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
            .path("/pokemon/translated/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "{\"name\":\"zubat\",\"description\":\"Forms colonies in perpetually dark places.And approach targets, uses ultrasonic waves to identify.\",\"isLegendary\":true,\"habitat\":\"urban\"}");
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
            .path("/pokemon/translated/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 404);
        assert_eq!(res.body(), "{\"error\":\"Failed to find pokemon\"}");
    }

    #[tokio::test]
    async fn it_returns_default_description_on_429() {
        // arrange
        let mock_server = MockServer::start().await;

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
            is_legendary: false,
            habitat: Habitat {
                name: "urban".into(),
                url: "https://pokeapi.co/api/v2/pokemon-habitat/8/".into(),
            },
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
        let filter = crate::routes::routes(
            PokemonClient::new(mock_server.uri()),
            TranslationClient::new(mock_server.uri(), None),
        );
        let res = warp::test::request()
            .method("GET")
            .path("/pokemon/translated/charizard")
            .reply(&filter)
            .await;

        // assert
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "{\"name\":\"charizard\",\"description\":\"Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally.\",\"isLegendary\":false,\"habitat\":\"urban\"}");
    }
}
