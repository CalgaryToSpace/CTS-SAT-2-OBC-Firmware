#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

use core::sync::atomic::{AtomicU32, Ordering};
use serde::{Deserialize, Serialize};
use serde_json_core::de::from_slice;

// --- Configuration Store (snake_case for json) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ConfigVariableName {
    #[serde(rename = "heartbeat_ms")]
    HeartbeatMs,
    #[serde(rename = "config_demo_variable1")]
    ConfigDemoVariable1,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum ConfigValue {
    U32(u32),
    U64(u64),
    Bool(bool),
    F32(f32),
    F64(f64),
    NullableU32(Option<u32>),
}

pub struct ConfigStore {
    heartbeat_ms: AtomicU32,
    config_demo_variable1: AtomicU32,
}

impl ConfigStore {
    // create new config store with default values
    pub fn new() -> Self {
        Self {
            heartbeat_ms: AtomicU32::new(1000),
            config_demo_variable1: AtomicU32::new(0),
        }
    }

    pub fn get(&self, name: ConfigVariableName) -> Option<ConfigValue> {
        match name {
            ConfigVariableName::HeartbeatMs => {
                Some(ConfigValue::U32(self.heartbeat_ms.load(Ordering::Relaxed)))
            }
            ConfigVariableName::ConfigDemoVariable1 => Some(ConfigValue::U32(
                self.config_demo_variable1.load(Ordering::Relaxed),
            )),
        }
    }

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
            _ => Err(ConfigError::TypeMismatch),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    TypeMismatch,
}

static CONFIG_STORE: ConfigStore = ConfigStore::new(); // Global singleton

pub fn get_config_store() -> &'static ConfigStore {
    &CONFIG_STORE
}

// --- Telecommand Structures ---
#[derive(Debug)]
#[allow(non_camel_case_types)] // Allow telecommand names that align with their function names.
pub struct DemoCommandWithArgumentsArgs {
    pub arg_u32: u32,
    pub arg_u64: u64,
    pub arg_bool: bool,
    pub arg_f32: f32,
    pub arg_f64: f64,
    pub arg_nullable_u32: Option<u32>,
}

pub struct ConfigGetArgs {
    // required arguments for a config get command
    pub name: ConfigVariableName,
}

pub struct ConfigSetArgs {
    // required arguments for a config set command
    pub name: ConfigVariableName,
    pub value: ConfigValue,
}

pub struct TelecommandExecutionResult {
    pub success: bool,
    pub message: &'static str,
}

pub enum Telecommand {
    hello_world,
    demo_command_with_arguments(DemoCommandWithArgumentsArgs),
    config_get(ConfigGetArgs),
    config_set(ConfigSetArgs),
}

// --- Telecommand Parsing ---
// TODO: Replace with meaningful telecommands
#[allow(clippy::result_unit_err)]
pub fn parse_telecommand(input: &str) -> Result<Telecommand, ()> {
    let command_name = input.trim().split('(').next().unwrap_or("");
    let command_args_str = input
        .trim()
        .strip_prefix(command_name)
        .and_then(|s| s.strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("")
        .trim();
    match command_name {
        "hello_world" => Ok(Telecommand::hello_world),
        "demo_command_with_arguments" => {
            let (args, _rest) =
                from_slice::<DemoCommandWithArgumentsArgs>(command_args_str.as_bytes())
                    .map_err(|_| ())?;
            Ok(Telecommand::demo_command_with_arguments(args))
        }
        "config_get" => {
            let (args, _rest) =
                from_slice::<ConfigGetArgs>(command_args_str.as_bytes()).map_err(|_| ())?;
            Ok(Telecommand::config_get(args))
        }
        "config_set" => {
            let (args, _rest) =
                from_slice::<ConfigSetArgs>(command_args_str.as_bytes()).map_err(|_| ())?;
            Ok(Telecommand::config_set(args))
        }
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(42, 42);
    }

    #[test]
    fn test_parse_telecommand_valid() {
        assert!(matches!(
            parse_telecommand("hello_world()"),
            Ok(Telecommand::hello_world)
        ));
        assert!(matches!(
            parse_telecommand(" hello_world() "),
            Ok(Telecommand::hello_world)
        ));
        assert!(matches!(
            parse_telecommand(
                r#"demo_command_with_arguments({
                    "arg_u32": 1,
                    "arg_u64": 2,
                    "arg_bool": true,
                    "arg_f32": 3.0,
                    "arg_f64": 4.0,
                    "arg_nullable_u32": null
                })"#
            ),
            Ok(Telecommand::demo_command_with_arguments(
                DemoCommandWithArgumentsArgs {
                    arg_u32: 1,
                    arg_u64: 2,
                    arg_bool: true,
                    arg_f32: 3.0,
                    arg_f64: 4.0,
                    arg_nullable_u32: None,
                }
            ))
        ));
    }

    #[test]
    fn test_parse_telecommand_invalid() {
        assert!(matches!(parse_telecommand("PINGS"), Err(())));
        assert!(matches!(parse_telecommand("PONGS"), Err(())));
        assert!(matches!(parse_telecommand(""), Err(())));
        assert!(matches!(parse_telecommand("LEDON"), Err(())));
        assert!(matches!(parse_telecommand("LEDOFF"), Err(())));
    }

    #[test]
    fn test_parse_json() {
        // Note: This is mostly a test of the serde_json_core library functionality.
        let json_data = r#"
        {
            "arg_u32": 123,
            "arg_u64": 45678901234,
            "arg_bool": true,
            "arg_f32": 3.14,
            "arg_f64": 2.718281828459045,
            "arg_nullable_u32": null
        }
        "#;

        let (parsed, _rest) =
            from_slice::<DemoCommandWithArgumentsArgs>(json_data.as_bytes()).unwrap();

        assert_eq!(parsed.arg_u32, 123);
        assert_eq!(parsed.arg_u64, 45678901234);
        assert_eq!(parsed.arg_bool, true);
        assert!((parsed.arg_f32 - 3.14).abs() < f32::EPSILON);
        assert!((parsed.arg_f64 - 2.718281828459045).abs() < f64::EPSILON);
        assert_eq!(parsed.arg_nullable_u32, None);
    }

    #[test]
    fn test_parse_demo_command_with_arguments() {
        let json_minified = r#"{"arg_u32":123,"arg_u64":45678901234,"arg_bool":true,"arg_f32":3.14,"arg_f64":2.718281828459045,"arg_nullable_u32":null}"#;

        let command_str = format!("demo_command_with_arguments({})", json_minified);
        let result = parse_telecommand(&command_str);
        assert!(matches!(
            result,
            Ok(Telecommand::demo_command_with_arguments(_))
        ));

        // Validate the parts inside the struct.
        assert!(
            if let Ok(Telecommand::demo_command_with_arguments(args)) = result {
                args.arg_u32 == 123
                    && args.arg_u64 == 45678901234
                    && args.arg_bool == true
                    && (args.arg_f32 - 3.14).abs() < f32::EPSILON
                    && (args.arg_f64 - 2.718281828459045).abs() < f64::EPSILON
                    && args.arg_nullable_u32.is_none()
            } else {
                false
            }
        );
    }
}
