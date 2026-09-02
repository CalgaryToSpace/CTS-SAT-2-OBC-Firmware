use cts2_obc_telecommands::DemoCommandWithArgumentsArgs;
use cts2_obc_telecommands::SendNameArgs;
use rtt_target::rprintln;

use crate::{error::ExecuteCommandErr, umbilical_uart::send_umbilical_uart};

pub fn run_hello_world_telecommand() -> Result<(), ExecuteCommandErr> {
    send_umbilical_uart(b"HELLO WORLD\r\n");

    Ok(())
}

pub fn run_demo_command_with_arguments(
    args: DemoCommandWithArgumentsArgs,
) -> Result<(), ExecuteCommandErr> {
    rprintln!(
        "DemoCommandWithArgumentsArgs: arg_u32={}, arg_u64={}, arg_bool={}, arg_f32={}, arg_f64={}, arg_nullable_u32={:?}\r\n",
        args.arg_u32,
        args.arg_u64,
        args.arg_bool,
        args.arg_f32,
        args.arg_f64,
        args.arg_nullable_u32
    );
    send_umbilical_uart(b"DEMO COMMAND WITH ARGUMENTS EXECUTED. See RTT output for details.\r\n");

    Ok(())
}

// Basic custom command
pub fn run_send_name_telecommand<'a>(entered_name: SendNameArgs<'a>,) -> Result<(), ExecuteCommandErr> {
    send_umbilical_uart(b"HELLO, MY NAME IS ");

    // Convert &str to all caps to be consistent
    for &c in entered_name.name.as_bytes() {
        let upper_char = if c >= b'a' && c <= b'z' {
            c - 32  // Subtract ASCII value to get uppercase equivalent
        } else {
            c
        };

        send_umbilical_uart(&[upper_char]);
    }

    send_umbilical_uart(b"\r\n");

    Ok(())
}