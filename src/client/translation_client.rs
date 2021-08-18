use super::client_error::ClientError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::{Client, StatusCode};

const API_TOKEN_KEY: &str = "X-Funtranslations-Api-Secret";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TranslationResponse {
    pub success: TranslationSuccess,
    pub contents: TranslationTextContents,
}

impl TranslationResponse {
    pub fn get_translation(&self) -> std::result::Result<String, ClientError> {
        match self.success.total {
            1 => Ok(self.contents.translated.clone()),
            _ => Err(ClientError::TranslationAPIError),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TranslationTextContents {
    pub translated: String,
    pub text: String,
    pub translation: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TranslationSuccess {
    pub total: i64,
}

pub enum TranslationType {
    YODA,
    SHAKESPEARE,
}

impl TranslationType {
    fn as_url(&self) -> &'static str {
        match self {
            TranslationType::YODA => "translate/yoda.json",
            TranslationType::SHAKESPEARE => "translate/shakespeare.json",
        }
    }
}

#[derive(Clone)]
pub struct TranslationClient {
    base_url: String,
    api_token: Option<String>,
    client: Client, // Surfs clone implementation shares the underlying HttpClient
}

impl TranslationClient {
    pub fn new(base_url: String, api_token: Option<String>) -> Self {
        Self {
            base_url,
            api_token,
            client: Client::new(),
        }
    }

    async fn get_translation_response(
        &self,
        text: &str,
        translation_type: TranslationType,
    ) -> std::result::Result<TranslationResponse, ClientError> {
        let request_body = json!({ "text": text });

        let url = format!("{}/{}", self.base_url, translation_type.as_url());

        let mut request = surf::post(url).body(request_body).build();
        if let Some(token) = &self.api_token {
            request.insert_header(API_TOKEN_KEY, token.as_str());
        }

        let mut response = surf::client()
            .send(request)
            .await
            .map_err(|_| ClientError::TranslationAPIError)?;

        match response.status() {
            StatusCode::Ok => {
                let data: TranslationResponse = response
                    .body_json()
                    .await
                    .map_err(|_| ClientError::TranslationDeserializationError)?;
                Ok(data)
            }
            StatusCode::TooManyRequests => Err(ClientError::TranslationTooManyRequestsError),
            _ => Err(ClientError::TranslationAPIError),
        }
    }

    pub async fn get_translation(
        &self,
        text: &str,
        translation_type: TranslationType,
    ) -> std::result::Result<String, ClientError> {
        let response = self
            .get_translation_response(text, translation_type)
            .await?;
        response.get_translation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, header, method, path};
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
            .and(path("translate/yoda.json"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = TranslationClient::new(mock_server.uri(), None);

        // act
        let response = client
            .get_translation("Hello world", TranslationType::YODA)
            .await;

        // assert
        if let Err(err) = response {
            assert_eq!(err, ClientError::TranslationAPIError);
        } else {
            unreachable!();
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
            .and(path("translate/yoda.json"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = TranslationClient::new(mock_server.uri(), None);

        // act
        let response = client
            .get_translation("Hello world", TranslationType::YODA)
            .await;

        // assert
        if let Err(err) = response {
            assert_eq!(err, ClientError::TranslationDeserializationError);
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn it_successfully_deserializes_a_response() {
        // arrange
        let expected_text = TextInput {
            text: "Hello world".into(),
        };

        let expected_body = TranslationResponse {
            success: TranslationSuccess { total: 1 },
            contents: TranslationTextContents {
                translated: "world hello".into(),
                text: "hello world".into(),
                translation: "shakespeare".into(),
            },
        };
        let mock_response = ResponseTemplate::new(200).set_body_json(json!(expected_body));

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .and(path("translate/shakespeare.json"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;

        let client = TranslationClient::new(mock_server.uri(), None);

        // act
        let response = client
            .get_translation_response("Hello world", TranslationType::SHAKESPEARE)
            .await
            .unwrap();

        // assert
        assert_eq!(response, expected_body);
    }

    #[tokio::test]
    async fn it_successfully_sends_an_api_key() {
        // arrange
        let api_token = String::from("an_api_token");
        let expected_text = TextInput {
            text: "Hello world".into(),
        };

        let expected_body = TranslationResponse {
            success: TranslationSuccess { total: 1 },
            contents: TranslationTextContents {
                translated: "world hello".into(),
                text: "hello world".into(),
                translation: "shakespeare".into(),
            },
        };

        let mock_response = ResponseTemplate::new(200).set_body_json(json!(expected_body));
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .and(path("translate/shakespeare.json"))
            .and(header(API_TOKEN_KEY, api_token.as_str()))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;

        let client = TranslationClient::new(mock_server.uri(), Some(api_token));

        // act
        let response = client
            .get_translation_response("Hello world", TranslationType::SHAKESPEARE)
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

        let expected_body = TranslationResponse {
            success: TranslationSuccess { total: 1 },
            contents: TranslationTextContents {
                translated: "world hello".into(),
                text: "hello world".into(),
                translation: "shakespeare".into(),
            },
        };
        let mock_response = ResponseTemplate::new(200).set_body_json(json!(expected_body));

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_json(expected_text))
            .and(path("translate/yoda.json"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;

        let client = TranslationClient::new(mock_server.uri(), None);

        //act
        let response = client
            .get_translation("Hello world", TranslationType::YODA)
            .await
            .unwrap();

        // assert
        assert_eq!(response, "world hello");
    }
}
