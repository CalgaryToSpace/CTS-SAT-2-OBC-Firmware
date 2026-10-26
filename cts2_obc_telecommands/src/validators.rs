use crate::error::ParsedTelecommandErr;
use crate::get_config_store;

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
    let config_store = get_config_store();
    config_store
        .get(args)
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentValue(0))?;

    Ok(())
}

// Validate set_config.
pub fn set_config(args: &str) -> Result<(), ParsedTelecommandErr> {
    let (name, value) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;
    let config_store = get_config_store();

    // Argument 0: recognized configuration variable.
    let config_name = config_store
        .get(name)
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentValue(0))?;

    // Both currently supported configuration variables use u32.
    // let expected_type = match config_name {
    //     ConfigVariableName::HeartbeatMs => "u32",
    //     ConfigVariableName::ConfigDemoVariable1 => "u32",
    // };

    let expected_type = config_name.type_str();

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

pub fn timestamp_timea_log(args: &str) -> Result<(), ParsedTelecommandErr> {
    if args.is_empty() {
        return Err(ParsedTelecommandErr::InvalidArgumentRange(0));
    }

    let mut tokens = [""; 22];
    let mut timea_iterator = args.split([',', ';', '*']);

    // 3. Extract exactly 22 tokens from the input
    for slot in tokens.iter_mut() {
        if let Some(token_entry) = timea_iterator.next() {
            *slot = token_entry;
        } else {
            // Return error if number of tokens is less than what's expected
            return Err(ParsedTelecommandErr::InvalidArgumentRange(0));
        }
    }

    // 4. Fail if there are more than 22 elements
    if timea_iterator.next().is_some() {
        return Err(ParsedTelecommandErr::InvalidArgumentRange(0));
    }

    // 5. Validate that the date/time indices (14 through 19) consist entirely of digits
    for (i, token_entry) in tokens[14..=19].iter().enumerate() {
        if token_entry.is_empty() || !token_entry.chars().all(|c| c.is_ascii_digit()) {
            return Err(ParsedTelecommandErr::InvalidArgumentRange((14 + i) as u8));
        }
    }

    // 6. Validate the UTC status string at index 20
    if tokens[20] != "VALID" {
        return Err(ParsedTelecommandErr::InvalidArgumentRange(20));
    }

    Ok(())
}
