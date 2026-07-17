use thiserror::Error;
use heapless::String;

// TODO: Use MAX_TELECOMMAND_STR_LENGTH constant instead of hardcode here
// when we have a shared crate cause importing here cause circular dependency
pub type CommandName = String<256>;

#[derive(Debug, Error)]
pub enum ParsedTelecommandErr {
    #[error("Unknown telecommand")]
    UnknownCommand(CommandName),

    #[error("Command too long")]
    CommandTooLong,
    
    #[error("Failed to deserialize telecommand arguments")]
    DeserializationError(#[from] serde_json_core::de::Error),
}
