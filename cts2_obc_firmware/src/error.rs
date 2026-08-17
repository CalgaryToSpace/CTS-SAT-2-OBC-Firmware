use cts2_obc_telecommands::error::{ConfigError, ParsedTelecommandErr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchCommandErr {
    #[error("Parsed telecommand error")]
    ParsedTelecommandError(#[from] ParsedTelecommandErr),

    #[error("Failed to execute telecommand")]
    ExecuteCommandError(#[from] ExecuteCommandErr),
}

#[derive(Debug, Error)]
pub enum ExecuteCommandErr {
    #[error("Config operation error")]
    ConfigError(#[from] ConfigError),
}
