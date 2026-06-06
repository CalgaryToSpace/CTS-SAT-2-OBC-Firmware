#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;

extern crate cortex_m;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt::free as critical_section;
use heapless::index_map::FnvIndexMap;
use serde::{Deserialize, Serialize};
use serde_json_core::de::from_slice;

// IndexMap to hold configuration variables. HashMaps cannot be used for embedded systems since they 
// are dynamically allocated; IndexMaps are a heapless alternative with a fixed capacity.
// There will need to be a separate IndexMap for each variable type (u32, u64, bool, ...)
// https://docs.rust-embedded.org/book/collections/
pub static CONFIG_U32_VARIABLES: Mutex<RefCell<FnvIndexMap<ConfigVariable, u32, 2>>> =
    Mutex::new(RefCell::new(FnvIndexMap::new()));

// Enum of all configuration variable names
#[derive(Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum ConfigVariable {
    heartbeat_ms,
    config_demo_variable1,
    // TODO: Add more configuration variables here
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DemoCommandWithArgumentsArgs {
    pub arg_u32: u32,
    pub arg_u64: u64,
    pub arg_bool: bool,
    pub arg_f32: f32,
    pub arg_f64: f64,
    pub arg_nullable_u32: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TCMDConfigSetU32VarArgs {
    pub arg_var_name: ConfigVariable,
    pub arg_u32: u32,
}

#[derive(Debug)]
#[allow(non_camel_case_types)] // Allow telecommand names that align with their function names.
pub enum Telecommand {
    hello_world,
    demo_command_with_arguments(DemoCommandWithArgumentsArgs),
    tcmd_config_set_u32_var(TCMDConfigSetU32VarArgs),               // Adding a setter telecommand to test methods (unfinished implementation).
}

// TODO: Replace with meaningful telecommands
#[allow(clippy::result_unit_err)] // TODO: Fix the () error type to be enum or string
pub fn parse_telecommand(input: &str) -> Result<Telecommand, ()> {
    // Extract string before the first '(' to identify the command.
    let command_name = input.trim().split('(').next().unwrap_or("");

    // Extract arguments string between parentheses, if any.
    let command_args_str = input
        .trim()
        .strip_prefix(command_name)
        .and_then(|s| s.strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("")
        .trim();

    match command_name.trim() {
        "hello_world" => Ok(Telecommand::hello_world),
        "demo_command_with_arguments" => {
            let (args, _rest) =
                from_slice::<DemoCommandWithArgumentsArgs>(command_args_str.as_bytes())
                    .map_err(|_| ())?;
            Ok(Telecommand::demo_command_with_arguments(args))
        }
        "tcmd_config_set_u32_var" => {
            let (args, _rest) =
                from_slice::<TCMDConfigSetU32VarArgs>(command_args_str.as_bytes())
                    .map_err(|_| ())?;
            Ok(Telecommand::tcmd_config_set_u32_var(args))
        }
        _ => Err(()),
    }
}

// The following functions will need to be modified if the location of the CONFIG_U32_VARIABLES IndexMap changes.
// This file might not be the best location for it.

// The idea was to run this at the beginning of the program to configure initial values of the variables
// and add them to the IndexMap. There may be a better way.
pub fn config_all_u32() {
    critical_section(|cs| {
        let mut config_u32 = CONFIG_U32_VARIABLES.borrow(cs).borrow_mut();
        config_u32.insert(ConfigVariable::heartbeat_ms, 500).unwrap();
        config_u32.insert(ConfigVariable::config_demo_variable1, 12345).unwrap();
    });
}

// Set function without Result<(), ()> return type
// Called within the attempted telecommand implementation in cts2_obc_firmware\src\telecommand_implementation\demo_commands.rs
pub fn config_set_u32_variable(var_name: ConfigVariable, new_value: u32)  {    
    critical_section(|cs| {
        if let Some(value) = CONFIG_U32_VARIABLES.borrow(cs).borrow_mut().get_mut(&var_name) {
            *value = new_value;
        }
    });
}

// Should eventually be called within the getter telecommand implementation.
pub fn config_get_u32_variable(var_name: ConfigVariable) -> Option<u32> {
    critical_section(|cs| {
        CONFIG_U32_VARIABLES.borrow(cs).borrow().get(&var_name).copied()
    })
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
