// Use Mutex to safely share peripherals across tasks/interrupt?
// Implement critical sections for safe access to shared resources?

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

pub mod config;
use config::ConfigStore;

pub mod error;
use error::ParsedTelecommandErr;

pub mod telecommand_definitions;
use telecommand_definitions::TelecommandDefinition;

mod shared;
use shared::extract_function_and_args;

// global static singleton for configuration
static CONFIG_STORE: ConfigStore = ConfigStore::new();

// get reference to the global configuration store
pub fn get_config_store() -> &'static ConfigStore {
    &CONFIG_STORE
}

// TODO:Add more args for other telecommands as needed

pub struct Telecommand<'a> {
    pub def: &'static TelecommandDefinition,
    pub args: &'a str,
}

// TODO: Replace with meaningful telecommands
#[allow(clippy::result_unit_err)] // TODO: Fix the () error type to be enum or string
pub fn parse_telecommand<'a>(
    input: &'a str,
    telecommand_definitions: &'static [TelecommandDefinition],
) -> Result<Telecommand<'a>, ParsedTelecommandErr> {
    // Extract string before the first '(' to identify the command.
    let (command_name, command_args_str) = extract_function_and_args(input);
    let args = command_args_str.split(',');
    let count = if command_args_str.is_empty() {
        0
    } else {
        args.clone().count()
    };

    for tcmd_def in telecommand_definitions.iter() {
        if tcmd_def.name == command_name {
            if count < usize::from(tcmd_def.num_parameters) {
                return Err(ParsedTelecommandErr::MissingArgument(count as u8));
            }
            if count > usize::from(tcmd_def.num_parameters) {
                return Err(ParsedTelecommandErr::ExceededArgumentCount);
            }
            if count > 0 {
                for (index, arg) in args.enumerate() {
                    if arg.trim().is_empty() {
                        return Err(ParsedTelecommandErr::MissingArgument(index as u8));
                    }
                }
            }
            return Ok(Telecommand {
                def: tcmd_def,
                args: command_args_str,
            });
        }
    }

    Err(ParsedTelecommandErr::UnknownCommand)
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
    fn test_parse_get_config() {
        let result = parse_telecommand("get_config(config_demo_variable1)");
        assert!(matches!(
            result,
            Ok(Telecommand::get_config(
                ConfigVariableName::ConfigDemoVariable1
            ))
        ));
    }

    #[test]
    fn test_parse_set_config() {
        let result = parse_telecommand("set_config(config_demo_variable1, u32(8386))");
        assert!(matches!(
            result,
            Ok(Telecommand::set_config(
                ConfigVariableName::ConfigDemoVariable1,
                ConfigValue::U32(8386)
            ))
        ));
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
