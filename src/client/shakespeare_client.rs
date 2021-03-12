use super::client_error::ClientError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::StatusCode;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ShakespeareResponse {
    pub success: ShakespeareSuccess,
    pub contents: ShakespeareTextContents,
}

impl ShakespeareResponse {
    pub fn get_translation(&self) -> std::result::Result<String, ClientError> {
        match self.success.total {
            1 => Ok(self.contents.translated.clone()),
            _ => Err(ClientError::ShakespeareAPIError),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ShakespeareTextContents {
    pub translated: String,
    pub text: String,
    pub translation: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ShakespeareSuccess {
    pub total: i64,
}

pub struct ShakespeareClient {
    base_url: String,
    api_token: Option<String>,
}

const API_TOKEN_KEY: &'static str = "X-Funtranslations-Api-Secret";
pub const BASE_URL: &'static str = "https://api.funtranslations.com/translate/shakespeare.json";

impl ShakespeareClient {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            base_url: BASE_URL.into(),
            api_token: None,
        }
    }

    pub fn new_with_base_url(base_url: String, api_token: Option<String>) -> Self {
        Self {
            base_url,
            api_token,
        }
    }

    pub async fn get_translation_response(
        &self,
        text: &str,
    ) -> std::result::Result<ShakespeareResponse, ClientError> {
        let request_body = json!({ "text": text });

        let mut request = surf::post(&self.base_url).body(request_body).build();
        if let Some(token) = &self.api_token {
            request.insert_header(API_TOKEN_KEY, token);
        }

        let mut response = surf::client()
            .send(request)
            .await
            .map_err(|_| ClientError::ShakespeareAPIError)?;

        match response.status() {
            StatusCode::Ok => {
                let data: ShakespeareResponse = response
                    .body_json()
                    .await
                    .map_err(|_| ClientError::ShakespeareDeserializationError)?;
                Ok(data)
            }
            StatusCode::TooManyRequests => Err(ClientError::ShakespeareTooManyRequestsError),
            _ => Err(ClientError::ShakespeareAPIError),
        }
    }

    pub async fn get_translation(&self, text: &str) -> std::result::Result<String, ClientError> {
        let response = self.get_translation_response(text).await?;
        return response.get_translation();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TextInput {
        text: String,
    }
    #[tokio::test]
    async fn it_returns_an_api_error_on_500_response() {
        // arrange
        let expected_text = TextInput {
            text: "Hello world".into(),
        };

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = ShakespeareClient::new_with_base_url(mock_server.uri(), None);

        // act
        let response = client.get_translation_response("Hello world").await;

        // assert
        if let Err(err) = response {
            assert_eq!(err, ClientError::ShakespeareAPIError);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn it_errors_on_invalid_data() {
        // arrange
        let expected_text = TextInput {
            text: "Hello world".into(),
        };

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = ShakespeareClient::new_with_base_url(mock_server.uri(), None);

        // act
        let response = client.get_translation_response("Hello world").await;

        // assert
        if let Err(err) = response {
            assert_eq!(err, ClientError::ShakespeareDeserializationError);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn it_successfully_deserializes_a_response() {
        // arrange
        let expected_text = TextInput {
            text: "Hello world".into(),
        };

        let expected_body = ShakespeareResponse {
            success: ShakespeareSuccess { total: 1 },
            contents: ShakespeareTextContents {
                translated: "world hello".into(),
                text: "hello world".into(),
                translation: "shakespeare".into(),
            },
        };
        let mock_response = ResponseTemplate::new(200).set_body_json(json!(expected_body));

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;

        let client = ShakespeareClient::new_with_base_url(mock_server.uri(), None);

        // act
        let response = client
            .get_translation_response("Hello world")
            .await
            .unwrap();

        // assert
        assert_eq!(response, expected_body);
    }

    #[tokio::test]
    async fn it_successfully_sends_an_api_key() {
        // arrange
        let api_token = "an_api_token";
        let expected_text = TextInput {
            text: "Hello world".into(),
        };

        let expected_body = ShakespeareResponse {
            success: ShakespeareSuccess { total: 1 },
            contents: ShakespeareTextContents {
                translated: "world hello".into(),
                text: "hello world".into(),
                translation: "shakespeare".into(),
            },
        };

        let mock_response = ResponseTemplate::new(200).set_body_json(json!(expected_body));
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .and(header(API_TOKEN_KEY, api_token))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;

        let client =
            ShakespeareClient::new_with_base_url(mock_server.uri(), Some(api_token.into()));

        // act
        let response = client
            .get_translation_response("Hello world")
            .await
            .unwrap();

        // assert
        assert_eq!(response, expected_body);
    }

    #[tokio::test]
    async fn it_successfully_gets_translation() {
        // arrange
        let expected_text = TextInput {
            text: "Hello world".into(),
        };

        let expected_body = ShakespeareResponse {
            success: ShakespeareSuccess { total: 1 },
            contents: ShakespeareTextContents {
                translated: "world hello".into(),
                text: "hello world".into(),
                translation: "shakespeare".into(),
            },
        };
        let mock_response = ResponseTemplate::new(200).set_body_json(json!(expected_body));

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;

        let client = ShakespeareClient::new_with_base_url(mock_server.uri(), None);

        //act
        let response = client.get_translation("Hello world").await.unwrap();

        // assert
        assert_eq!(response, "world hello");
    }
}
