use core::fmt::Write;

use crate::error::ExecuteCommandErr;
use crate::timekeeping::uptime_ms;
use crate::umbilical_uart::send_umbilical_uart;

pub fn get_sys_uptime_ms_telecommand(_args: &str) -> Result<(), ExecuteCommandErr> {
    let sys_time = uptime_ms();
    let buff = heapless::format!(32; "System Uptime: {} ms\r\n", sys_time)
        .unwrap()
        .into_bytes();
    send_umbilical_uart(&buff);
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
