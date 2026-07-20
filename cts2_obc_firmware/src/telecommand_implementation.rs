use core::fmt::Write;

use crate::error::ExecuteCommandErr;
use crate::timekeeping::uptime_ms;
use crate::umbilical_uart::send_umbilical_uart;
use cts2_obc_telecommands::config::{ConfigVariableName, ConfigValue};
use cts2_obc_telecommands::get_config_store;

pub mod demo_commands;

pub fn get_sys_uptime_ms_telecommand() -> Result<(), ExecuteCommandErr> {
    let sys_time = uptime_ms();
    let buff = heapless::format!(32; "System Uptime: {} ms\r\n", sys_time)
        .unwrap()
        .into_bytes();
    send_umbilical_uart(&buff);
    Ok(())
}

pub fn config_get_config_variable(name: ConfigVariableName) -> Result<(), ExecuteCommandErr>{
    let config_store = get_config_store();
    let value = config_store.get(name);

    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Variable: {:?} = {:?}\r\n", name, value);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}

pub fn config_set_config_variable(name: ConfigVariableName, value: ConfigValue) -> Result<(), ExecuteCommandErr> {
    let config_store = get_config_store();
    config_store.set(name, value)?;

    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Variable: {:?} set to {:?}\r\n", name, value);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}
