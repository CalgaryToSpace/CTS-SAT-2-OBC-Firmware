use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ParsedTelecommandErr {
    #[error("Unknown telecommand")]
    UnknownCommand,

    #[error("Failed to deserialize telecommand arguments")]
    DeserializationError(#[from] serde_json_core::de::Error),
}
