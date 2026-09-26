use core::str::FromStr;

use crate::config::ConfigVariableName;
use crate::error::ParsedTelecommandErr;

// Commands that do not accept arguments.
pub fn no_args(args: &str) -> Result<(), ParsedTelecommandErr> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(ParsedTelecommandErr::ExceededArgumentCount)
    }
}

// Validate get_config.
pub fn get_config(args: &str) -> Result<(), ParsedTelecommandErr> {
    ConfigVariableName::from_str(args)
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentValue(0))?;

    Ok(())
}

// Validate set_config.
pub fn set_config(args: &str) -> Result<(), ParsedTelecommandErr> {
    let (name, value) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;

    // Argument 0: recognized configuration variable.
    let config_name = ConfigVariableName::from_str(name)
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentValue(0))?;

    // Both currently supported configuration variables use u32.
    let expected_type = match config_name {
        ConfigVariableName::HeartbeatMs => "u32",
        ConfigVariableName::ConfigDemoVariable1 => "u32",
    };

    // Argument 1: require the correct type and parentheses.
    let (actual_type, rest) = value
        .split_once('(')
        .ok_or(ParsedTelecommandErr::InvalidArgumentType(1))?;

    if actual_type != expected_type {
        return Err(ParsedTelecommandErr::InvalidArgumentType(1));
    }

    let number = rest
        .strip_suffix(')')
        .ok_or(ParsedTelecommandErr::InvalidArgumentValue(1))?;

    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(1));
    }

    // Checking u32 also rejects values exceeding its numeric range.
    number
        .parse::<u32>()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentRange(1))?;

    Ok(())
}

pub fn unix_milliseconds(args: &str) -> Result<(), ParsedTelecommandErr> {
    if args.is_empty() || !args.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(0));
    }

    args.parse::<u64>()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentRange(0))?;

    Ok(())
}
