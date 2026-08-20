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

    #[error("Unbalanced parentheses")]
    UnbalancedParentheses,

    #[error("Cannot parse the type with the value string")]
    ParseStrValueError,

    #[error("Empty telecommand string")]
    EmptyTelecommandString,
}

// config operation errors
#[derive(Debug, PartialEq, Copy, Clone, Error)]
pub enum ConfigError {
    #[error("Configuration variable not found")]
    ConfigVariableNotFound,

    #[error("Cannot parse the type with the value string")]
    ConfigParseValueTypeError,

    #[error("Type mismatch for configuration variable (config variable is another type)")]
    ConfigVariableNotThisType,

    #[error("Unknown type for configuration variable")]
    ConfigVariableUnknownType,
}
