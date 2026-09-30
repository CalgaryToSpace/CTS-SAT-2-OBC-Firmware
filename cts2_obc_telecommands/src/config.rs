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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_store_get_set() {
        let store = ConfigStore::new();

        // Test default values
        assert_eq!(
            store.get(ConfigVariableName::HeartbeatMs),
            ConfigValue::U32(1000)
        );
        assert_eq!(
            store.get(ConfigVariableName::ConfigDemoVariable1),
            ConfigValue::U32(123)
        );

        // Test setting values
        assert!(
            store
                .set(ConfigVariableName::HeartbeatMs, ConfigValue::U32(2000))
                .is_ok()
        );
        assert_eq!(
            store.get(ConfigVariableName::HeartbeatMs),
            ConfigValue::U32(2000)
        );

        assert!(
            store
                .set(
                    ConfigVariableName::ConfigDemoVariable1,
                    ConfigValue::U32(42)
                )
                .is_ok()
        );
        assert_eq!(
            store.get(ConfigVariableName::ConfigDemoVariable1),
            ConfigValue::U32(42)
        );
    }

    #[test]
    fn test_config_store_set_type_mismatch() {
        let store = ConfigStore::new();

        let result = store.set(
            ConfigVariableName::ConfigDemoVariable1,
            ConfigValue::F32(42.0),
        );

        assert_eq!(result, Err(ConfigError::ConfigVariableNotThisType));
    }

    #[test]
    fn test_config_store_parse_unknown_variable() {
        let result = ConfigVariableName::from_str("unknown_variable");
        assert_eq!(result, Err(ConfigError::ConfigVariableNotFound));
    }

    #[test]
    fn test_config_store_parse_unknown_type() {
        let result = ConfigValue::from_str("unknown_type(42)");
        assert_eq!(result, Err(ConfigError::ConfigVariableUnknownType));
    }

    #[test]
    fn test_config_store_parse_invalid_value() {
        let result = ConfigValue::from_str("u32(not_a_number)");
        assert_eq!(result, Err(ConfigError::ConfigParseValueTypeError));
    }
}
