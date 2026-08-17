pub mod demo_commands;

use core::fmt::Write;
use crate::error::ExecuteCommandErr;
use crate::timekeeping::uptime_ms;
use crate::umbilical_uart::send_umbilical_uart;
use cts2_obc_telecommands::config::{ConfigValue, ConfigVariableName};
use cts2_obc_telecommands::get_config_store;
use crate::telecommand_implementation::demo_commands::{
    DEMO_ARGS, run_demo_command_with_arguments, run_hello_world_telecommand,
};
use cts2_obc_logic::scheduler::TaskArgs;

pub fn telecommand_hello_world(_args: TaskArgs) -> Result<(), ExecuteCommandErr> {
    run_hello_world_telecommand()
}

pub fn telecommand_demo_command_with_arguments(_args: TaskArgs) -> Result<(), ExecuteCommandErr> {
    use cortex_m::interrupt::free as critical_section;
    if let Some(args) = critical_section(|cs| DEMO_ARGS.borrow(cs).borrow_mut().take()) {
        return run_demo_command_with_arguments(args);
    }
    // Could throw an error but this is just a demo command, so we can just return Ok
    // could add an error type for this case if we want to be more strict
    Ok(())
}

pub fn telecommand_get_sys_uptime(_args: TaskArgs) -> Result<(), ExecuteCommandErr> {
    let sys_time = uptime_ms();
    let buff = heapless::format!(32; "System Uptime: {} ms\r\n", sys_time)
        .unwrap()
        .into_bytes();
    send_umbilical_uart(&buff);
    Ok(())
}

pub fn get_config_variable(name: ConfigVariableName) -> Result<(), ExecuteCommandErr> {
    let config_store = get_config_store();
    let value = config_store.get(name);

    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Variable: {:?} = {:?}\r\n", name, value);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}

pub fn set_config_variable(
    name: ConfigVariableName,
    value: ConfigValue,
) -> Result<(), ExecuteCommandErr> {
    let config_store = get_config_store();
    config_store.set(name, value)?;

    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Variable: {:?} set to {:?}\r\n", name, value);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}
