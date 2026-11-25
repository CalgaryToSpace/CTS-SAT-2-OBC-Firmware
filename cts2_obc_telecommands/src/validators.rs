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
        return Err(ParsedTelecommandErr::MissingArgument(0));
    }

    let mut tokens = [""; 22];
    let mut timea_iterator = args.split([',', ';', '*']);

    // Extract exactly 22 tokens from the input
    for slot in tokens.iter_mut() {
        if let Some(mut token_entry) = timea_iterator.next() {
            if token_entry.starts_with('"') {
                token_entry = &token_entry[1..];
            }
            if token_entry.ends_with('"') {
                token_entry = &token_entry[..token_entry.len() - 1];
            }

            *slot = token_entry;
        } else {
            // Return error if number of tokens is less than what's expected
            return Err(ParsedTelecommandErr::InvalidCommandFormat);
        }
    }

    // Fail if there are more than 22 elements
    if timea_iterator.next().is_some() {
        return Err(ParsedTelecommandErr::InvalidCommandFormat);
    }

    // Validate that the date/time indices (14 through 19) consist entirely of digits
    let year: i32 = tokens[14]
        .parse()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentType(14))?;

    let month: i32 = tokens[15]
        .parse()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentType(15))?;

    let day: i32 = tokens[16]
        .parse()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentType(16))?;

    let hour: i32 = tokens[17]
        .parse()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentType(17))?;

    let minute: i32 = tokens[18]
        .parse()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentType(18))?;

    let millisecond: i32 = tokens[19]
        .parse()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentType(19))?;

    // Validate that all time fields are within their respective ranges
    if !(1..=12).contains(&month) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(15));
    }

    if !(0..=23).contains(&hour) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(17));
    }

    if !(0..=59).contains(&minute) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(18));
    }

    if !(0..=59999).contains(&millisecond) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(19));
    }

    // Validate days based on the specific month (accounting for leap years)
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let max_days = match month {
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap {
                29
            } else {
                28
            }
        }
        _ => 31,
    };

    if day < 1 || day > max_days {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(16));
    }

    // 6. Validate the UTC status string at index 20
    if tokens[20] != "VALID" {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(20));
    }

    Ok(())
}

pub fn hex_address(value: &str) -> Result<u32, ParsedTelecommandErr> {
    let digits = value
        .strip_prefix("0x")
        .ok_or(ParsedTelecommandErr::MissingHexPrefix)?;

    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ParsedTelecommandErr::InvalidHexDigits);
    }

    u32::from_str_radix(digits, 16).map_err(|_| ParsedTelecommandErr::AddressOutOfRange)
}

pub fn hex_data(value: &str) -> Result<&str, ParsedTelecommandErr> {
    let digits = value
        .strip_prefix("0x")
        .ok_or(ParsedTelecommandErr::MissingHexPrefix)?;

    if !digits.len().is_multiple_of(2) || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ParsedTelecommandErr::InvalidWriteData);
    }

    Ok(digits)
}

pub fn flash_read(args: &str) -> Result<(), ParsedTelecommandErr> {
    parse_flash_read(args).map(|_| ())
}

pub const MAX_FLASH_READ_BYTES: u32 = 256;

pub fn parse_flash_read(args: &str) -> Result<(u32, u32), ParsedTelecommandErr> {
    let (start, length) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;
    let start = hex_address(start)?;

    if length.is_empty() || !length.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ParsedTelecommandErr::InvalidByteCount);
    }

    let length = length
        .parse::<u32>()
        .map_err(|_| ParsedTelecommandErr::InvalidReadLength)?;

    if length == 0 || length > MAX_FLASH_READ_BYTES {
        return Err(ParsedTelecommandErr::InvalidReadLength);
    }

    Ok((start, length))
}

pub fn flash_write(args: &str) -> Result<(), ParsedTelecommandErr> {
    parse_flash_write(args).map(|_| ())
}

pub fn parse_flash_write(args: &str) -> Result<(u32, &str), ParsedTelecommandErr> {
    let (start, data) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;

    let start = hex_address(start)?;
    let data = hex_data(data)?;

    Ok((start, data))
}

pub fn flash_erase(args: &str) -> Result<(), ParsedTelecommandErr> {
    parse_flash_erase(args).map(|_| ())
}

pub fn parse_flash_erase(args: &str) -> Result<(u32, u32), ParsedTelecommandErr> {
    let (start, page_count) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;
    let start = hex_address(start)?;

    if page_count.is_empty() || !page_count.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ParsedTelecommandErr::InvalidPageCount);
    }

    let page_count = page_count
        .parse::<u32>()
        .map_err(|_| ParsedTelecommandErr::InvalidPageCount)?;

    Ok((start, page_count))
}

fn is_valid_log_sink_name(value: &str) -> bool {
    matches!(
        value.trim(),
        "UHF_RADIO" | "UHF" | "FILE" | "UMBILICAL" | "UMBILICAL_UART"
    )
}

fn is_valid_log_system_name(value: &str) -> bool {
    matches!(
        value.trim(),
        "OBC"
            | "UHF_RADIO"
            | "UHF"
            | "UMBILICAL"
            | "UMBILICAL_UART"
            | "GNSS"
            | "MPI"
            | "EPS"
            | "BOOM"
            | "ADCS"
            | "LFS"
            | "FLASH"
            | "ANTENNA_DEPLOY"
            | "ANTS"
            | "LOG"
            | "TELECOMMAND"
            | "UNIT_TEST"
            | "TEST"
            | "UNKNOWN"
    )
}

pub fn set_log_sink_enabled(args: &str) -> Result<(), ParsedTelecommandErr> {
    let (sink_name, enabled_value) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;

    if !is_valid_log_sink_name(sink_name) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(0));
    }

    let (actual_type, rest) = enabled_value
        .trim()
        .split_once('(')
        .ok_or(ParsedTelecommandErr::InvalidArgumentType(1))?;

    if actual_type != "bool" {
        return Err(ParsedTelecommandErr::InvalidArgumentType(1));
    }

    let enabled = rest
        .strip_suffix(')')
        .ok_or(ParsedTelecommandErr::InvalidArgumentValue(1))?;

    match enabled {
        "true" | "false" => Ok(()),
        _ => Err(ParsedTelecommandErr::InvalidArgumentValue(1)),
    }
}

pub fn get_log_sink_config(args: &str) -> Result<(), ParsedTelecommandErr> {
    if !is_valid_log_sink_name(args) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(0));
    }

    Ok(())
}

pub fn get_log_system_config(args: &str) -> Result<(), ParsedTelecommandErr> {
    if !is_valid_log_system_name(args) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(0));
    }

    Ok(())
}

pub fn set_log_system_severity_mask(args: &str) -> Result<(), ParsedTelecommandErr> {
    let (system_name, mask_value) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;

    if !is_valid_log_system_name(system_name) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(0));
    }

    let (actual_type, rest) = mask_value
        .trim()
        .split_once('(')
        .ok_or(ParsedTelecommandErr::InvalidArgumentType(1))?;

    if actual_type != "u32" {
        return Err(ParsedTelecommandErr::InvalidArgumentType(1));
    }

    let value = rest
        .strip_suffix(')')
        .ok_or(ParsedTelecommandErr::InvalidArgumentValue(1))?;

    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(1));
    }

    let mask = value
        .parse::<u32>()
        .map_err(|_| ParsedTelecommandErr::InvalidArgumentRange(1))?;

    if mask & !0b11111 != 0 {
        return Err(ParsedTelecommandErr::InvalidArgumentRange(1));
    }

    Ok(())
}

pub fn set_log_system_file_logging_enabled(args: &str) -> Result<(), ParsedTelecommandErr> {
    let (system_name, enabled_value) = args
        .split_once(',')
        .ok_or(ParsedTelecommandErr::MissingArgument(1))?;

    if !is_valid_log_system_name(system_name) {
        return Err(ParsedTelecommandErr::InvalidArgumentValue(0));
    }

    let (actual_type, rest) = enabled_value
        .trim()
        .split_once('(')
        .ok_or(ParsedTelecommandErr::InvalidArgumentType(1))?;

    if actual_type != "bool" {
        return Err(ParsedTelecommandErr::InvalidArgumentType(1));
    }

    let enabled = rest
        .strip_suffix(')')
        .ok_or(ParsedTelecommandErr::InvalidArgumentValue(1))?;

    match enabled {
        "true" | "false" => Ok(()),
        _ => Err(ParsedTelecommandErr::InvalidArgumentValue(1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_write_checks_syntax_only() {
        assert_eq!(
            parse_flash_write("0x08000000,0x01aBFF"),
            Ok((0x0800_0000, "01aBFF"))
        );
        assert_eq!(parse_flash_write("0x08000000,0x"), Ok((0x0800_0000, "")));
        assert_eq!(flash_write("0x08000001,0x01"), Ok(()));
        assert_eq!(flash_write("0xFFFFFFFF,0x0102"), Ok(()));
    }

    #[test]
    fn flash_write_rejects_malformed_arguments() {
        assert_eq!(
            flash_write("0x08000000"),
            Err(ParsedTelecommandErr::MissingArgument(1))
        );
    }

    #[test]
    fn data_accepts_large_hex_payload() {
        let digits = "01aBFF80".repeat(256);
        let value = std::format!("0x{digits}");
        assert_eq!(hex_data(&value), Ok(digits.as_str()));
    }

    #[test]
    fn data_requires_prefixed_hex_byte_pairs() {
        for value in ["", "01AB", "0X01"] {
            assert_eq!(
                hex_data(value),
                Err(ParsedTelecommandErr::MissingHexPrefix),
                "{value}"
            );
        }
        for value in ["0x0", "0xGG", "0x01_02", "0x01 02", "0x01,02", "0xé"] {
            assert_eq!(
                hex_data(value),
                Err(ParsedTelecommandErr::InvalidWriteData),
                "{value}"
            );
        }
    }

    #[test]
    fn address_requires_prefixed_hex() {
        // Valid hex strings
        assert_eq!(hex_address("0x08000000"), Ok(0x0800_0000));
        assert_eq!(hex_address("0xabcdefAB"), Ok(0xabcd_efab));
        assert_eq!(hex_address("0xFFFFFFFF"), Ok(u32::MAX));

        // Invalid hex prefix
        for value in ["08000000", "134217728", ""] {
            assert_eq!(
                hex_address(value),
                Err(ParsedTelecommandErr::MissingHexPrefix),
                "{value}"
            );
        }

        // Invalid whole hex strings
        for value in ["0x", "0x+1", "0xGG", "0x08_00", "0x 08"] {
            assert_eq!(
                hex_address(value),
                Err(ParsedTelecommandErr::InvalidHexDigits),
                "{value}"
            );
        }

        // Overflow address
        assert_eq!(
            hex_address("0x100000000"),
            Err(ParsedTelecommandErr::AddressOutOfRange)
        );
    }

    #[test]
    fn flash_read_checks_syntax_and_length_limit() {
        assert_eq!(flash_read("0x08000000,16"), Ok(()));
        assert_eq!(flash_read("0xFFFFFFFF,1"), Ok(()));
        assert_eq!(flash_read("0xFFFFFFFF,2"), Ok(()));
        assert_eq!(
            parse_flash_read(&std::format!("0x08000000,{MAX_FLASH_READ_BYTES}")),
            Ok((0x0800_0000, MAX_FLASH_READ_BYTES))
        );

        for length in [
            0,
            u64::from(MAX_FLASH_READ_BYTES) + 1,
            u64::from(u32::MAX) + 1,
        ] {
            let args = std::format!("0x08000000,{length}");
            assert_eq!(
                flash_read(&args),
                Err(ParsedTelecommandErr::InvalidReadLength),
                "{args}"
            );
        }
    }

    #[test]
    fn flash_erase_checks_syntax_only() {
        assert_eq!(parse_flash_erase("0x080FE000,2"), Ok((0x080F_E000, 2)));
        assert_eq!(parse_flash_erase("0x08000001,0"), Ok((0x0800_0001, 0)));
        assert_eq!(flash_erase("0xFFFFFFFF,4294967295"), Ok(()));
    }

    #[test]
    fn flash_erase_rejects_malformed_arguments() {
        assert_eq!(
            flash_erase("0x080FE000"),
            Err(ParsedTelecommandErr::MissingArgument(1))
        );
        assert_eq!(
            flash_erase("080FE000,1"),
            Err(ParsedTelecommandErr::MissingHexPrefix)
        );
        for page_count in ["", "-1", "+1", "0x1", "1.5", " 1", "1,1", "4294967296"] {
            let args = std::format!("0x080FE000,{page_count}");
            assert_eq!(
                flash_erase(&args),
                Err(ParsedTelecommandErr::InvalidPageCount),
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
            Err(ParsedTelecommandErr::MissingHexPrefix)
        );

        // Non-decimal length values
        for length in ["", "-1", "+1", "0x10", "1.5", " 16", "16,1"] {
            let args = std::format!("0x08000000,{length}");
            assert_eq!(
                flash_read(&args),
                Err(ParsedTelecommandErr::InvalidByteCount),
                "{args}"
            );
        }
    }
}
