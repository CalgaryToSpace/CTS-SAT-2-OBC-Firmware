use core::fmt::Write;
use core::str::FromStr;
use serde_json_core::to_string;

use crate::error::ExecuteCommandErr;
use crate::telecommand_registry::TELECOMMAND_DEFINITIONS;
use crate::timekeeping::uptime_ms;
use crate::umbilical_uart::send_umbilical_uart;
use cts2_obc_telecommands::config::{ConfigValue, ConfigVariableName};
use cts2_obc_telecommands::error::ConfigError;
use cts2_obc_telecommands::get_config_store;

pub mod demo_commands;

pub fn get_sys_uptime_ms_telecommand(_args: &str) -> Result<(), ExecuteCommandErr> {
    let sys_time = uptime_ms();
    let buff = heapless::format!(32; "System Uptime: {} ms\r\n", sys_time)
        .unwrap()
        .into_bytes();
    send_umbilical_uart(&buff);
    Ok(())
}

pub fn get_config_variable(args: &str) -> Result<(), ExecuteCommandErr> {
    let name = ConfigVariableName::from_str(args.trim())?;
    let config_store = get_config_store();
    let value = config_store.get(name);

    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Variable: {:?} = {:?}\r\n", name, value);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}

pub fn set_config_variable(args: &str) -> Result<(), ExecuteCommandErr> {
    let (name, value) = args
        .split_once(',')
        .ok_or(ConfigError::ConfigParseValueTypeError)?;
    let name = ConfigVariableName::from_str(name.trim())?;
    let value = ConfigValue::from_str(value.trim())?;
    let config_store = get_config_store();
    config_store.set(name, value)?;

    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Variable: {:?} set to {:?}\r\n", name, value);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}

pub fn get_obc_info_telecommand(_args: &str) -> Result<(), ExecuteCommandErr> {
    let firmware_version = env!("CARGO_PKG_VERSION");
    let build_timestamp = env!("BUILD_TIMESTAMP");
    let commit_hash = env!("COMMIT_HASH");

    // May need to define a larger buffer if more info is added - git commit hash is 40 characters long
    let mut buffer = heapless::String::<128>::new();
    let _ = write!(buffer, "Firmware Version: {}\r\n", firmware_version);
    let _ = write!(buffer, "Build Timestamp (UNIX): {}\r\n", build_timestamp);
    let _ = write!(buffer, "Commit Hash: {}\r\n", commit_hash);

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}

pub fn get_telecommand_list_telecommand(_args: &str) -> Result<(), ExecuteCommandErr> {
    // Arbitrary buffer size for testing, will need to define a larger buffer if telecommand registry becomes longer
    let mut buffer = serde_json_core::heapless::String::<1024>::new();

    for telecommand in TELECOMMAND_DEFINITIONS.iter() {
        let json_string: serde_json_core::heapless::String<128> =
            to_string(telecommand).map_err(ExecuteCommandErr::from)?;
        let _ = write!(buffer, "{}\r\n", json_string);
    }

    send_umbilical_uart(buffer.as_bytes());
    Ok(())
}
