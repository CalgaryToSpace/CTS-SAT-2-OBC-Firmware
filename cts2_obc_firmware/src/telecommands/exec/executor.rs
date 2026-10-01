use core::fmt::Write;
use cts2_obc_telecommands::parse_telecommand;

use super::registry::TELECOMMAND_DEFINITIONS;
use crate::error::DispatchCommandErr;
use crate::umbilical_uart::send_umbilical_uart;

// TODO: Make different functions to handle each separate command.
// TODO: Fix the () error type to be enum or string
// TODO: Replace with meaningful telecommands.
pub(crate) fn dispatch_command(cmd_str: &str) -> Result<(), DispatchCommandErr> {
    let cmd = match parse_telecommand(cmd_str, TELECOMMAND_DEFINITIONS) {
        Ok(cmd) => cmd,
        Err(err) => {
            send_uart_error(&err);
            return Err(err.into());
        }
    };

    if let Err(err) = (cmd.def.exec)(cmd.args) {
        send_uart_error(&err);
        return Err(err.into());
    }
    Ok(())
}

fn send_uart_error(err: &impl core::fmt::Display) {
    let mut msg = heapless::String::<128>::new();
    let _ = write!(msg, "ERR: {err}\r\n");
    send_umbilical_uart(msg.as_bytes());
}
