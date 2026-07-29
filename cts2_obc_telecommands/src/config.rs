use crate::error::ConfigError;
use core::str::FromStr;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::shared;

// Global configuration store
// There is no float for atomic, consider
// using AtomicU32 to store and just parse as float
pub struct ConfigStore {
    heartbeat_ms: AtomicU32,
    config_demo_variable1: AtomicU32,
}

// All configuration variable names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigVariableName {
    HeartbeatMs,
    ConfigDemoVariable1,
}

impl FromStr for ConfigVariableName {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heartbeat_ms" => Ok(ConfigVariableName::HeartbeatMs),
            "config_demo_variable1" => Ok(ConfigVariableName::ConfigDemoVariable1),
            _ => Err(ConfigError::ConfigVariableNotFound),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigValue {
    U32(u32),
    Bool(bool),
    F32(f32),
    I32(i32),
    U8(u8),
}

impl FromStr for ConfigValue {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (value_type, value_str) = shared::extract_function_and_args(s);

        macro_rules! parse_value {
            ($type:ty, $variant:path) => {
                value_str
                    .parse::<$type>()
                    .map($variant)
                    .map_err(|_| ConfigError::ConfigParseValueTypeError)
            };
        }

        match value_type {
            "u32" => parse_value!(u32, ConfigValue::U32),
            "bool" => parse_value!(bool, ConfigValue::Bool),
            "f32" => parse_value!(f32, ConfigValue::F32),
            "i32" => parse_value!(i32, ConfigValue::I32),
            "u8" => parse_value!(u8, ConfigValue::U8),
            _ => Err(ConfigError::ConfigVariableUnknownType),
        }
    }
}

impl ConfigStore {
    // create new config store with default values
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            heartbeat_ms: AtomicU32::new(1000),
            config_demo_variable1: AtomicU32::new(123),
        }
    }

    // get a config value by name
    pub fn get(&self, name: ConfigVariableName) -> ConfigValue {
        match name {
            ConfigVariableName::HeartbeatMs => {
                ConfigValue::U32(self.heartbeat_ms.load(Ordering::Relaxed))
            }
            ConfigVariableName::ConfigDemoVariable1 => {
                ConfigValue::U32(self.config_demo_variable1.load(Ordering::Relaxed))
            }
        }
    }

    // set a configuration value by name
    pub fn set(&self, name: ConfigVariableName, value: ConfigValue) -> Result<(), ConfigError> {
        match (name, value) {
            (ConfigVariableName::HeartbeatMs, ConfigValue::U32(v)) => {
                self.heartbeat_ms.store(v, Ordering::Relaxed);
                Ok(())
            }
            (ConfigVariableName::ConfigDemoVariable1, ConfigValue::U32(v)) => {
                self.config_demo_variable1.store(v, Ordering::Relaxed);
                Ok(())
            }
            _ => Err(ConfigError::ConfigVariableNotThisType),
        }
    }
}
