// Use Mutex to safely share peripherals across tasks/interrupt?
// Implement critical sections for safe access to shared resources?

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

mod config;
use config::ConfigStore;

use serde::{Deserialize, Serialize};
use serde_json_core::de::from_slice;

// global static singleton for configuration
static CONFIG_STORE: ConfigStore = ConfigStore::new();

// get reference to the global configuration store
pub fn get_config_store() -> &'static ConfigStore {
    &CONFIG_STORE
}

// --- Existing Telecommand Code ---
#[derive(Debug, Deserialize, Serialize)]

pub struct DemoCommandWithArgumentsArgs {
    pub arg_u32: u32,
    pub arg_u64: u64,
    pub arg_bool: bool,
    pub arg_f32: f32,
    pub arg_f64: f64,
    pub arg_nullable_u32: Option<u32>,
}

// TODO:Add more args for other telecommands as needed

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Telecommand {
    hello_world, // telecommand with no args
    demo_command_with_arguments(DemoCommandWithArgumentsArgs),
}

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
        // TODO: Add config_get and config_set telecommands that interact with the ConfigStore
        // "config_get" => {
        //     let (args, _rest) =
        //         from_slice::<ConfigGetArgs>(command_args_str.as_bytes()).map_err(|_| ())?;
        //     Ok(Telecommand::config_get(args))
        // }
        // "config_set" => {
        //     let (args, _rest) =
        //         from_slice::<ConfigSetArgs>(command_args_str.as_bytes()).map_err(|_| ())?;
        //     Ok(Telecommand::config_set(args))
        // }
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{ConfigVariableName, ConfigValue};

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
        assert!(matches!(parse_telecommand("PINGS"), Err(())));
        assert!(matches!(parse_telecommand("PONGS"), Err(())));
        assert!(matches!(parse_telecommand(""), Err(())));
        assert!(matches!(parse_telecommand("LEDON"), Err(())));
        assert!(matches!(parse_telecommand("LEDOFF"), Err(())));
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
