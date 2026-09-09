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
    use crate::config::{ConfigValue, ConfigVariableName};
    use crate::error::ConfigError;
    use crate::telecommand_definitions::ReadinessLevel;
    use core::str::FromStr;

    const SAMPLE_DEFINITIONS: &[TelecommandDefinition] = &[
        TelecommandDefinition {
            name: "sample",
            num_parameters: 2,
            readiness: ReadinessLevel::GroundUsage,
            exec: |_| panic!("Parsing must not execute commands"),
        },
        TelecommandDefinition {
            name: "sample_empty",
            num_parameters: 0,
            readiness: ReadinessLevel::GroundUsage,
            exec: |_| panic!("Parsing must not execute commands"),
        },
    ];

    #[test]
    fn test_parse_unknown_command() {
        assert_eq!(
            parse_telecommand("unknown(1,2)", SAMPLE_DEFINITIONS).err(),
            Some(ParsedTelecommandErr::UnknownCommand),
        );
    }

    #[test]
    fn test_parse_missing_argument() {
        for (input, index) in [
            ("sample()", 0),
            ("sample(   )", 0),
            ("sample(first)", 1),
            ("sample(,second)", 0),
            ("sample(first,)", 1),
            ("sample(first,   )", 1),
            ("sample( , )", 0),
        ] {
            assert_eq!(
                parse_telecommand(input, SAMPLE_DEFINITIONS).err(),
                Some(ParsedTelecommandErr::MissingArgument(index)),
                "input: {input}",
            );
        }
    }

    #[test]
    fn test_parse_extra_arguments() {
        for input in ["sample(1,2,3)", "sample(1,2,)"] {
            assert_eq!(
                parse_telecommand(input, SAMPLE_DEFINITIONS).err(),
                Some(ParsedTelecommandErr::ExceededArgumentCount),
                "input: {input}",
            );
        }
    }

    #[test]
    fn test_parse_valid_sample_arguments() {
        for (input, name, args) in [
            ("sample(first, second)", "sample", "first, second"),
            ("sample_empty()", "sample_empty", ""),
        ] {
            let cmd = parse_telecommand(input, SAMPLE_DEFINITIONS).unwrap();
            assert_eq!(cmd.def.name, name);
            assert_eq!(cmd.args, args);
        }
    }

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
    fn test_extract_command_without_arguments() {
        assert_eq!(extract_function_and_args("hello_world()"), ("hello_world", ""));
        assert_eq!(extract_function_and_args(" hello_world() "), ("hello_world", ""));
    }

    #[test]
    fn test_extract_config_arguments() {
        assert_eq!(
            extract_function_and_args("get_config(config_demo_variable1)"),
            ("get_config", "config_demo_variable1"),
        );
        assert_eq!(
            extract_function_and_args("set_config(config_demo_variable1, u32(8386))"),
            ("set_config", "config_demo_variable1, u32(8386)"),
        );
    }

    #[test]
    fn test_extract_empty_input() {
        assert_eq!(extract_function_and_args(""), ("", ""));
        assert_eq!(extract_function_and_args("   "), ("", ""));
    }

    #[test]
    fn test_parse_with_empty_registry() {
        for input in ["PONGS", "PINGS", ""] {
            assert_eq!(
                parse_telecommand(input, &[]).err(),
                Some(ParsedTelecommandErr::UnknownCommand),
            );
        }
    }
}
