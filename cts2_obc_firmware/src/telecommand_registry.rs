use core::cell::RefCell;
use cortex_m::interrupt::{Mutex, free as critical_section};
use cts2_obc_logic::scheduler::{Priority, Task};

use crate::error::DispatchCommandErr;
use crate::scheduler_instance::SCHEDULER;
use crate::telecommand_implementation::demo_commands::*;
use crate::telecommand_implementation::*;
use cts2_obc_telecommands::telecommand_definitions::{ReadinessLevel, TelecommandDefinition};
use cts2_obc_telecommands::{EXECUTE_IMMEDIATELY, Telecommand, validators};

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
    TelecommandDefinition {
        name: "get_all_config_variables_jsonl",
        exec: get_all_config_variables_jsonl,
        num_parameters: 0,
        readiness: ReadinessLevel::Operation,
        validate: validators::no_args,
    },
    TelecommandDefinition {
        name: "get_obc_info",
        exec: get_obc_info_telecommand,
        num_parameters: 0,
        readiness: ReadinessLevel::Operation,
        validate: validators::no_args,
    },
];

// Number of recent command timestamps to remember.
const TIMESTAMP_HISTORY_SIZE: usize = 32;

struct TimestampHistory {
    timestamps: [Option<u64>; TIMESTAMP_HISTORY_SIZE],
    next: usize,
}

static TIMESTAMP_HISTORY: Mutex<RefCell<TimestampHistory>> =
    Mutex::new(RefCell::new(TimestampHistory {
        timestamps: [None; TIMESTAMP_HISTORY_SIZE],
        next: 0,
    }));

// Limitations:
// The history stores only 32 timestamps. Older entries are eventually overwritten
// Restarting the STM32 clears this history.
// Commands without ts_sent cannot be checked for duplicates.
// persistent duplicate prevention would need to be implemented
pub(crate) enum DispatchOutcome {
    Queued,
    Duplicate,
}

pub(crate) fn enqueue(cmd: Telecommand<'_>) -> Result<DispatchOutcome, DispatchCommandErr> {
    critical_section(|cs| {
        let mut history = TIMESTAMP_HISTORY.borrow(cs).borrow_mut();
        if cmd
            .ts_sent
            .is_some_and(|timestamp| history.timestamps.contains(&Some(timestamp)))
        {
            return Ok(DispatchOutcome::Duplicate);
        }

        let task = Task {
            name: cmd.def.name,
            execute: cmd.def.exec,
            args: heapless::String::try_from(cmd.args)
                .map_err(|_| DispatchCommandErr::ArgumentsTooLong)?,
            priority: Priority::Medium,
            tsexec: cmd.ts_exec.unwrap_or(EXECUTE_IMMEDIATELY),
        };
        SCHEDULER.borrow(cs).borrow_mut().add_task(task)?;

        if let Some(timestamp) = cmd.ts_sent {
            let index = history.next;
            history.timestamps[index] = Some(timestamp);
            history.next = (index + 1) % TIMESTAMP_HISTORY_SIZE;
        }

        Ok(DispatchOutcome::Queued)
    })
}

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
