use crate::error::ExecuteCommandErr;
use crate::error::ParsedTelecommandErr;
use core::ops::Range;
use serde::Serialize;

pub enum ReadinessLevel {
    Operation,

    RecoveryOrExpert,

    FlightTest,

    GroundUsage,

    HighRiskUnsafe,
}

// Add new parameter types as needed
#[derive(Serialize)]
#[serde(untagged)]
pub enum ParameterType {
    U32,
    U64,
    String,
}

impl ParameterType {
    pub fn type_str(&self) -> &'static str {
        match self {
            ParameterType::U32 => "u32",
            ParameterType::U64 => "u64",
            ParameterType::String => "str",
        }
    }
}

#[derive(Serialize)]
pub struct Parameter {
    pub name: &'static str,
    pub data_type: ParameterType,
    pub units: Option<&'static str>,
    pub constraints: Option<Range<u64>>,
}

pub struct TelecommandDefinition {
    pub name: &'static str,

    pub num_parameters: u8,

    pub parameter_list: &'static [Parameter],

    pub readiness: ReadinessLevel,

    pub exec: fn(&str) -> Result<(), ExecuteCommandErr>,

    pub validate: fn(&str) -> Result<(), ParsedTelecommandErr>,
}
