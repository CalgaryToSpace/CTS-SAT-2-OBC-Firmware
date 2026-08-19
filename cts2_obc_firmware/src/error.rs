use cts2_obc_logic::error::SchedulerError;
use cts2_obc_telecommands::error::{ConfigError, ParsedTelecommandErr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchCommandErr {
    #[error("Parsed telecommand error")]
    ParsedTelecommand(#[from] ParsedTelecommandErr),

    #[error("Scheduler error")]
    Scheduler(#[from] SchedulerError),
}

#[derive(Debug, Error)]
pub enum ExecuteCommandErr {
    #[error("Config operation error")]
    ConfigError(#[from] ConfigError),
}
