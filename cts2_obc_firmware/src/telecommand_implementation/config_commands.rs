use cts2_obc_telecommands::config::{ConfigValue, ConfigVariableName};
use cts2_obc_telecommands::get_config_store;
use core::fmt::Write;

use crate::error::ExecuteCommandErr;
use crate::umbilical_uart::send_umbilical_uart;

pub fn get_config_variable_command(name: ConfigVariableName) -> Result<(), ExecuteCommandErr> {
    let config_store = get_config_store();
    let value = config_store.get(name);

    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Variable: {:?} = {:?}\r\n", name, value);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}

pub fn set_config_variable_command(
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
