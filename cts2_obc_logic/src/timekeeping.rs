#[derive(Debug)]
pub enum TimestampError {
    InvalidUTCStatus,
    InvalidTIMEALength,
    ParseIntError,
    InitFailed,
    ValueOutOfRange,
}

// Specialized error messages
impl TimestampError {
    pub fn message(&self) -> &'static str {
        match self {
            TimestampError::InvalidUTCStatus => "UTC Timestamp status is not labelled 'VALID'",
            TimestampError::InvalidTIMEALength => {
                "Invalid token number: Expected exactly 22 tokens"
            }
            TimestampError::ParseIntError => "Could not parse integer values",
            TimestampError::InitFailed => "System initialization failed",
            TimestampError::ValueOutOfRange => {
                "At least one value from the TIMEA log is out of range"
            }
        }
    }
}

// Returns UNIX timestamp
// * IMPORTANT: this function calculates global UTC time
pub fn timestamp_ms(timea_input: &str) -> Result<u64, TimestampError> {
    // Initialize array for tokens from input, and initialize iterator
    let mut tokens = [""; 22];
    let mut timea_iterator = timea_input.split([',', ';', '*']);

    // Store sliced strings from interator in array
    for slot in tokens.iter_mut() {
        if let Some(token_entry) = timea_iterator.next() {
            *slot = token_entry;
        } else {
            // Return error if number of tokens is less than what's expected
            return Err(TimestampError::InvalidTIMEALength);
        }
    }

    // Returns error if there's more than 22 items
    if timea_iterator.next().is_some() {
        return Err(TimestampError::InvalidTIMEALength);
    }

    // Safely parse important UTC date & time components
    // Throw error if data from these fields is not valid
    let year: i32 = tokens[14]
        .parse()
        .map_err(|_| TimestampError::ParseIntError)?;
    let month: i32 = tokens[15]
        .parse()
        .map_err(|_| TimestampError::ParseIntError)?;
    let day: i32 = tokens[16]
        .parse()
        .map_err(|_| TimestampError::ParseIntError)?;
    let hour: i32 = tokens[17]
        .parse()
        .map_err(|_| TimestampError::ParseIntError)?;
    let minute: i32 = tokens[18]
        .parse()
        .map_err(|_| TimestampError::ParseIntError)?;
    let milliseconds: i32 = tokens[19]
        .parse()
        .map_err(|_| TimestampError::ParseIntError)?;
    let utc_status: &str = tokens[20];

    // Reject invalid UTC status
    if utc_status != "VALID" {
        return Err(TimestampError::InvalidUTCStatus);
    }

    // Check to ensure each time field is within the proper range
    if !(1..=12).contains(&month)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=60999).contains(&milliseconds)
    {
        return Err(TimestampError::ValueOutOfRange);
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
        return Err(TimestampError::ValueOutOfRange);
    }

    // Embedded trick treats March as the first month of the year,
    // and Jan/Feb as the 13th/14th month of the previous year
    // Makes it easier to account for leap years
    let (y, m) = if month <= 2 {
        (year - 1, month + 9)
    } else {
        (year, month - 3)
    };

    // Calculate total number of days that have passed since year 0. Also accounts for leap years
    let total_days = (365 * y) + (y / 4) - (y / 100) + (y / 400) + (((153 * m) + 2) / 5) + day;

    // Calculate total days that have passed since UNIX Epoch Time started on Jan 1 1970
    let days_since_epoch = total_days - 719469;

    // Get total seconds since epoch time
    let seconds: u64 =
        (days_since_epoch as u64 * 86400) + (hour as u64 * 3600) + (minute as u64 * 60);

    // Get total milliseconds. Add 21600000 for local time
    let unix_time_ms: u64 = (seconds * 1000) + (milliseconds as u64);

    Ok(unix_time_ms)
}

// Note: This is a valid TIMEA Log (for testing get_timestamp()):
// #TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,9,29,19,03,45000,VALID*1100ad64

// Various tests for get_timestamp()
#[cfg(test)]
mod tests {
    use super::*;

    // Testing to make sure the time returned is accurate to the actual UNIX time
    #[test]
    fn timestamp_ms_accurate_unix() {
        // This is October 3, 2026, exactly 16:36:45 (0 milliseconds). NOTE: treat this as UTC time, not local
        let control_time: u64 = 1791045405000;

        let timea_input_log: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,VALID*1100ad64";

        let return_value: u64 = timestamp_ms(timea_input_log).unwrap();

        assert_eq!(return_value, control_time);
    }

    // Test to make sure it handles INVALID UTC status correctly
    #[test]
    fn timestamp_error_test_utc_status() {
        // UTC Status set to 'INVALID'
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,INVALID*1100ad64";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(
            return_value,
            Err(TimestampError::InvalidUTCStatus)
        ));
    }

    // Test to make sure it doesn't crash when there are missing tokens
    #[test]
    fn timestamp_error_test_invalid_length_less_than() {
        // Missing a token
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,VALID*1100ad64";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(
            return_value,
            Err(TimestampError::InvalidTIMEALength)
        ));
    }

    // Test to make sure it doesn't crash when there are excessive/junk tokens
    #[test]
    fn timestamp_error_test_invalid_length_greater_than() {
        // Missing a token
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,5462452,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,VALID*1100ad64;Sponges";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(
            return_value,
            Err(TimestampError::InvalidTIMEALength)
        ));
    }

    // Test with junk input that doesn't match the proper format at all
    #[test]
    fn timestamp_error_test_invalid_length_bad_format() {
        // Missing a token
        let invalid_timea: &str =
            "I am the Banana Man! I'm the Banana Man! I'm the Banana Man! Selling... bananas!";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(
            return_value,
            Err(TimestampError::InvalidTIMEALength)
        ));
    }

    // Test parsing error handling, ensure faulty int tokens don't cause crashes
    #[test]
    fn timestamp_error_test_parse_error() {
        // Some time fields have errors in them, which should cause a parse int error
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,20p26,10,3,16=,36,45000,INVALID*1100ad64";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(return_value, Err(TimestampError::ParseIntError)));
    }

    #[test]
    fn timestamp_error_test_value_out_of_range() {
        // Month is set to 13, beyond the proper range for months
        let out_of_range_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,13,3,16,36,45000,VALID*1100ad64";

        let return_value = timestamp_ms(out_of_range_timea);

        assert!(matches!(return_value, Err(TimestampError::ValueOutOfRange)));
    }
}
