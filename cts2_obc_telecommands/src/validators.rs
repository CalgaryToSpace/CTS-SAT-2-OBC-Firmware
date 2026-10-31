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

pub fn hex_address(value: &str, index: u8) -> Result<u32, ParsedTelecommandErr> {
    let digits = value
        .strip_prefix("0x")
        .ok_or(ParsedTelecommandErr::InvalidArgumentValue(index))?;

    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(index));
    }

    u32::from_str_radix(digits, 16).map_err(|_| ParsedTelecommandErr::InvalidArgumentRange(index))
}

pub fn flash_read(args: &str) -> Result<(), ParsedTelecommandErr> {
    let (start, length) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;
    let start = hex_address(start, 0)?;

    if length.is_empty() || !length.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(1));
    }

    let length = length
        .parse::<u32>()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentRange(1))?;

    length
        .checked_sub(1)
        .and_then(|offset| start.checked_add(offset))
        .ok_or(ParsedTelecommandErr::InvalidArgumentRange(1))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_requires_prefixed_hex() {
        // Valid hex strings
        assert_eq!(hex_address("0x08000000", 0), Ok(0x0800_0000));
        assert_eq!(hex_address("0xabcdefAB", 0), Ok(0xabcd_efab));
        assert_eq!(hex_address("0xFFFFFFFF", 0), Ok(u32::MAX));

        // Invalid hex strings
        for value in [
            "08000000",
            "134217728",
            "",
            "0x",
            "0x+1",
            "0xGG",
            "0x08_00",
            "0x 08",
        ] {
            assert_eq!(
                hex_address(value, 2),
                Err(ParsedTelecommandErr::InvalidArgumentValue(2)),
                "{value}"
            );
        }

        // Overflow address
        assert_eq!(
            hex_address("0x100000000", 2),
            Err(ParsedTelecommandErr::InvalidArgumentRange(2))
        );
    }

    #[test]
    fn flash_read_checks_inclusive_range() {
        // Valid ranges
        assert_eq!(flash_read("0x08000000,16"), Ok(()));
        assert_eq!(flash_read("0xFFFFFFFF,1"), Ok(()));

        // Errors: zero length, overflow, and exceeding u32 range
        for args in ["0x08000000,0", "0xFFFFFFFF,2", "0x08000000,4294967296"] {
            assert_eq!(
                flash_read(args),
                Err(ParsedTelecommandErr::InvalidArgumentRange(1)),
                "{args}"
            );
        }
    }

    #[test]
    fn flash_read_rejects_malformed_arguments() {
        assert_eq!(
            flash_read("0x08000000"),
            Err(ParsedTelecommandErr::MissingArgument(1))
        );

        // Invalid start address
        assert_eq!(
            flash_read("08000000,16"),
            Err(ParsedTelecommandErr::InvalidArgumentValue(0))
        );

        // Non-decimal length values
        for length in ["", "-1", "+1", "0x10", "1.5", " 16", "16,1"] {
            let args = std::format!("0x08000000,{length}");
            assert_eq!(
                flash_read(&args),
                Err(ParsedTelecommandErr::InvalidArgumentValue(1)),
                "{args}"
            );
        }
    }
}
