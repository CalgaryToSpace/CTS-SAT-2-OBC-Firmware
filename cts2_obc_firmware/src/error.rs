use cts2_obc_telecommands::error::ParsedTelecommandErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchCommandErr {
    #[error("Parsed telecommand error")]
    ParsedTelecommandError(#[from] ParsedTelecommandErr),

    #[error("Failed to execute telecommand")]
    ExecuteCmdError(#[from] ExecuteCommandErr),
}

#[derive(Debug, Error)]
pub enum ExecuteCommandErr {}
