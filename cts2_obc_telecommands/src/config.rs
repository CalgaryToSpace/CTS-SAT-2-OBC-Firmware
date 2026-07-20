/**
 * - Enum of all configuration variable names.
 * - Struct that we create a global static singleton that contains all variable names.
 * - Create a getter and a setter telecommand (getter arg: name of variable, 
 *   setter arg: name of variable, value)
 * - Add configuration variable: heartbeat in ms (for testing to start)
 * - Add configuration variable: config_demo_variable1
 */
use core::sync::atomic::{AtomicU32, Ordering};
use core::str::FromStr;
use crate::error::ConfigError;

use crate::shared;

// Global configuration store
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
        match value_type {
            "u32" => value_str
                .parse::<u32>()
                .map(ConfigValue::U32)
                .map_err(|_| ConfigError::ConfigTypeMismatch),
            "bool" => value_str
                .parse::<bool>()
                .map(ConfigValue::Bool)
                .map_err(|_| ConfigError::ConfigTypeMismatch),
            "f32" => value_str
                .parse::<f32>()
                .map(ConfigValue::F32)
                .map_err(|_| ConfigError::ConfigTypeMismatch),
            "i32" => value_str
                .parse::<i32>()
                .map(ConfigValue::I32)
                .map_err(|_| ConfigError::ConfigTypeMismatch),
            "u8" => value_str
                .parse::<u8>()
                .map(ConfigValue::U8)
                .map_err(|_| ConfigError::ConfigTypeMismatch),
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
            config_demo_variable1: AtomicU32::new(0),
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
            _ => Err(ConfigError::ConfigTypeMismatch),
        }
    }
}

impl ConfigValue {
    pub fn parse_from_str(s: &str) -> Result<Self, ConfigError> {
        Err(ConfigError::ConfigValueParseError)
    }
}
