use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt::{Mutex, free as critical_section};
use cts2_obc_telecommands::parse_telecommand;

use super::registry::TELECOMMAND_DEFINITIONS;
use crate::error::DispatchCommandErr;
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

// Returns false if this timestamp has already been received.
// Otherwise, records it and returns true.
fn claim_timestamp(timestamp: u64) -> bool {
    critical_section(|cs| {
        let mut history = TIMESTAMP_HISTORY.borrow(cs).borrow_mut();

        if history.timestamps.contains(&Some(timestamp)) {
            return false;
        }

        let index = history.next;
        history.timestamps[index] = Some(timestamp);
        history.next = (index + 1) % TIMESTAMP_HISTORY_SIZE;

        true
    })
}

// TODO: Make different functions to handle each separate command.
// TODO: Fix the () error type to be enum or string
// TODO: Replace with meaningful telecommands.
// Limitations:
// The history stores only 32 timestamps. Older entries are eventually overwritten
// Restarting the STM32 clears this history.
// Commands without ts_sent cannot be checked for duplicates.
// persistent duplicate prevention would need to be implemented
pub(crate) enum DispatchOutcome {
    Executed,
    Duplicate,
    DelayedNotReady,
}

pub(crate) fn dispatch_command(cmd_str: &str) -> Result<DispatchOutcome, DispatchCommandErr> {
    let cmd = match parse_telecommand(cmd_str, TELECOMMAND_DEFINITIONS) {
        Ok(cmd) => cmd,
        Err(err) => {
            send_uart_error(&err);
            return Err(err.into());
        }
    };

    // Reject delayed commands until scheduling is implemented.
    // An absent timestamp or timestamp 0 means immediate execution.
    if cmd.ts_exec.is_some_and(|ts| ts != 0) {
        send_umbilical_uart(b"ERR: Delayed execution not implemented\r\n");

        return Ok(DispatchOutcome::DelayedNotReady);
    }

    // Check whether the command has already been received.
    if let Some(timestamp) = cmd.ts_sent
        && !claim_timestamp(timestamp)
    {
        send_umbilical_uart(b"ACK: Duplicate command ignored\r\n");

        return Ok(DispatchOutcome::Duplicate);
    }

    // Execute the registered command.
    if let Err(err) = (cmd.def.exec)(cmd.args) {
        send_uart_error(&err);
        return Err(err.into());
    }

    Ok(DispatchOutcome::Executed)
}

fn send_uart_error(err: &impl core::fmt::Display) {
    let mut msg = heapless::String::<128>::new();
    let _ = write!(msg, "ERR: {err}\r\n");
    send_umbilical_uart(msg.as_bytes());
}
