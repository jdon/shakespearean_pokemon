use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum ClientError {
    #[error("Failed to find pokemon")]
    PokemonNotFoundError,
    #[error("Failed to deserialize pokemon data")]
    PokemonDeserializationError,
    #[error("Failed to get pokemon")]
    PokemonAPIError,
    #[error("Failed to deserialize shakespeare data")]
    ShakespeareDeserializationError,
    #[error("Failed to get shakespeare translation")]
    ShakespeareAPIError,
    #[error("Failed to get shakespeare translation, too many requests")]
    ShakespeareTooManyRequestsError,
}
