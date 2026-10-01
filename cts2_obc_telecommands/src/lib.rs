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
