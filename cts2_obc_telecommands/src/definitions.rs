use crate::error::ExecuteCommandErr;
use crate::error::ParsedTelecommandErr;

pub enum ReadinessLevel {
    Operation,

    RecoveryOrExpert,

    FlightTest,

    GroundUsage,

    HighRiskUnsafe,
}

pub struct TelecommandDefinition {
    pub name: &'static str,

    pub num_parameters: u8,

    pub readiness: ReadinessLevel,

    pub exec: fn(&str) -> Result<(), ExecuteCommandErr>,

    pub validate: fn(&str) -> Result<(), ParsedTelecommandErr>,
}
