use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::interrupt::free as critical_section;

/// True after successful init.
static INIT_DONE: AtomicBool = AtomicBool::new(false);

/// Accumulated cycles (u64) to handle DWT 32-bit wrap-arounds.
static mut ACCUM_CYCLES: u64 = 0;

/// Last seen 32-bit cycle counter value.
static mut LAST_CYCCNT: u32 = 0;

/// Number of CPU cycles per millisecond (core_hz / 1000).
/// CPU core clock in Hz (set at init).
static mut CORE_HZ: u32 = 0;

#[derive(Debug)]
pub enum TimestampError {
    InvalidUTCStatus,
    InvalidTIMEALength,
    ParseIntError,
    InitFailed,
}

// Specialized error messages
impl TimestampError {
    pub fn message(&self) -> &'static str  {
        match self {
            TimestampError::InvalidUTCStatus => "UTC Timestamp status is not labelled 'VALID'",
            TimestampError::InvalidTIMEALength => "Invalid token number: Expected exactly 22 tokens",
            TimestampError::ParseIntError => "Could not parse integer values",
            TimestampError::InitFailed => "System initialization failed",
        }
    }
}

/// Initialize the DWT cycle counter. Call once during startup.
/// `core_hz` is the CPU core clock frequency in Hz (e.g. 64_000_000).
pub fn init(core_hz: u32) -> Result<(), &'static str> {
    if core_hz < 1000 {
        return Err("core_hz too small");
    }

    // store core_hz for conversions
    let _cycles_per_ms = core_hz / 1000; // keep for a quick sanity derivation if needed

    unsafe {
        // Enable trace in DCB (set TRCENA in DEMCR at 0xE000EDFC bit 24)
        let demcr_ptr = 0xE000_EDFC as *mut u32;
        let demcr = core::ptr::read_volatile(demcr_ptr);
        core::ptr::write_volatile(demcr_ptr, demcr | (1 << 24));

        // Reset the cycle counter
        let cyccnt_ptr = (0xE000_1000u32 + 0x04) as *mut u32;
        core::ptr::write_volatile(cyccnt_ptr, 0u32);

        // Enable cycle counter (DWT CTRL bit 0)
        let dwt_ctrl_ptr = 0xE000_1000 as *mut u32;
        let ctrl = core::ptr::read_volatile(dwt_ctrl_ptr);
        core::ptr::write_volatile(dwt_ctrl_ptr, ctrl | 1u32);
    }

    unsafe {
        CORE_HZ = core_hz;
        LAST_CYCCNT = 0;
    }
    unsafe {
        ACCUM_CYCLES = 0;
    }
    INIT_DONE.store(true, Ordering::Release);

    Ok(())
}

/// Returns uptime in milliseconds since `init` was called.
/// If `init` hasn't been called successfully, returns 0.
pub fn uptime_ms() -> u64 {
    if !INIT_DONE.load(Ordering::Acquire) {
        return 0;
    }

    // Read current CYCCNT (32-bit) from DWT->CYCCNT at 0xE0001004
    let now_lo: u32 = unsafe { core::ptr::read_volatile((0xE000_1000u32 + 0x04) as *const u32) };

    // Update accumulated cycles handling wrap-around inside a critical section
    critical_section(|_| {
        let last = unsafe { LAST_CYCCNT };
        let delta = now_lo.wrapping_sub(last) as u64;
        if delta != 0 {
            unsafe {
                ACCUM_CYCLES = ACCUM_CYCLES.wrapping_add(delta);
            }
            unsafe {
                LAST_CYCCNT = now_lo;
            }
        }
    });

    let total_cycles = critical_section(|_| unsafe { ACCUM_CYCLES });

    // Use 128-bit math to avoid overflow and apply rounding when converting to ms.
    let core_hz = critical_section(|_| unsafe { CORE_HZ as u128 });
    if core_hz == 0 {
        return 0;
    }
    let cycles128 = total_cycles as u128;
    let ms = (cycles128 * 1000u128 + core_hz / 2u128) / core_hz;
    ms as u64
}

// Returns UNIX timestamp
// * IMPORTANT: this function calculates global UTC time
pub fn timestamp_ms(timea_input: &str) -> Result<u64, TimestampError> {
    
    // Return Error if initiation failed
    if !INIT_DONE.load(Ordering::Acquire) {
        return Err(TimestampError::InitFailed);
    }

    // Initialize array for tokens from input, and initialize iterator
    let mut tokens = [""; 22];
    let mut timea_iterator = timea_input.split(|c| c == ',' || c == ';' || c == '*');

    // Store sliced strings from interator in array
    for i in 0..tokens.len() {
        if let Some(token) = timea_iterator.next() {
            tokens[i] = token;
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
    let year: i32 = tokens[14].parse().map_err(|_| TimestampError::ParseIntError)?;
    let month: i32 = tokens[15].parse().map_err(|_| TimestampError::ParseIntError)?;
    let day: i32 = tokens[16].parse().map_err(|_| TimestampError::ParseIntError)?;
    let hour: i32 = tokens[17].parse().map_err(|_| TimestampError::ParseIntError)?;
    let minute: i32 = tokens[18].parse().map_err(|_| TimestampError::ParseIntError)?;
    let milliseconds: i32 = tokens[19].parse().map_err(|_| TimestampError::ParseIntError)?;
    let utc_status: &str = tokens[20];

    // Reject invalid UTC status
    if (utc_status != "VALID") {
        return Err(TimestampError::InvalidUTCStatus);
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
    let seconds: u64 = (days_since_epoch as u64 * 86400) + (hour as u64 * 3600) + (minute as u64 * 60);

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
    fn timestamp_ms_accurate_UNIX() {

        // This is October 3, 2026, exactly 16:36:45 (0 milliseconds). NOTE: treat this as UTC time, not local
        let control_time: u64 = 1791045405000;

        let timea_input_log: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,VALID*1100ad64";

        let return_value: u64 = timestamp_ms(timea_input_log);

        assert_eq!(return_value, control_time);
    }

    // Test to make sure it handles INVALID UTC status correctly
    #[test]
    fn timestamp_error_test_utc_status() {
        // UTC Status set to 'INVALID'
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,INVALID*1100ad64";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(return_value, Err(TimestampError::InvalidUTCStatus)));
    }

    // Test to make sure it doesn't crash when there are missing tokens
    #[test]
    fn timestamp_error_test_invalid_length_less_than() {
        // Missing a token
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,VALID*1100ad64";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(return_value, Err(TimestampError::InvalidTIMEALength)));
    }

    // Test to make sure it doesn't crash when there are excessive/junk tokens
    #[test]
    fn timestamp_error_test_invalid_length_greater_than() {
        // Missing a token
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,5462452,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,VALID*1100ad64;Sponges";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(return_value, Err(TimestampError::InvalidTIMEALength)));
    }

    // Test with junk input that doesn't match the proper format at all
    fn timestamp_error_test_invalid_length_bad_format() {
        // Missing a token
        let invalid_timea: &str = "I am the Banana Man! I'm the Banana Man! I'm the Banana Man! Selling... bananas!";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(return_value, Err(TimestampError::InvalidTIMEALength)));
    }

    // Test parsing error handling, ensure faulty int tokens don't cause crashes
    #[test]
    fn timestamp_error_test_parse_error() {
        // Some time fields have errors in them, which should cause a parse int error
        let invalid_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,20p26,10,3,16=,36,45000,INVALID*1100ad64";

        let return_value = timestamp_ms(invalid_timea);

        assert!(matches!(return_value, Err(TimestampError::ParseIntError)));
    }

    // Test Init failed error
    #[test]
    fn timestamp_error_test_init_failed() {
        let valid_timea: &str = "#TIMEA,USB1,0,50.5,FINESTEERING,2209,515163.000,02000020,9924,16809;VALID,-2.501488425e-09,6.133312031e-10,-17.99999999630,2026,10,3,16,36,45000,VALID*1100ad64";

        // Store proper state of INIT_DONE
        let original_state = INIT_DONE.load(Ordering::Acquire);

        // Temporarily change INIT_DONE to false for testing
        INIT_DONE.store(false, Ordering::Release);

        let return_value = timestamp_ms(valid_timea);
        assert!(matches!(return_value, Err(TimestampError::InitFailed)));

        // Restore original state after test complete
        INIT_DONE.store(original_state, Ordering::Release);
    }
}