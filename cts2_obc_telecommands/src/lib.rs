// Use Mutex to safely share peripherals across tasks/interrupt?
// Implement critical sections for safe access to shared resources?

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

pub mod config;
use config::{CONFIG_STORE, ConfigStore};

pub mod error;
use error::ParsedTelecommandErr;

pub mod telecommand_definitions;
use telecommand_definitions::TelecommandDefinition;

mod shared;
use shared::{extract_function_and_args, has_valid_command_structure};

pub mod validators;

// get reference to the global configuration store
pub fn get_config_store() -> &'static ConfigStore {
    &CONFIG_STORE
}

// TODO:Add more args for other telecommands as needed

pub struct Telecommand<'a> {
    pub def: &'static TelecommandDefinition,
    pub args: &'a str,
    pub ts_sent: Option<u64>,
    pub ts_exec: Option<u64>,
    pub resp_fname: Option<&'a str>,
}

// TODO: Replace with meaningful telecommands
// A very important note. Every command needs to end with a !
// For the parser to acknowledge it, meaning that when sending
// data through SerialTest, the parse_telecommand will accept any strings
// until a '!' is sent then it interprets everything before it as a command
// This is because '!' acts as a delimiter
pub fn parse_telecommand<'a>(
    input: &'a str,
    telecommand_definitions: &'static [TelecommandDefinition],
) -> Result<Telecommand<'a>, ParsedTelecommandErr> {
    // Removes any leading and trailing white spaces
    let input = input.trim();

    // Telecommand must start with "CTS2+"
    let input = input
        .strip_prefix("CTS2+")
        .ok_or(ParsedTelecommandErr::InvalidPrefix)?;

    // Telecommand must end with '!'
    let input = input
        .strip_suffix('!')
        .ok_or(ParsedTelecommandErr::MissingEndMarker)?;

    // We then separate the command suffixes like
    // @tssent or @tsexec by splitting the string at '@'
    let mut sections = input.split('@');

    let command_part = sections
        .next()
        .ok_or(ParsedTelecommandErr::UnknownCommand)?;

    let mut ts_sent = None;
    let mut ts_exec = None;
    let mut resp_fname = None;

    // Validate suffixes
    for suffix in sections {
        if let Some(value) = suffix.strip_prefix("tssent=") {
            ts_sent = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| ParsedTelecommandErr::InvalidTimestamp)?,
            );
        } else if let Some(value) = suffix.strip_prefix("tsexec=") {
            ts_exec = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| ParsedTelecommandErr::InvalidTimestamp)?,
            );
        } else if let Some(value) = suffix.strip_prefix("resp_fname=") {
            if value.is_empty() || value.len() > 63 {
                return Err(ParsedTelecommandErr::InvalidResponseFilename);
            }

            resp_fname = Some(value);
        } else {
            return Err(ParsedTelecommandErr::InvalidSuffix);
        }
    }

    if !has_valid_command_structure(command_part) {
        return Err(ParsedTelecommandErr::InvalidCommandFormat);
    }

    // Examine arguments before extract_function_and_args()
    // removes leading and trailing whitespace.
    let open = command_part
        .find('(')
        .ok_or(ParsedTelecommandErr::InvalidCommandFormat)?;

    let raw_args = &command_part[open + 1..command_part.len() - 1];

    // Preserve MissingArgument errors for empty arguments.
    // Reject whitespace in otherwise complete arguments.
    if raw_args.split(',').all(|arg| !arg.trim().is_empty())
        && raw_args.chars().any(char::is_whitespace)
    {
        return Err(ParsedTelecommandErr::SpacesNotAllowed);
    }

    // Extract command name and argument string
    let (command_name, command_args_str) = extract_function_and_args(command_part);

    // Specification says no spaces between arguments
    if command_args_str.contains(' ') {
        return Err(ParsedTelecommandErr::SpacesNotAllowed);
    }

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

            (tcmd_def.validate)(command_args_str)?;
            return Ok(Telecommand {
                def: tcmd_def,
                args: command_args_str,
                ts_sent,
                ts_exec,
                resp_fname,
            });
        }
    }

    Err(ParsedTelecommandErr::UnknownCommand)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CONFIG_DEMO_VARIABLE1, ConfigValue, HEARTBEAT_MS};
    use crate::error::ConfigError;
    use crate::telecommand_definitions::ReadinessLevel;
    use core::str::FromStr;

    fn validate_sample(_args: &str) -> Result<(), ParsedTelecommandErr> {
        Ok(())
    }

    const SAMPLE_DEFINITIONS: &[TelecommandDefinition] = &[
        TelecommandDefinition {
            name: "sample",
            num_parameters: 2,
            readiness: ReadinessLevel::GroundUsage,
            exec: |_| panic!("Parsing must not execute commands"),
            validate: validate_sample,
        },
        TelecommandDefinition {
            name: "sample_empty",
            num_parameters: 0,
            readiness: ReadinessLevel::GroundUsage,
            exec: |_| panic!("Parsing must not execute commands"),
            validate: validators::no_args,
        },
    ];

    const VALIDATION_DEFINITIONS: &[TelecommandDefinition] = &[TelecommandDefinition {
        name: "set_config",
        num_parameters: 2,
        readiness: ReadinessLevel::Operation,
        exec: |_| panic!("Parser must not execute commands"),
        validate: validators::set_config,
    }];

    #[test]
    fn test_config_argument_validation() {
        let valid = "CTS2+set_config(heartbeat_ms,u32(1000))!";

        assert!(parse_telecommand(valid, VALIDATION_DEFINITIONS).is_ok());

        for (input, expected_error) in [
            (
                "CTS2+set_config(unknown,u32(1000))!",
                ParsedTelecommandErr::InvalidArgumentValue(0),
            ),
            (
                "CTS2+set_config(heartbeat_ms,bool(true))!",
                ParsedTelecommandErr::InvalidArgumentType(1),
            ),
            (
                "CTS2+set_config(heartbeat_ms,u32(abc))!",
                ParsedTelecommandErr::InvalidArgumentValue(1),
            ),
            (
                "CTS2+set_config(heartbeat_ms,u32(4294967296))!",
                ParsedTelecommandErr::InvalidArgumentRange(1),
            ),
        ] {
            assert_eq!(
                parse_telecommand(input, VALIDATION_DEFINITIONS).err(),
                Some(expected_error),
                "input: {input}",
            );
        }
    }

    #[test]
    fn test_parse_cts2_command() {
        let cmd = parse_telecommand("CTS2+sample(first,second)!", SAMPLE_DEFINITIONS).unwrap();

        assert_eq!(cmd.def.name, "sample");
        assert_eq!(cmd.args, "first,second");
    }

    #[test]
    fn test_parse_cts2_with_tssent() {
        let cmd = parse_telecommand(
            "CTS2+sample(first,second)@tssent=1716611908453!",
            SAMPLE_DEFINITIONS,
        )
        .unwrap();

        assert_eq!(cmd.def.name, "sample");
        assert_eq!(cmd.args, "first,second");
    }

    #[test]
    fn test_parse_cts2_with_all_suffixes() {
        let cmd = parse_telecommand(
        "CTS2+sample(first,second)@tssent=1716611908453@tsexec=1716611999999@resp_fname=test.txt!",
        SAMPLE_DEFINITIONS,
    )
    .unwrap();

        assert_eq!(cmd.def.name, "sample");
        assert_eq!(cmd.args, "first,second");

        assert_eq!(cmd.ts_sent, Some(1716611908453));
        assert_eq!(cmd.ts_exec, Some(1716611999999));
        assert_eq!(cmd.resp_fname, Some("test.txt"));
    }

    #[test]
    fn test_parse_cts2_empty_command() {
        let cmd = parse_telecommand("CTS2+sample_empty()!", SAMPLE_DEFINITIONS).unwrap();

        assert_eq!(cmd.def.name, "sample_empty");
        assert_eq!(cmd.args, "");

        assert_eq!(cmd.ts_sent, None);
        assert_eq!(cmd.ts_exec, None);
        assert_eq!(cmd.resp_fname, None);
    }
    #[test]
    fn test_parse_unknown_command() {
        assert_eq!(
            parse_telecommand("CTS2+unknown(1,2)!", SAMPLE_DEFINITIONS).err(),
            Some(ParsedTelecommandErr::UnknownCommand),
        );
    }

    #[test]
    fn test_parse_missing_argument() {
        for (input, index) in [
            ("CTS2+sample()!", 0),
            ("CTS2+sample(first)!", 1),
            ("CTS2+sample(,second)!", 0),
            ("CTS2+sample(first,)!", 1),
            ("CTS2+sample(   )!", 0),
            ("CTS2+sample(first,   )!", 1),
            ("CTS2+sample( , )!", 0),
        ] {
            assert_eq!(
                parse_telecommand(input, SAMPLE_DEFINITIONS).err(),
                Some(ParsedTelecommandErr::MissingArgument(index)),
                "input: {input}",
            );
        }
    }

    #[test]
    fn test_parse_spaces_not_allowed() {
        for input in [
            "CTS2+sample(first, second)!",
            "CTS2+sample(fi rst,second)!",
            "CTS2+sample(first,sec ond)!",
        ] {
            assert_eq!(
                parse_telecommand(input, SAMPLE_DEFINITIONS).err(),
                Some(ParsedTelecommandErr::SpacesNotAllowed),
                "input: {input}",
            );
        }
    }

    #[test]
    fn test_parse_extra_arguments() {
        for input in ["CTS2+sample(1,2,3)!", "CTS2+sample(1,2,)!"] {
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
            ("CTS2+sample(first,second)!", "sample", "first,second"),
            ("CTS2+sample_empty()!", "sample_empty", ""),
        ] {
            let cmd = parse_telecommand(input, SAMPLE_DEFINITIONS).unwrap();

            assert_eq!(cmd.def.name, name);
            assert_eq!(cmd.args, args);
        }
    }

    #[test]
    fn test_config_store_get_set() {
        let store = get_config_store();

        // Test default values
        assert_eq!(
            store.get(HEARTBEAT_MS.name).unwrap(),
            ConfigValue::U32(1000)
        );
        assert_eq!(
            store.get(CONFIG_DEMO_VARIABLE1.name).unwrap(),
            ConfigValue::U32(123)
        );

        // Test setting values
        assert!(store.set(HEARTBEAT_MS.name, ConfigValue::U32(2000)).is_ok());
        assert_eq!(
            store.get(HEARTBEAT_MS.name).unwrap(),
            ConfigValue::U32(2000)
        );

        assert!(
            store
                .set(CONFIG_DEMO_VARIABLE1.name, ConfigValue::U32(42))
                .is_ok()
        );
        assert_eq!(
            store.get(CONFIG_DEMO_VARIABLE1.name).unwrap(),
            ConfigValue::U32(42)
        );
    }

    #[test]
    fn test_config_store_set_type_mismatch() {
        let store = get_config_store();

        let result = store.set(CONFIG_DEMO_VARIABLE1.name, ConfigValue::F32(42.0));

        assert_eq!(result, Err(ConfigError::ConfigVariableNotThisType));
    }

    #[test]
    fn test_config_store_parse_unknown_variable() {
        let result = get_config_store().get("unknown_variable");
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

        store.set(HEARTBEAT_MS.name, ConfigValue::U32(500)).unwrap();
        assert_eq!(store.get(HEARTBEAT_MS.name).unwrap(), ConfigValue::U32(500));
    }

    #[test]
    fn test_extract_command_without_arguments() {
        assert_eq!(
            extract_function_and_args("hello_world()"),
            ("hello_world", "")
        );
        assert_eq!(
            extract_function_and_args(" hello_world() "),
            ("hello_world", "")
        );
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
        for input in ["CTS2+PONGS()!", "CTS2+PINGS()!", "CTS2+unknown()!"] {
            assert_eq!(
                parse_telecommand(input, &[]).err(),
                Some(ParsedTelecommandErr::UnknownCommand),
            );
        }
    }

    #[test]
    fn test_parse_leading_trailing_whitespace() {
        let cmd =
            parse_telecommand(" \r\nCTS2+sample(first,second)!\r\n ", SAMPLE_DEFINITIONS).unwrap();

        assert_eq!(cmd.def.name, "sample");
        assert_eq!(cmd.args, "first,second");
    }

    #[test]
    fn test_invalid_command_structure() {
        for input in [
            "CTS2+sample_empty!",
            "CTS2+sample_empty(!",
            "CTS2+sample_empty()extra!",
            "CTS2+sample_empty())!",
            "CTS2+sample_empty()sample_empty()!",
        ] {
            assert_eq!(
                parse_telecommand(input, SAMPLE_DEFINITIONS).err(),
                Some(ParsedTelecommandErr::InvalidCommandFormat),
                "input: {input}",
            );
        }
    }

    #[test]
    fn test_argument_boundary_whitespace() {
        for input in [
            "CTS2+sample( first,second)!",
            "CTS2+sample(first,second )!",
            "CTS2+sample(first, second)!",
        ] {
            assert_eq!(
                parse_telecommand(input, SAMPLE_DEFINITIONS).err(),
                Some(ParsedTelecommandErr::SpacesNotAllowed),
                "input: {input}",
            );
        }
    }
}
