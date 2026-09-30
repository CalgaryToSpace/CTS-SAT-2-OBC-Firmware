use crate::telecommand_implementation::{
    demo_commands::run_hello_world_telecommand, get_config_variable, get_sys_uptime_ms_telecommand,
    get_unix_time_telecommand, set_config_variable, set_unix_time_telecommand,
};
use cts2_obc_telecommands::telecommand_definitions::{ReadinessLevel, TelecommandDefinition};
use cts2_obc_telecommands::validators;

// For now, each command has an appropriate validator
// that checks whether the appropriate argument exists, its type, its range, etc.
// For example, hello_world accepts no commands at all
// So its validator is the no_args that ensures there are no arguments being sent
// Into the hello_world command.
// In contrasts, the set_config needs to validate that the argument is of type u32
// that the argument exists and not empty, etc.
pub const TELECOMMAND_DEFINITIONS: &[TelecommandDefinition] = &[
    TelecommandDefinition {
        name: "hello_world",
        exec: run_hello_world_telecommand,
        num_parameters: 0,
        readiness: ReadinessLevel::Operation,
        validate: validators::no_args,
    },
    TelecommandDefinition {
        name: "get_sys_uptime",
        exec: get_sys_uptime_ms_telecommand,
        num_parameters: 0,
        readiness: ReadinessLevel::Operation,
        validate: validators::no_args,
    },
    TelecommandDefinition {
        name: "get_config",
        exec: get_config_variable,
        num_parameters: 1,
        readiness: ReadinessLevel::Operation,
        validate: validators::get_config,
    },
    TelecommandDefinition {
        name: "set_config",
        exec: set_config_variable,
        num_parameters: 2,
        readiness: ReadinessLevel::Operation,
        validate: validators::set_config,
    },
    TelecommandDefinition {
        name: "set_unix_time",
        exec: set_unix_time_telecommand,
        num_parameters: 1,
        readiness: ReadinessLevel::GroundUsage,
        validate: validators::unix_milliseconds,
    },
    TelecommandDefinition {
        name: "get_unix_time",
        exec: get_unix_time_telecommand,
        num_parameters: 0,
        readiness: ReadinessLevel::GroundUsage,
        validate: validators::no_args,
    },
];

const fn str_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut i = 0;
    while i < a_bytes.len() {
        if a_bytes[i] != b_bytes[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn assert_unique() {
    let mut i = 0;
    while i < TELECOMMAND_DEFINITIONS.len() {
        let name_i = TELECOMMAND_DEFINITIONS[i].name;
        let mut j = i + 1;
        while j < TELECOMMAND_DEFINITIONS.len() {
            let name_j = TELECOMMAND_DEFINITIONS[j].name;
            if str_eq(name_i, name_j) {
                panic!("Duplicate telecommand name");
            }
            j += 1;
        }
        i += 1;
    }
}

// Insert all compile time checks here.
// This will run at compile time
const _: () = assert_unique();
