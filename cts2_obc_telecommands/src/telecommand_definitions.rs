use crate::error::ExecuteCommandErr;
use crate::error::ParsedTelecommandErr;
use serde::Serialize;

#[derive(Serialize)]
pub enum ReadinessLevel {
    Operation,

    RecoveryOrExpert,

    FlightTest,

    GroundUsage,

    HighRiskUnsafe,
}

#[derive(Serialize)]
pub struct TelecommandDefinition {
    pub name: &'static str,

    pub num_parameters: u8,

    pub readiness: ReadinessLevel,

    #[serde(skip_serializing)]
    pub exec: fn(&str) -> Result<(), ExecuteCommandErr>,

    #[serde(skip_serializing)]
    pub validate: fn(&str) -> Result<(), ParsedTelecommandErr>,
}
