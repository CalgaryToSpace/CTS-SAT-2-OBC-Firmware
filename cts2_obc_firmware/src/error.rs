use thiserror::Error;
use cts2_obc_telecommands::error::ParsedTelecommandErr;

#[derive(Debug, Error)]
pub enum DispatchCmdErr {
    #[error("Parsed telecommand error")]
    ParsedTelecommandError(#[from] ParsedTelecommandErr),

    #[error("Failed to execute telecommand")]
    ExecuteCmdError(#[from] ExecuteCmdErr),
}

#[derive(Debug, Error)]
pub enum ExecuteCmdErr {
}
