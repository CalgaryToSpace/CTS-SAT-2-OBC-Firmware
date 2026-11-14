use cts2_obc_logic::error::SchedulerError;
pub use cts2_obc_telecommands::error::ExecuteCommandErr;
use cts2_obc_telecommands::error::ParsedTelecommandErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchCommandErr {
    #[error("Parsed telecommand error")]
    ParsedTelecommandError(#[from] ParsedTelecommandErr),

    #[error("Failed to execute telecommand")]
    ExecuteCommandError(#[from] ExecuteCommandErr),

    #[error("Scheduler error")]
    Scheduler(#[from] SchedulerError),

    #[error("Telecommand arguments exceed scheduler buffer capacity")]
    ArgumentsTooLong,
}
