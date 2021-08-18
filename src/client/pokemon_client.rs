use super::client_error::ClientError;
use serde::{Deserialize, Serialize};
use surf::{Client, StatusCode};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Pokemon {
    pub id: i64,
    pub name: String,
    pub is_legendary: bool,
    pub habitat: Habitat,
    pub flavor_text_entries: Vec<FlavorTextEntry>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Habitat {
    pub name: String,
    pub url: String,
}

impl Pokemon {
    pub fn get_description(&self) -> Option<String> {
        let description = &self
            .flavor_text_entries
            .iter()
            .find(|entry| entry.language.name == "en");
        description.map(|entry| entry.flavor_text.replace("\n", " ").replace("\u{c}", ""))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FlavorTextEntry {
    pub flavor_text: String,
    pub language: Language,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Language {
    pub name: String,
    pub url: String,
}

#[derive(Clone)]
pub struct PokemonClient {
    base_url: String,
    client: Client, // Surfs clone implementation shares the underlying HttpClient
}

impl PokemonClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }
    pub async fn get_pokemon(&self, pokemon: &str) -> std::result::Result<Pokemon, ClientError> {
        let url = format!("{}/api/v2/pokemon-species/{}", self.base_url, pokemon);

        let mut res = self
            .client
            .get(url)
            .await
            .map_err(|_| ClientError::PokemonAPIError)?;

        match res.status() {
            StatusCode::Ok => {
                let data: Pokemon = res
                    .body_json()
                    .await
                    .map_err(|_| ClientError::PokemonDeserializationError)?;
                Ok(data)
            }
            StatusCode::NotFound => Err(ClientError::PokemonNotFoundError),
            _ => Err(ClientError::PokemonAPIError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn it_error_on_404() {
        // arrange
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/charizard"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let pokemon_client = PokemonClient::new(mock_server.uri());

        // act
        let res = pokemon_client.get_pokemon("charizard").await;

        // assert
        if let Err(err) = res {
            assert_eq!(err, ClientError::PokemonNotFoundError);
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn it_errors_on_invalid_data() {
        // arrange
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/charizard"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let pokemon_client = PokemonClient::new(mock_server.uri());

        // act
        let res = pokemon_client.get_pokemon("charizard").await;

        // assert
        if let Err(err) = res {
            assert_eq!(err, ClientError::PokemonDeserializationError);
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn it_correctly_deserializes_pokemon_species() {
        // arrange
        let mock_server = MockServer::start().await;

        let generated_pokemon = Pokemon {
            id: 6,
            name: "charizard".into(),
            is_legendary: false,
            habitat: Habitat {
                name: "urban".into(),
                url: "https://pokeapi.co/api/v2/pokemon-habitat/8/".into(),
            },
            flavor_text_entries: vec![FlavorTextEntry {
                flavor_text: "Spits fire that is hot enough to melt boulders.\nKnown to cause forest fires unintentionally.".into(),
                language: Language {
					name: "en".into(),
					url: "https://pokeapi.co/api/v2/language/9/".into()
				},
            }],
        };

        let mock_response = ResponseTemplate::new(200).set_body_json(json!(generated_pokemon));
        Mock::given(method("GET"))
            .and(path("/api/v2/pokemon-species/charizard"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;

        let pokemon_client = PokemonClient::new(mock_server.uri());

        // act
        let res = pokemon_client.get_pokemon("charizard").await.unwrap();

        // assert
        assert_eq!(res, generated_pokemon);
    }
}
