// Use Mutex to safely share peripherals across tasks/interrupt?
// Implement critical sections for safe access to shared resources?

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

mod config;
use config::{ConfigStore, ConfigValue, ConfigVariableName};

pub mod error;
use error::{ConfigError, ParsedTelecommandErr};

mod shared;
use shared::extract_function_and_args;

use core::str::FromStr;
use serde::{Deserialize, Serialize};
use serde_json_core::de::from_slice;

// global static singleton for configuration
static CONFIG_STORE: ConfigStore = ConfigStore::new();

// get reference to the global configuration store
pub fn get_config_store() -> &'static ConfigStore {
    &CONFIG_STORE
}

// --- Existing Telecommand Code ---
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DemoCommandWithArgumentsArgs {
    pub arg_u32: u32,
    pub arg_u64: u64,
    pub arg_bool: bool,
    pub arg_f32: f32,
    pub arg_f64: f64,
    pub arg_nullable_u32: Option<u32>,
}

// TODO:Add more args for other telecommands as needed

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)] // Allow telecommand names that align with their function names.
pub enum Telecommand {
    hello_world, // telecommand with no args
    get_sys_uptime,
    demo_command_with_arguments(DemoCommandWithArgumentsArgs),
    config_get(ConfigVariableName),
    config_set(ConfigVariableName, ConfigValue),
}

// TODO: Replace with meaningful telecommands
#[allow(clippy::result_unit_err)] // TODO: Fix the () error type to be enum or string
pub fn parse_telecommand(input: &str) -> Result<Telecommand, ParsedTelecommandErr> {
    // Extract string before the first '(' to identify the command.
    let (command_name, command_args_str) = extract_function_and_args(input);

    let mut parts = command_args_str.split(',').map(|s| s.trim());
    match command_name {
        "hello_world" => Ok(Telecommand::hello_world),
        "demo_command_with_arguments" => {
            let (args, _rest) =
                from_slice::<DemoCommandWithArgumentsArgs>(command_args_str.as_bytes())
                    .map_err(ParsedTelecommandErr::DeserializationError)?;
            Ok(Telecommand::demo_command_with_arguments(args))
        }
        "get_sys_uptime" => Ok(Telecommand::get_sys_uptime),
        "config_get" => {
            let name_str = parts
                .next()
                .ok_or(ParsedTelecommandErr::MissingArgument(0))?;
            let name_enum = ConfigVariableName::from_str(name_str).map_err(|_| {
                ParsedTelecommandErr::ConfigError(ConfigError::ConfigVariableNotFound)
            })?;
            if parts.next().is_some() {
                return Err(ParsedTelecommandErr::ExceededArgumentCount);
            }

            Ok(Telecommand::config_get(name_enum))
        }
        "config_set" => {
            let name_str = parts
                .next()
                .ok_or(ParsedTelecommandErr::MissingArgument(0))?;
            let name_enum = ConfigVariableName::from_str(name_str)
                .map_err(ParsedTelecommandErr::ConfigError)?;

            let value_str = parts
                .next()
                .ok_or(ParsedTelecommandErr::MissingArgument(1))?;
            let value_enum =
                ConfigValue::from_str(value_str).map_err(ParsedTelecommandErr::ConfigError)?;

            Ok(Telecommand::config_set(name_enum, value_enum))
        }
        _ => Err(ParsedTelecommandErr::UnknownCommand),
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
            ConfigValue::U32(0)
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
        assert_eq!(
            parse_telecommand("PINGS"),
            Err(ParsedTelecommandErr::UnknownCommand)
        );
        assert_eq!(
            parse_telecommand("PONGS"),
            Err(ParsedTelecommandErr::UnknownCommand)
        );
        assert_eq!(
            parse_telecommand(""),
            Err(ParsedTelecommandErr::UnknownCommand)
        );
        assert_eq!(
            parse_telecommand("LEDON"),
            Err(ParsedTelecommandErr::UnknownCommand)
        );
        assert_eq!(
            parse_telecommand("LEDOFF"),
            Err(ParsedTelecommandErr::UnknownCommand)
        );
        assert_eq!(
            parse_telecommand("demo_command_with_arguments({invalid_json})"),
            Err(ParsedTelecommandErr::DeserializationError(
                serde_json_core::de::Error::KeyMustBeAString
            ))
        );
    }

    #[test]
    fn test_parse_json() {
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
