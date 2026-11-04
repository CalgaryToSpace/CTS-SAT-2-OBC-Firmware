use core::fmt::Write;

use cts2_obc_telecommands::error::ConfigError;

use crate::error::ExecuteCommandErr;
use crate::log::{
    LogSink, LogSystem, log_get_sink_configuration, log_get_system_configuration,
    log_set_sink_enabled, log_set_system_file_logging_enabled_state,
    log_set_system_severity_mask,
};
use crate::umbilical_uart::send_umbilical_uart;

pub fn set_log_sink_enabled_telecommand(args: &str) -> Result<(), ExecuteCommandErr> {
    let (sink_name, enabled_value) = args
        .split_once(',')
        .ok_or(ConfigError::ConfigParseValueTypeError)?;

    let sink = LogSink::try_from(sink_name)?;

    let (value_type, raw_value) = enabled_value
        .trim()
        .split_once('(')
        .ok_or(ConfigError::ConfigParseValueTypeError)?;

    if value_type != "bool" {
        return Err(ConfigError::ConfigParseValueTypeError.into());
    }

    let enabled = match raw_value.strip_suffix(')') {
        Some("true") => true,
        Some("false") => false,
        _ => return Err(ConfigError::ConfigParseValueTypeError.into()),
    };

    log_set_sink_enabled(sink, enabled);

    let mut response = heapless::String::<64>::new();
    let status = if enabled { "enabled" } else { "disabled" };
    let _ = write!(response, "Log sink {} has been {}", sink.as_str(), status);

    send_umbilical_uart(response.as_bytes());
    send_umbilical_uart(b"\r\n");

    Ok(())
}

pub fn get_log_sink_config_telecommand(args: &str) -> Result<(), ExecuteCommandErr> {
    let sink = LogSink::try_from(args)?;

    let config = log_get_sink_configuration(sink)
        .ok_or(ConfigError::ConfigVariableNotFound)?;

    if let Some(response) = config.format() {
        send_umbilical_uart(response.as_bytes());
        send_umbilical_uart(b"\r\n");
    }

    Ok(())
}

pub fn get_log_system_config_telecommand(args: &str) -> Result<(), ExecuteCommandErr> {
    let system = LogSystem::try_from(args)?;

    let config = log_get_system_configuration(system)
        .ok_or(ConfigError::ConfigVariableNotFound)?;

    if let Some(response) = config.format() {
        send_umbilical_uart(response.as_bytes());
        send_umbilical_uart(b"\r\n");
    }

    Ok(())
}

pub fn set_log_system_severity_mask_telecommand(args: &str) -> Result<(), ExecuteCommandErr> {
    let (system_name, mask_value) = args
        .split_once(',')
        .ok_or(ConfigError::ConfigParseValueTypeError)?;

    let system = LogSystem::try_from(system_name)?;

    let (value_type, raw_value) = mask_value
        .trim()
        .split_once('(')
        .ok_or(ConfigError::ConfigParseValueTypeError)?;

    if value_type != "u32" {
        return Err(ConfigError::ConfigParseValueTypeError.into());
    }

    let severity_mask = raw_value
        .strip_suffix(')')
        .ok_or(ConfigError::ConfigParseValueTypeError)?
        .parse::<u32>()
        .map_err(|_| ConfigError::ConfigParseValueTypeError)?;

    log_set_system_severity_mask(system, severity_mask);

    let mut response = heapless::String::<64>::new();
    let _ = write!(
        response,
        "Updated severity mask for {} to {:#07b}",
        system.as_str(),
        severity_mask
    );

    send_umbilical_uart(response.as_bytes());
    send_umbilical_uart(b"\r\n");

    Ok(())
}

pub fn set_log_system_file_logging_enabled_telecommand(args: &str) -> Result<(), ExecuteCommandErr> {
    let (system_name, enabled_value) = args
        .split_once(',')
        .ok_or(ConfigError::ConfigParseValueTypeError)?;

    let system = LogSystem::try_from(system_name)?;

    let (value_type, raw_value) = enabled_value
        .trim()
        .split_once('(')
        .ok_or(ConfigError::ConfigParseValueTypeError)?;

    if value_type != "bool" {
        return Err(ConfigError::ConfigParseValueTypeError.into());
    }

    let enabled = match raw_value.strip_suffix(')') {
        Some("true") => true,
        Some("false") => false,
        _ => return Err(ConfigError::ConfigParseValueTypeError.into()),
    };

    log_set_system_file_logging_enabled_state(system, enabled);

    let mut response = heapless::String::<64>::new();
    let status = if enabled { "enabled" } else { "disabled" };
    let _ = write!(response, "File logging for {} has been {}", system.as_str(), status);

    send_umbilical_uart(response.as_bytes());
    send_umbilical_uart(b"\r\n");

    Ok(())
}