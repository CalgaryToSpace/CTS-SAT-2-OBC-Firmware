use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParsedTelecommandErr<'a> {
    #[error("Unknown telecommand")]
    UnknownCommand(&'a str),
    
    #[error("Failed to deserialize telecommand arguments")]
    DeserializationError(#[from] serde_json_core::de::Error),
}
