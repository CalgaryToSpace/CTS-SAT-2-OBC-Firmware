use cts2_obc_telecommands::{DemoCommandWithArgumentsArgs, TCMDConfigSetU32VarArgs};
use cts2_obc_telecommands::{config_set_u32_variable, config_get_u32_variable};
use rtt_target::rprintln;

use crate::umbilical_uart::send_umbilical_uart;

pub fn run_hello_world_telecommand() -> Result<(), ()> {
    send_umbilical_uart(b"HELLO WORLD\r\n");

    Ok(())
}

pub fn run_demo_command_with_arguments(args: DemoCommandWithArgumentsArgs) -> Result<(), ()> {
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

/* 
 * UNFINISHED IMPLEMENTATION
 * cts2_obc_telecommands\src\lib.rs: 
 *      added a struct TCMDConfigSetU32VarArgs
 *      added to enum Telecommand: tcmd_config_set_u32_var(TCMDConfigSetU32VarArgs)
 *      modified parse_telecommand to include "tcmd_config_set_u32_var"
 * cts2_obc_firmware\src\umbilical_uart.rs:
 *      called this function (run_tcmd_config_set_u32_var) in the dispatch_command function
 */
pub fn run_tcmd_config_set_u32_var(args: TCMDConfigSetU32VarArgs) -> Result<(), ()> {
    rprintln!("TCMDConfigSetU32Vars: arg_var_name={:?}, arg_u32={}\r\n", args.arg_var_name, args.arg_u32);
    config_set_u32_variable(args.arg_var_name, args.arg_u32);
    send_umbilical_uart(b"TCMD CONFIG SET U32 VAR EXECUTED.\r\n");

    Ok(())
}