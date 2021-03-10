use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum ClientError {
    #[error("Failed to find pokemon")]
    PokemonNotFoundError,
    #[error("Failed to deserialize pokemon data")]
    PokemonDeserializationError,
    #[error("Failed to get pokemon")]
    PokemonAPIError,
}
