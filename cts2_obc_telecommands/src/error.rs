use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ParsedTelecommandErr {
    #[error("Unknown telecommand")]
    UnknownCommand,

    #[error("Failed to deserialize telecommand arguments")]
    DeserializationError(#[from] serde_json_core::de::Error),
}

// config operation errors
#[derive(Debug, PartialEq, Error)]
pub enum ConfigError {
    #[error("Configuration variable not found")]
    ConfigVariableNotFound,
}
