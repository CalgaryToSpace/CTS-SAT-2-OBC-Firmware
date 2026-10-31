use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt::{Mutex, free as critical_section};
use cts2_obc_logic::scheduler::{Priority, Task};
use cts2_obc_telecommands::{EXECUTE_IMMEDIATELY, Telecommand, parse_telecommand};

use super::registry::TELECOMMAND_DEFINITIONS;
use crate::error::DispatchCommandErr;
use crate::scheduler_instance::SCHEDULER;
use crate::umbilical_uart::send_umbilical_uart;

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

fn enqueue(cmd: Telecommand<'_>) -> Result<DispatchOutcome, DispatchCommandErr> {
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

pub(crate) fn dispatch_command(cmd_str: &str) -> Result<DispatchOutcome, DispatchCommandErr> {
    let cmd = match parse_telecommand(cmd_str, TELECOMMAND_DEFINITIONS) {
        Ok(cmd) => cmd,
        Err(err) => {
            send_uart_error(&err);
            return Err(err.into());
        }
    };

    match enqueue(cmd) {
        Ok(DispatchOutcome::Duplicate) => {
            send_umbilical_uart(b"ACK: Duplicate command ignored\r\n");
            Ok(DispatchOutcome::Duplicate)
        }
        Ok(outcome) => Ok(outcome),
        Err(err) => {
            send_uart_error(&err);
            Err(err)
        }
    }
}

pub(crate) fn send_uart_error(err: &impl core::fmt::Display) {
    let mut msg = heapless::String::<128>::new();
    let _ = write!(msg, "ERR: {err}\r\n");
    send_umbilical_uart(msg.as_bytes());
}
