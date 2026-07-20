use thiserror::Error;

pub type IndexMissing = u8;

#[derive(Debug, Error, PartialEq)]
pub enum ParsedTelecommandErr {
    #[error("Unknown telecommand")]
    UnknownCommand,

    #[error("Failed to deserialize telecommand arguments")]
    DeserializationError(#[from] serde_json_core::de::Error),

    #[error("Missing required argument")]
    MissingArgument(IndexMissing),

    #[error("Too many arguments provided")]
    ExceededArgumentCount,

    #[error("Configuration error")]
    ConfigError(#[from] ConfigError),
}

// config operation errors
#[derive(Debug, PartialEq, Error)]
pub enum ConfigError {
    #[error("Configuration variable not found")]
    ConfigVariableNotFound,

    #[error("Cannot parse value for configuration variable")]
    ConfigValueParseError,

    #[error("Type mismatch (cannot parse) for configuration variable")]
    ConfigTypeMismatch,

    #[error("Unknown type for configuration variable")]
    ConfigVariableUnknownType,
}
