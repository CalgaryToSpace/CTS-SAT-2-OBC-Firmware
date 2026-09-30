// Use Mutex to safely share peripherals across tasks/interrupt?
// Implement critical sections for safe access to shared resources?

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

pub mod config;
use config::ConfigStore;

pub mod error;

pub mod definitions;

pub mod parser;
pub use parser::{Telecommand, parse_telecommand};

mod shared;

// global static singleton for configuration
static CONFIG_STORE: ConfigStore = ConfigStore::new();

// get reference to the global configuration store
pub fn get_config_store() -> &'static ConfigStore {
    &CONFIG_STORE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigValue, ConfigVariableName};
    use crate::error::ConfigError;
    use core::str::FromStr;

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

    #[test]
    fn test_global_config_store() {
        let store = get_config_store();

        store
            .set(ConfigVariableName::HeartbeatMs, ConfigValue::U32(500))
            .unwrap();
        assert_eq!(
            store.get(ConfigVariableName::HeartbeatMs),
            ConfigValue::U32(500)
        );
    }
}
